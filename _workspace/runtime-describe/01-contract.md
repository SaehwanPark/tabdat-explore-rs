# Contract: eager-runtime `describe`

Status: contract frozen; implementation and review are pending in draft PR
`runtime-describe`.

Selected skills: `tabdat-migration`, `tabdat-data-semantics`,
`simple-code-writer`

Boundary: pinned Python execution contract → Rust-owned read-only runtime result

## Scope

Execute the existing syntax-only `Command::Describe` against the active dataset
held by the bounded eager local-Parquet `tabdat-runtime` session. A successful
request returns an owned `DescribeResult` containing the cached `DatasetInfo`
unchanged. A fresh session returns a deterministic typed no-active-dataset
error. The command does not initialize DuckDB, issue a query, materialize a
plan, or mutate session state.

This slice builds only on the accepted eager local-Parquet `use` boundary in
PR #22. It does not add schema discovery, lazy loading, relation/query APIs,
label or panel metadata, formatting, JSON/MCP output, CLI/REPL behavior, or
execution for `status`, `count`, `head`, or `tail`.

## Python authority

The clean sibling oracle is `../tabdat-explore` at pinned commit
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, with Python `3.13.3` and the
`uv.lock` digest recorded in `docs/migration/README.md`.

Authoritative paths:

- `src/tabdat/models.py:137-140` (`DescribeCommand`);
- `src/tabdat/models.py:1004-1028` (`DatasetInfo`);
- `src/tabdat/models.py:1130-1133` (`DescribeResult`);
- `src/tabdat/executor.py:896-898` (execution dispatch);
- `src/tabdat/executor.py:6505-6508` (active-dataset requirement);
- `tests/test_executor.py:8112-8131` (no-active and active success cases); and
- `docs/commands/describe.md:5-9` (public syntax).

The focused oracle command is:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_executor.py -k \
  'test_describe_requires_active_dataset or test_describe_returns_active_dataset'
```

Observed result: `2 passed, 415 deselected`. The successful fixture has three
rows and ordered columns beginning with `age`; the result returns the same
active dataset metadata rather than a newly queried schema.

## Rust contract

Add the following owned result and error surface to `tabdat-runtime`:

```rust
pub struct DescribeResult { pub dataset: DatasetInfo }
pub enum ExecutionResult { Load(LoadResult), Describe(DescribeResult) }
```

Executing `Command::Describe` behaves as follows:

| Session state | Result | State transition |
| --- | --- | --- |
| no active dataset | `RuntimeError::NoActiveDataset` with `describe requires an active dataset; run use <path> first` | unchanged; backend remains uninitialized |
| active eager local-Parquet dataset | `ExecutionResult::Describe` with an equal owned clone of `DatasetInfo` | unchanged; no DuckDB query or metadata invalidation |
| failed later `use` replacement | same active dataset as before the failure | unchanged; failed staging does not affect `describe` |

The public API exposes no DuckDB connection, statement, Arrow value, raw pointer,
or foreign lifetime. `DatasetInfo` fields remain the existing owned source,
row count, ordered columns, execution mode, and optional lazy engine. Python
label/panel metadata and lazy/materialization behavior remain explicit
deferrals because they are outside the existing Rust session contract.

## Test contract

Add runtime tests for:

- fresh-session exact no-active-dataset error;
- parser-to-session `use` followed by `describe` with ordered metadata and
  eager mode preserved;
- repeated `describe` stability and unchanged `active_dataset()`; and
- a corrupt replacement load failing before `describe`, with the previous
  active dataset still returned.

Keep the existing language parser tests and eager-`use` state/atomicity tests
unchanged. Run the pinned oracle probe, all locked workspace checks, policy
checks, and hosted baseline/runtime workflows.

## Stop conditions

Stop and record a decision if the cached `DatasetInfo` is not sufficient to
match the pinned success/error contract, if describing requires a backend query
or materialization, or if error/state behavior cannot remain atomic. Do not
expand this slice into general inspection or session-status semantics.
