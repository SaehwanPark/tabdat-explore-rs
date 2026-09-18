# Contract: bounded eager-runtime `head`

Status: contract frozen; implementation and review are pending in draft PR
`runtime-head`.

Selected skills: `tabdat-migration`, `tabdat-data-semantics`,
`simple-code-writer`

Boundary: pinned Python execution contract → Rust-owned eager-session preview

## Scope

Execute the existing syntax-only `Command::Head { limit }` against the active
dataset held by the bounded eager local-Parquet `tabdat-runtime` session. A
successful request returns an owned `PreviewResult` containing schema-order
column names and owned cell values in source insertion order. The default limit
is five rows, `head 0` returns the columns with no rows, and a limit larger than
the eager dataset returns all available rows. A fresh session returns the
deterministic typed no-active error.

This slice builds on the accepted eager local-Parquet `use` boundary in PR #22,
the read-only `describe` boundary in PR #31, and the cached eager `count`
boundary in PR #32. It does not add `tail`, lazy loading/materialization,
transforms, labels, panel metadata, formatting, JSON/MCP output, CLI/REPL
behavior, or execution for `summarize`, `codebook`, or `missing`.

The public result uses a Rust-owned `CellValue` enum. It exposes no DuckDB
`Value`/`ValueRef`, Arrow value, connection, statement, raw pointer, or foreign
lifetime. The fixture-complete value set is null, boolean, signed/unsigned
integer, floating point, exact decimal `(width, scale, value)`, text, and bytes;
other DuckDB logical/container values remain an explicit `head failed` boundary
until a separate value-conversion contract is recovered.

## Python authority

The clean sibling oracle is `../tabdat-explore` at pinned commit
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, with Python `3.13.3` and the
`uv.lock` digest recorded in `docs/migration/README.md`.

Authoritative paths:

- `src/tabdat/models.py:222-230` (`HeadCommand(limit=5)`);
- `src/tabdat/models.py:1255-1258` (`PreviewResult`);
- `src/tabdat/executor.py:977-983` (active-dataset requirement and read-only
  preview result);
- `src/tabdat/backend.py:1112-1150` (eager insertion-order preview); and
- `tests/test_executor.py:8303-8314,9696-9711` (success and no-active
  diagnostics).

The focused oracle command is:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_executor.py -k \
  'test_head_returns_first_rows or \
   test_phase_3_inspection_commands_require_active_dataset'
```

The focused probe observed `7 passed, 410 deselected`. The exact no-active diagnostic is
`head requires an active dataset; run use <path> first`.

## Rust contract

Add the following owned result/value surfaces to `tabdat-runtime`:

```rust
pub enum CellValue {
    Null,
    Boolean(bool),
    SignedInteger(i128),
    UnsignedInteger(u128),
    Float(f64),
    Decimal { width: u8, scale: u8, value: i128 },
    Text(String),
    Bytes(Vec<u8>),
}

pub struct PreviewResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<CellValue>>,
}

pub enum ExecutionResult {
    Load(LoadResult),
    Describe(DescribeResult),
    Count(CountResult),
    Head(PreviewResult),
}
```

Executing `Command::Head { limit }` behaves as follows:

| Session state | Result | State transition |
| --- | --- | --- |
| no active dataset | `RuntimeError::NoActiveDataset { command: "head" }` with exact Python-compatible display text | unchanged; backend remains uninitialized |
| active eager dataset, limit `0` | `ExecutionResult::Head` with schema-order columns and empty rows | unchanged; no relation query is needed |
| active eager dataset, finite supported limit | `ExecutionResult::Head` with at most `limit` owned rows in insertion order | unchanged |
| active eager dataset, limit larger than row count | all available owned rows | unchanged |
| row-limit conversion, query, or supported-value conversion fails | `RuntimeError::PreviewFailed { command: "head" }`, displayed as `head failed` | active metadata remains exactly as before |

`RowLimit` remains an arbitrary canonical decimal at the language boundary. The
runtime accepts values through `i64::MAX`; larger values fail deterministically
as `head failed`, matching the pinned Python execution boundary without a
silent narrowing conversion. Lazy/materialized preview behavior remains
deferred.

## Test contract

Add runtime tests for:

- fresh-session exact no-active-dataset error and backend non-initialization;
- parser-to-session eager `use` followed by `head 2`, returning ordered
  `age`, `bmi`, `sex`, `cost` columns and the first two rows;
- default `head` returning all three fixture rows;
- `head 0` preserving columns and returning no rows;
- an oversized finite limit returning all rows;
- null and exact-decimal cell conversion;
- repeated `head` stability and unchanged active metadata;
- a failed/corrupt active relation returning `head failed` without mutating
  active metadata; and
- a parsed limit above `i64::MAX` failing deterministically at execution.

Keep existing language parser tests and eager `use`/`describe`/`count` state
tests unchanged. Run the pinned oracle probe, all locked workspace checks,
policy checks, and hosted baseline/runtime workflows.

## Documentation hygiene

Correct stale current-state wording that still says all count execution is
deferred: `SPEC.md`, ADR 0007, `_workspace/use-eager-parquet/04-summary.md`,
and the `Command::Count` language comment. Keep historical parser-slice
artifacts unchanged. Do not mark the roadmap `head` checkbox until this slice
is merged and its post-merge evidence is green.

## Stop conditions

Stop and record a decision if the fixture's decimal/null values cannot be
converted into owned Rust values without exposing backend types, if insertion
order is not stable, or if a failed preview mutates active metadata. Do not
expand this slice into `tail`, lazy materialization, transforms, or presentation
surfaces.
