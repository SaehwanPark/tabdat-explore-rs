# Contract: bounded eager-runtime `tail`

Status: recovered contract; implementation pending.

Selected skills: `tabdat-migration`, `tabdat-data-semantics`,
`simple-code-writer`

Boundary: pinned Python execution contract → Rust-owned eager-session preview

## Scope

Execute the existing syntax-only `Command::Tail { limit }` against the active
dataset held by the bounded eager local-Parquet `tabdat-runtime` session. A
successful request returns an owned `PreviewResult` containing schema-order
column names and owned cell values for the final rows of the active relation,
restored to their original insertion order. The default limit is five rows,
`tail 0` returns the columns with no rows, and a limit larger than the eager
dataset returns all available rows. A fresh session returns the deterministic
typed no-active error.

This slice builds on the accepted eager local-Parquet `use` boundary in PR #22,
the read-only `describe` boundary in PR #31, the cached eager `count` boundary
in PR #32, and the bounded eager `head` boundary in PR #33. It does not add
lazy loading/materialization, transforms, labels, panel metadata, formatting,
JSON/MCP output, CLI/REPL behavior, or execution for `summarize`, `codebook`,
or `missing`.

The public result reuses the Rust-owned `CellValue` and `PreviewResult` types.
It exposes no DuckDB `Value`/`ValueRef`, Arrow value, connection, statement,
raw pointer, or foreign lifetime. The fixture-complete value set is null,
boolean, signed/unsigned integer, floating point, exact decimal `(width, scale,
value)`, text, and bytes; other DuckDB logical/container values remain an
explicit `tail failed` boundary until a separate value-conversion contract is
recovered.

## Python authority

The clean sibling oracle is `../tabdat-explore` at pinned commit
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, with Python `3.13.3` and the
`uv.lock` digest recorded in `docs/migration/README.md`.

Authoritative paths:

- `src/tabdat/models.py:232-234` (`TailCommand(limit=5)`);
- `src/tabdat/models.py:1255-1258` (`PreviewResult`);
- `src/tabdat/executor.py:985-992` (active-dataset requirement and read-only
  preview result); and
- `src/tabdat/backend.py:1112-1150` (ordered bounded preview, including the
  tail selection/reversal); and
- `tests/test_executor.py` tail success, ordering, and no-active cases.

The focused oracle command is:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_executor.py -k \
  'test_tail_returns_last_rows or \
   test_active_row_order_is_consistent_for_previews_and_filters or \
   test_phase_3_inspection_commands_require_active_dataset'
```

The focused probe observed `10 passed, 407 deselected`. The exact no-active
diagnostic is `tail requires an active dataset; run use <path> first`.

## Rust contract

Add `ExecutionResult::Tail(PreviewResult)` and execute
`Command::Tail { limit }` as follows:

| Session state | Result | State transition |
| --- | --- | --- |
| no active dataset | `RuntimeError::NoActiveDataset { command: "tail" }` with exact Python-compatible display text | unchanged; backend remains uninitialized |
| active eager dataset, limit `0` | `ExecutionResult::Tail` with schema-order columns and empty rows | unchanged; no relation query is needed |
| active eager dataset, finite supported limit | `ExecutionResult::Tail` with at most `limit` owned final rows in original insertion order | unchanged |
| active eager dataset, limit larger than row count | all available owned rows in insertion order | unchanged |
| row-limit conversion, query, or supported-value conversion fails | `RuntimeError::PreviewFailed { command: "tail" }`, displayed as `tail failed` | active metadata remains exactly as before |

`RowLimit` remains an arbitrary canonical decimal at the language boundary. The
runtime accepts values through `i64::MAX`; larger values fail deterministically
as `tail failed`, matching the pinned Python execution boundary without a
silent narrowing conversion. Lazy/materialized preview behavior remains
deferred.

## Test contract

Add runtime tests for:

- fresh-session exact no-active-dataset error and backend non-initialization;
- parser-to-session eager `use` followed by `tail 2`, returning ordered
  `age`, `bmi`, `sex`, `cost` columns and the final two rows;
- default `tail` returning all three fixture rows;
- `tail 0` preserving columns and returning no rows;
- an oversized finite limit and `i64::MAX` returning all rows;
- null and exact-decimal cell conversion;
- a nontrivial source insertion order restored after tail selection;
- repeated `tail` stability and unchanged active metadata;
- a failed/corrupt active relation returning `tail failed` without mutating
  active metadata; and
- a parsed limit above `i64::MAX` failing deterministically at execution.

Keep existing language parser tests and eager `use`/`describe`/`count`/`head`
state tests unchanged. Run the pinned oracle probe, all locked workspace
checks, policy checks, and hosted baseline/runtime workflows.

## Documentation hygiene

After acceptance, correct stale current-state wording that still lists only
`describe`/`count`/`head` as bounded inspection execution: `SPEC.md`, ADR 0007,
`_workspace/use-eager-parquet/04-summary.md`, and the `Command::Tail` language
comment. Keep historical parser-slice and runtime-head artifacts unchanged.
Do not mark the roadmap `tail` checkbox until this slice is merged and its
post-merge evidence is green.

## Stop conditions

Stop and record a decision if the fixture's decimal/null values cannot be
converted into owned Rust values without exposing backend types, if source
insertion order cannot be restored after bounded tail selection, or if a
failed preview mutates active metadata. Do not expand this slice into lazy
materialization, transforms, or presentation surfaces.
