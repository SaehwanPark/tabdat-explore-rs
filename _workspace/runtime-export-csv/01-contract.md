# Bounded eager-runtime CSV `export` contract

Status: accepted after PR #72 (`5cb5b34`) squash merge; post-merge workflow
verification is recorded in `02-evidence-migration.md`.

Producer: task owner, using `tabdat-migration`, `tabdat-data-semantics`, and
`simple-code-writer`.
Consumer: the bounded eager CSV export implementation and its focused review.

## Scope

Execute the already-parsed form `export <path> [, replace]` against the active
eager local-Parquet DuckDB relation.

This slice will:

- require an active eager local-Parquet relation;
- accept only a `.csv` output path (case-insensitive extension);
- create missing parent directories;
- reject an existing output unless `replace` is true;
- reject existing directories as output targets;
- write the currently published relation with a header, schema order, row
  order, row count, and SQL NULL values preserved; and
- return an owned output path and output-oriented dataset metadata without
  changing active session state.

`save` is already accepted separately for Parquet. Parquet aliasing through
`export`, Feather/Arrow output, lazy or materialized relations, metadata
serialization, labels, panel state, CLI/JSON/MCP surfaces, and broad Python
export parity are explicitly deferred.

## Python contract

Pinned authority:

- repository: `https://github.com/SaehwanPark/tabdat-explore.git`;
- revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`;
- tree: `601b236788872323af9277d2276a236154a0f129`;
- package: `0.25.0`; and
- `uv.lock` SHA-256:
  `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.

Relevant authority paths:

- `src/tabdat/backend.py:778-795` — supported suffixes, overwrite policy,
  parent creation, and CSV/Feather dispatch;
- `src/tabdat/backend.py:3617-3628` — output-target validation helper; and
- `tests/test_executor.py:9472-9530` — transformed export, overwrite gating,
  and exact CSV/Parquet/Feather result checks.

The bounded Python behavior is:

1. no active dataset raises the command-specific active-dataset error;
2. the `.csv` destination is accepted after user-path expansion in Python;
3. an existing path is rejected unless `replace` is requested, and an
   existing directory is never treated as a file target;
4. missing parent directories are created;
5. the active relation is exported in its current schema and row order with a
   CSV header and blank CSV fields for SQL NULL values; and
6. successful output does not replace or otherwise mutate the active relation.

The Python implementation supports Parquet and Feather plus lazy/materialized
Polars behavior. Those observations constrain later work but are not part of
this bounded Rust slice.

## Rust contract

Add an owned `ExportResult` and `ExecutionResult::Export`. The result contains
the requested output `PathBuf` and a `DatasetInfo` whose source is that output
path, with schema, row count, and execution metadata copied from the active
relation. The session's published `active_dataset` remains unchanged. Add
typed runtime errors for:

- no active dataset (`NoActiveDataset { command: "export" }`);
- unsupported output format (`ExportUnsupportedFormat { path }`);
- an existing target without `replace` (`ExportTargetExists { path }`);
- an existing target that is not a regular file (`ExportTargetNotAFile { path }`);
- path conversion or directory preparation failure; and
- backend/write failure (`ExportFailed { path }`).

The implementation must validate the command and output path before invoking
DuckDB, bind the path through the backend API rather than interpolating raw
user text into SQL, and only return success after the copy completes. It must
not alter `active_dataset`, `label_metadata`, or the private active relation.
A failed copy may leave an implementation-created partial file only if the
backend cannot provide stronger atomic output; that limitation must be
covered and documented rather than hidden.

## State-transition contract

| Situation | Active relation | Published metadata | Backend initialization |
| --- | --- | --- | --- |
| No active dataset | unchanged | unchanged | must not occur |
| Format/path/overwrite validation failure | unchanged | unchanged | already-loaded backend only |
| Directory or DuckDB write failure | unchanged | unchanged | already-loaded backend only |
| Successful export | unchanged | unchanged | already-loaded backend |

## Test contract

Focused Rust coverage must include:

- no active dataset without initializing DuckDB;
- successful export after `use`, with an exact typed `ExportResult`;
- exact CSV header and fixture bytes for schema order, row order, decimals,
  quoted text, and NULL values;
- transformed active data being what is exported;
- case-insensitive `.CSV` acceptance and unsupported-suffix rejection;
- existing-file rejection without `replace` and successful replacement with it;
- parent-directory creation and existing-directory rejection;
- quoted/path-containing-space output names; and
- failed output preparation or backend copy preserving active metadata and rows.

The pinned oracle probes are:

```sh
cd ../tabdat-explore
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_executor.py -k 'phase_9_export_writes_supported_formats'
```

The broader parser/script oracle, focused Rust tests, locked workspace checks,
policy checks, `git diff --check`, PR-head CI, and merge-head CI are required
for completion. The Python oracle is behavioral evidence; it is not a Rust
runtime dependency.

## Implementation mapping and deferrals

- Native Rust: owned result/error types, path validation, and state-preserving
  dispatch.
- DuckDB adapter: bound `COPY (SELECT * FROM __tabdat_active) TO ? (FORMAT CSV,
  HEADER)` or the equivalent parameterized backend call.
- Deferred: Parquet/Feather/Arrow export aliases, lazy/materialized output,
  `~` expansion and broader path normalization, label/panel persistence,
  atomic temporary-file replacement, CLI/JSON/MCP, and general
  session/table-registry semantics.

Completion is `partial` until implementation, independent review, hosted
acceptance, merge, and post-merge workflow evidence are recorded.
