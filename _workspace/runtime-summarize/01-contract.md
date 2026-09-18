# Contract: bounded eager-runtime `summarize`

Status: recovered contract; implementation pending.

Selected skills: `tabdat-migration`, `tabdat-data-semantics`,
`simple-code-writer`

Boundary: pinned Python execution contract → Rust-owned eager-session summaries

## Scope

Execute the existing syntax-only `Command::Summarize { variables }` against the
active dataset held by the bounded eager local-Parquet `tabdat-runtime` session.
A successful request returns owned summary rows in requested order. An empty
variable list selects numeric columns in active schema order. Each row reports
the non-null count, mean, sample standard deviation, minimum, and maximum. The
operation is read-only: active metadata and the private relation remain
unchanged.

This slice builds on the accepted eager local-Parquet `use` boundary in PR #22,
the read-only `describe` boundary in PR #31, cached eager `count` in PR #32,
and owned eager `head`/`tail` previews in PRs #33/#34. It does not add grouped
`by summarize`, lazy loading/materialization, transforms, labels, formatting,
JSON/MCP output, CLI/REPL behavior, or execution for `codebook`, `missing`,
`duplicates`, or `isid`.

The public result owns all values and exposes no DuckDB `Value`/`ValueRef`,
Arrow value, connection, statement, raw pointer, or foreign lifetime. Numeric
minimum/maximum values reuse the existing owned `CellValue` enum so integer and
decimal types remain lossless where DuckDB returns them. Mean and sample
standard deviation are nullable `f64`; count is an owned non-negative integer.

## Python authority

The Python oracle is the clean sibling checkout `../tabdat-explore` at pinned
commit `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, with Python `3.13.3` and the
`uv.lock` digest recorded in `docs/migration/README.md`.

Authoritative paths:

- `src/tabdat/models.py:158-166` (`SummarizeCommand`);
- `src/tabdat/models.py:1171-1183` (`SummaryRow`/`SummarizeResult`);
- `src/tabdat/executor.py:900-903` (active-dataset requirement and read-only
  result); and
- `src/tabdat/backend.py:55-79,816-834,2041-2069` (numeric type selection,
  validation, aggregate semantics, and nullable result values); and
- `tests/test_executor.py:8169-8263,9707-9715` (success, all-missing,
  validation, and no-active cases).

The focused oracle command is:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_executor.py -k \
  'summarize or test_phase_3_inspection_commands_require_active_dataset'
```

The focused probe observed `13 passed, 404 deselected`. The exact no-active
diagnostic is `summarize requires an active dataset; run use <path> first`.

## Rust contract

Add owned summary result types and a typed execution variant:

```rust
pub struct SummaryRow {
  pub variable: String,
  pub count: u64,
  pub mean: Option<f64>,
  pub std_dev: Option<f64>,
  pub minimum: Option<CellValue>,
  pub maximum: Option<CellValue>,
}

pub struct SummarizeResult {
  pub rows: Vec<SummaryRow>,
}

pub enum ExecutionResult {
  Summarize(SummarizeResult),
  // existing variants unchanged
}
```

Numeric data types are matched by their DuckDB base name after trimming
parameters: signed/unsigned integer aliases, `FLOAT`/`REAL`/`FLOAT32`/
`FLOAT64`/`DOUBLE`, and `DECIMAL`. Empty `variables` selects those columns in
schema order. Explicit variables preserve order and duplicates.

| Session state or input | Result | State transition |
| --- | --- | --- |
| no active dataset | `RuntimeError::NoActiveDataset { command: "summarize" }` with exact Python-compatible display text | unchanged; backend remains uninitialized |
| explicit unknown variables | typed summary unknown-variable error, displayed as `summarize unknown variable: <names>` | unchanged |
| explicit nonnumeric variables | typed summary type error, displayed as `summarize requires numeric variables: <names>` | unchanged |
| empty variable list and no numeric columns | typed no-numeric error, displayed as `summarize found no numeric columns` | unchanged |
| valid eager dataset | `ExecutionResult::Summarize` with nullable aggregate fields and owned min/max values | unchanged |
| backend aggregate or supported-value conversion failure | `RuntimeError::SummaryFailed` displayed as `summarize failed for variable: <name>` | active metadata remains exactly as before |

Nulls are ignored by count/aggregates. An all-null numeric column returns
`count = 0` and null mean, sample standard deviation, minimum, and maximum. A
single non-null value has a null sample standard deviation. Lazy/materialized
summary behavior and grouped execution remain deferred.

## Test contract

Add runtime tests for:

- fresh-session exact no-active error and backend non-initialization;
- parser-to-session eager `use` followed by explicit `summarize age cost`;
- empty-variable default selection in schema numeric order;
- exact count/mean/sample-SD/min/max values, including decimal and null
  handling;
- explicit unknown-variable and nonnumeric-variable diagnostics;
- a relation with no numeric columns and an all-null numeric column;
- repeated read stability and unchanged active metadata;
- failed/corrupt active relation or conversion failure preserving metadata; and
- explicit deferral of lazy/materialized and grouped `by summarize` behavior.

Keep existing language parser tests and eager `use`/`describe`/`count`/preview
state tests unchanged. Run the pinned oracle probe, all locked workspace
checks, policy checks, and hosted baseline/runtime workflows.

## Documentation hygiene

After acceptance, correct stale current-state wording that still lists bounded
inspection only through `tail`: `crates/tabdat-language/src/lib.rs`, `SPEC.md`,
ADR 0007, and `_workspace/use-eager-parquet/04-summary.md`. Keep historical
runtime-count/head/tail artifacts unchanged. Create
`_workspace/runtime-summarize/{01-contract,02-evidence-migration,03-review}.md`.
Do not mark the roadmap `summarize` checkbox until merge and post-merge hosted
evidence are green.

## Stop conditions

Stop and record a decision if DuckDB aggregate result ownership or nullable
statistics cannot remain backend-independent, if numeric selection/typing or
sample-SD semantics diverge from the pinned oracle, or if a failed summary
mutates active metadata. Do not expand this slice into grouped summaries,
lazy materialization, transforms, or presentation surfaces.
