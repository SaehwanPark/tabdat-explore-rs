# Bounded eager-runtime `save` contract

Status: contract checkpoint; implementation and validation are in progress.

Producer: task owner, using `tabdat-migration`, `tabdat-data-semantics`, and
`simple-code-writer`.
Consumer: the bounded eager runtime implementation and its focused review.

## Scope

Execute the already-parsed form `save <path> [, replace]` against the active
eager local-Parquet DuckDB relation.

This slice will:

- require an active eager local-Parquet relation;
- accept only a `.parquet` output path (case-insensitive extension);
- create missing parent directories;
- reject an existing output unless `replace` is true;
- write the currently published relation without changing active session state;
- return an owned output path and output-oriented dataset metadata; and
- preserve schema order, row order, row count, and SQL NULL values in the
  round-tripped Parquet file.

`export`, CSV/Feather/Arrow output, lazy or materialized relations, metadata
serialization, labels, panel state, CLI/JSON/MCP surfaces, and broad persistence
parity are explicitly deferred.

## Python contract

Pinned authority:

- repository: `https://github.com/SaehwanPark/tabdat-explore.git`;
- revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`;
- tree: `601b236788872323af9277d2276a236154a0f129`;
- package: `0.25.0`;
- Python: `3.13.3` in the recorded oracle environment; and
- `uv.lock` SHA-256:
  `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.

Relevant authority paths:

- `src/tabdat/models.py:453-463` — `SaveCommand(path, replace)`;
- `src/tabdat/parser.py:704-708,1612-1624` — path and flag parsing;
- `src/tabdat/backend.py:758-778` — Parquet-only output, overwrite policy,
  parent creation, and DuckDB copy;
- `src/tabdat/backend.py:3617-3628` — output-path validation helper; and
- `tests/test_executor.py:9442-9470` — transformed-data output, overwrite,
  and replacement behavior.

The bounded Python behavior is:

1. no active dataset raises the command-specific active-dataset error;
2. a non-`.parquet` path is rejected before writing;
3. an existing path is rejected unless `replace` is requested, and an existing
   directory is never treated as a file target;
4. missing parent directories are created;
5. the active relation is copied to Parquet in its current schema and row order;
6. successful output does not replace or otherwise mutate the active relation;
   and
7. write or backend failures become a deterministic command error naming the
   original path.

The Python implementation supports broader lazy/materialized and reporting
behavior. Those observations constrain later work but are not part of this
bounded Rust slice.

## Rust contract

Add an owned `SaveResult` and `ExecutionResult::Save`. The result contains the
requested output `PathBuf` and a `DatasetInfo` whose source is that output path,
with schema, row count, and execution metadata copied from the active relation.
The session's published `active_dataset` remains unchanged. Add typed runtime
errors for:

- no active dataset (`NoActiveDataset { command: "save" }`);
- unsupported output format (`SaveUnsupportedFormat { path }`);
- an existing target without `replace` (`SaveTargetExists { path }`);
- an existing target that is not a regular file (`SaveTargetNotAFile { path }`);
- path conversion or directory preparation failure; and
- backend/write failure (`SaveFailed { path }`).

The implementation must validate the command and output path before invoking
DuckDB, quote/bind the path through the backend API rather than interpolating
raw user text into SQL, and only return success after the copy completes. It
must not alter `active_dataset`, `label_metadata`, or the private active
relation. A failed copy may leave an implementation-created partial file only
if the backend cannot provide stronger atomic output; that limitation must be
covered and documented rather than hidden.

## State-transition contract

| Situation | Active relation | Published metadata | Backend initialization |
| --- | --- | --- | --- |
| No active dataset | unchanged | unchanged | must not occur |
| Format/path/overwrite validation failure | unchanged | unchanged | already-loaded backend only |
| Directory or DuckDB write failure | unchanged | unchanged | already-loaded backend only |
| Successful save | unchanged | unchanged | already-loaded backend |

## Test contract

Focused Rust coverage must include:

- no active dataset without initializing DuckDB;
- successful save after `use`, with an exact typed `SaveResult`;
- read-back of schema order, row order, row count, and NULL values;
- transformed active data (for example, `generate`) being what is written;
- case-insensitive `.parquet` acceptance and unsupported-extension rejection;
- existing-file rejection without `replace` and successful replacement with it;
- parent-directory creation and existing-directory rejection;
- quoted/path-containing-space output names; and
- failed output preparation or backend copy preserving active metadata and rows.

The pinned oracle probes are:

```sh
cd ../tabdat-explore
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_executor.py -k 'phase_9_save_writes_transformed_active_dataset'
```

The broader parser/script oracle, focused Rust tests, locked workspace checks,
policy checks, `git diff --check`, PR-head CI, and merge-head CI are required
for completion. The Python oracle is behavioral evidence; it is not a Rust
runtime dependency.

## Implementation mapping and deferrals

- Native Rust: owned result/error types, path validation, and state-preserving
  dispatch.
- DuckDB adapter: bound `COPY (SELECT * FROM __tabdat_active) TO ? (FORMAT
  PARQUET)` or the equivalent parameterized backend call.
- Deferred: `export`, CSV/Feather/Arrow writers, lazy/materialized output,
  label/panel persistence, atomic temporary-file replacement, CLI/JSON/MCP,
  and general session/table registry semantics.

Completion is `partial` until implementation, independent review, hosted
acceptance, merge, and post-merge workflow evidence are recorded.
