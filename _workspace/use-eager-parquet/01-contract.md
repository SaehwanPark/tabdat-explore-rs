# Contract: eager local-Parquet `use`

Status: draft; contract recovery for the bounded Phase 4 runtime slice in PR #22.

Selected skills: `tabdat-migration`, `tabdat-data-semantics`,
`tabdat-native-backends`, `simple-code-writer`

Rust base: `main` at `6806cf7` (the accepted `summarize` syntax slice and
post-merge documentation checks with green hosted workflows).

## Scope

Add one library-only execution path for an existing local `.parquet` source:

```text
use <existing-local-parquet>
```

The path is parsed by `tabdat-language::Command::Use`. A Rust-owned session
loads the file eagerly through a private DuckDB adapter, returns owned dataset
metadata, and records that dataset as the active relation. Loading is staged and
validated before replacing the prior active relation so a failed read leaves
the previous dataset and metadata unchanged.

This bounded API accepts an already-resolved local path. Shell-style `~` home
directory expansion remains deferred to the caller; the pinned Python resolver
performs `expanduser()` and that parity gap is recorded rather than hidden.

This slice deliberately does not add a CLI/REPL, JSON/MCP rendering, CSV/DTA/
Feather/Arrow loading, URI/network access, lazy plans, named tables, `describe`
or `count` execution, transformations, statistics, labels, or a general
relation API.

## Python contract

The pinned clean oracle is the sibling checkout `../tabdat-explore` at commit
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`), Python 3.13.3. The checkout is
clean and its `uv.lock` digest is recorded in `docs/migration/README.md`.

Authoritative paths are:

- `src/tabdat/models.py:937-950`: `ColumnInfo`;
- `src/tabdat/models.py:1004-1028`: `DatasetInfo`;
- `src/tabdat/models.py:1119-1126`: `LoadResult`;
- `src/tabdat/executor.py:873-876,1310-1334`: `use` dispatch and session update;
- `src/tabdat/backend.py:165-392`: source loading and eager Parquet staging;
- `src/tabdat/backend.py:394-426`: schema and row-count metadata;
- `src/tabdat/backend.py:3914-3966`: local path expansion, suffix precedence,
  and existence checks;
- `tests/conftest.py:15-34`: the synthetic three-row, four-column fixture;
- `tests/test_executor.py:750-763`: eager local-Parquet success;
- `tests/test_executor.py:1043-1059`: failed Parquet load preserves active state;
- `docs/commands/use.md:1-27`: public source and mode syntax.

The focused oracle check is:

```sh
cd ../tabdat-explore
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_executor.py -k \
  'test_use_loads_active_dataset or test_failing_lazy_use_preserves_existing_active_dataset'
```

The recovered result is `2 passed, 415 deselected`. The successful fixture has
three rows and four columns ordered `age`, `bmi`, `sex`, `cost`, with eager mode
and no lazy engine. A corrupt Parquet load raises
`use could not read Parquet file: <path>` and leaves the existing three-row
active dataset in place.

The broader Python parser/script oracle remains the pinned `516 passed` suite;
it is regression evidence, not proof of Rust runtime parity.

## Rust contract

Add a workspace `tabdat-runtime` library with a safe public boundary:

```rust
pub struct Session { /* active metadata and private DuckDB adapter */ }
pub struct DatasetInfo { /* owned source, ordered columns, row count, mode */ }
pub struct LoadResult { pub dataset: DatasetInfo }
pub enum ExecutionResult { Load(LoadResult) }
```

`Session::new` does not initialize DuckDB. Executing a supported `Command::Use`
request initializes the backend lazily, validates the case-insensitive
`.parquet` suffix before checking that the source is a local existing regular
file, and rejects lazy mode, URI sources, and CSV options with deterministic typed
errors. The public API exposes no DuckDB connection, statement, Arrow value, raw
pointer, or foreign lifetime.

The Rust boundary intentionally does not expand `~`; callers must provide the
resolved path. This is a documented parity deferral from Python's
`Path(...).expanduser()` behavior.

The bounded diagnostics intentionally differ from Python outside the supported
success path: Rust reports one deterministic local-Parquet runtime error for
unsupported suffixes and for lazy/URI/option configurations, while Python's
general resolver reports `use only supports .parquet, .dta, .csv, .feather, and
.arrow files` for a wrong suffix and supports the deferred formats and modes.
These are scope/diagnostic deviations, not claims of whole-`use` parity.

The adapter sets `preserve_insertion_order = true`, creates a staging table from
the bound Parquet path, reads ordered schema and a non-negative row count, then
replaces the internal active table in a transaction. The session updates its
owned metadata only after that operation succeeds. The active metadata reports
`execution_mode: Eager` and `lazy_engine: None`.

## Test contract

Tests will cover:

- parser-to-session eager loading of the synthetic three-row fixture;
- ordered column names/types, row count, eager mode, and no lazy engine;
- lazy/URI/non-Parquet/missing/directory/unsupported-option rejection;
- unsupported-suffix precedence for missing paths and directories;
- corrupt-Parquet failure after a successful load, with prior metadata and
  staged relation preserved;
- a fresh session that has not initialized DuckDB until a supported load is
  attempted; and
- unchanged language/scaffold behavior through the workspace checks.

The runtime tests use a temporary synthetic Parquet fixture generated through
the pinned DuckDB dependency and remove it with a scoped guard. No private data
or committed native artifacts are needed.

## Implementation mapping and deferrals

- Native Rust: `tabdat-runtime::Session`, owned metadata/results/errors, path
  validation, transaction/staging lifecycle, parser-to-session wiring, and
  focused tests.
- DuckDB: `duckdb-rs` `1.10505.0` with `bundled` and `parquet` features, hidden
  behind the runtime adapter; this is the first production-boundary evaluation,
  not a blanket adoption of every spike capability.
- Deferred: `~` expansion, all other `use` formats/modes/sources, relation query APIs,
  schema-cache invalidation, named tables, session status, inspect/transform/
  statistics commands, reporting/serialization, and CLI/MCP surfaces.

Acceptance requires local locked Rust checks, dependency/audit/unsafe scans,
`git diff --check`, focused/full pinned oracle evidence, native ownership and
license review, independent parser/runtime/workspace review, and all hosted
baseline/native workflows. Stop and record a decision if DuckDB ownership,
transactional cleanup, platform builds, or license obligations cannot be
established without widening the slice.
