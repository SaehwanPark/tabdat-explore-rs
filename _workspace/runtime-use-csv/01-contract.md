# Bounded eager-runtime CSV `use` contract

Status: accepted after PR #74 (`7a5b8d4`); post-merge workflow verification is
recorded in [`02-evidence-migration.md`](02-evidence-migration.md).

Producer: task owner, using `tabdat-migration`, `tabdat-data-semantics`, and
`simple-code-writer`.
Consumer: the bounded eager CSV ingestion implementation and its focused review.

## Scope

Extend the already-parsed `use <path> [, delimiter(...)] [, has_header(...)]`
form to load a local eager `.csv` file through the existing private DuckDB
adapter. Existing eager local-Parquet behavior remains unchanged.

This slice will:

- accept only an existing local `.csv` path (case-insensitive extension) in
  eager mode;
- accept optional `delimiter` and `has_header` values and bind both through
  DuckDB's `read_csv_auto` call;
- create a staged relation, inspect schema and row count, and publish it only
  after all reads succeed;
- return the existing owned `LoadResult` with the CSV source path, ordered
  schema, row count, eager mode, and no lazy engine; and
- preserve the prior active relation, metadata, labels, and backend state when
  validation, CSV reading, schema inspection, row counting, or publication
  fails.

URI, lazy, Feather/Arrow, DTA, remote, `~` expansion, metadata serialization,
atomic temporary-file replacement, CLI/JSON/MCP surfaces, and broad Python
ingestion parity are explicitly deferred.

## Python contract

Pinned authority:

- repository: `https://github.com/SaehwanPark/tabdat-explore.git`;
- revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`;
- tree: `601b236788872323af9277d2276a236154a0f129`;
- package: `0.25.0`; and
- `uv.lock` SHA-256:
  `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.

Relevant authority paths:

- `src/tabdat/backend.py:290-327` — local CSV path, optional delimiter/header
  parameters, and parameterized `read_csv_auto` ingestion;
- `src/tabdat/backend.py:3617-3628` — source-path validation; and
- `tests/test_executor.py:10139-10172` — delimiter/header, typed load result,
  schema, row count, and preview behavior.

The bounded Python behavior is:

1. a local CSV path is accepted in eager mode;
2. optional delimiter and header values are passed as query parameters;
3. quoted fields, empty fields, NULL inference, and insertion order are owned
   by DuckDB's CSV reader;
4. a failed replacement does not discard the prior active relation; and
5. a successful load publishes source-oriented metadata and clears prior
   label metadata.

The Python implementation also supports remote, lazy, Feather/Arrow, DTA, and
broader Polars behavior. Those observations constrain later work but are not
part of this bounded Rust slice.

## Rust contract

Reuse `ExecutionResult::Load` and `LoadResult`. Add typed CSV read, schema, and
row-count errors alongside the existing Parquet errors. The implementation
must validate the local path before initializing a backend, bind the path and
optional CSV options through DuckDB rather than interpolating user text into
SQL, stage into `__tabdat_next`, and publish only after schema and count checks
complete. A failed staged read must leave `__tabdat_active`, published
`DatasetInfo`, and labels unchanged.

## State-transition contract

| Situation | Active relation | Published metadata | Backend initialization |
| --- | --- | --- | --- |
| No active session, invalid path | unchanged | unchanged | must not occur |
| First CSV load fails | absent | absent | may be initialized only after path validation |
| Replacement CSV load fails | unchanged | unchanged | existing backend remains usable |
| Successful CSV load | replaced atomically | source/schema/count published | existing or newly initialized backend |

## Test contract

Focused Rust coverage must include:

- default CSV auto-detection with header and insertion order;
- explicit semicolon delimiter and `has_header(false)` options;
- quoted fields, SQL NULL/empty fields, uppercase `.CSV`, and empty/header-only
  files;
- ordered schema, row count, and owned preview values;
- missing, non-file, unsupported-suffix, and malformed CSV errors; and
- failed replacement preserving prior metadata, labels, and private rows.

The pinned oracle probe is:

```sh
cd ../tabdat-explore
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_executor.py -k 'test_execute_ingestion_csv_feather_arrow'
```

The broader parser/script oracle, focused Rust tests, locked workspace checks,
policy checks, `git diff --check`, PR-head workflows, and merge-head workflows
are required for completion. The Python oracle is behavioral evidence; it is
not a Rust runtime dependency.

## Implementation mapping and deferrals

- Native Rust: format dispatch, typed read errors, and state-preserving session
  publication.
- DuckDB adapter: `read_csv_auto(?)` with bound `delim` and `header` options,
  staged through the existing relation lifecycle.
- Deferred: remote and lazy sources, Feather/Arrow, DTA, `~` expansion, atomic
  temporary-file replacement, metadata/label/panel serialization, CLI/JSON/MCP,
  and general table-registry semantics.

Completion is `partial` until implementation, independent review, hosted
acceptance, merge, and post-merge workflow evidence are recorded.
