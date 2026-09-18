# Contract: eager-runtime `count`

Status: accepted after PR #32 squash merge `2287fff`; post-merge `main`
verification and final hosted workflow checks are green.

Selected skills: `tabdat-migration`, `tabdat-data-semantics`,
`simple-code-writer`

Boundary: pinned Python execution contract → Rust-owned eager-session row count

## Scope

Execute the existing syntax-only `Command::Count` against the active dataset
held by the bounded eager local-Parquet `tabdat-runtime` session. A successful
request returns an owned `CountResult` from the already-known eager row count.
A fresh session returns the deterministic typed no-active error.

This slice builds only on the accepted eager local-Parquet `use` boundary in
PR #22 and the bounded read-only `describe` boundary in PR #31. It does not
add lazy loading/materialization, transforms, relation APIs, labels, panel
metadata, formatting, JSON/MCP output, CLI/REPL behavior, or execution for
`summarize`, `head`, or `tail`.

## Python authority

The clean sibling oracle is `../tabdat-explore` at pinned commit
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, with Python `3.13.3` and the
`uv.lock` digest recorded in `docs/migration/README.md`.

Authoritative paths:

- `src/tabdat/models.py:216-219` (`CountCommand`);
- `src/tabdat/models.py:1251-1253` (`CountResult`);
- `src/tabdat/executor.py:971-975` (active-dataset requirement, backend count,
  and row-count publication);
- `src/tabdat/backend.py:428-437` (`active_row_count` and `count failed`);
- `tests/test_executor.py:8291-8300` (active eager success); and
- `tests/test_executor.py:9696-9708` (no-active command diagnostics).

The focused oracle command is:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_executor.py -k \
  'test_count_returns_active_dataset_row_count or \
   test_phase_3_inspection_commands_require_active_dataset'
```

Observed result: `7 passed, 410 deselected`. The eager fixture has three rows;
the exact no-active diagnostic is `count requires an active dataset; run use
<path> first`.

## Rust contract

Add the following owned result surface to `tabdat-runtime`:

```rust
pub struct CountResult { pub row_count: u64 }
pub enum ExecutionResult { Load(LoadResult), Describe(DescribeResult), Count(CountResult) }
```

Executing `Command::Count` behaves as follows:

| Session state | Result | State transition |
| --- | --- | --- |
| no active dataset | `RuntimeError::NoActiveDataset { command: "count" }` with exact Python-compatible display text | unchanged; backend remains uninitialized |
| active eager local-Parquet dataset | `ExecutionResult::Count` with the cached `u64` row count | unchanged; eager load already established the known row count |

The Python backend internally executes `SELECT COUNT(*)`, including lazy
materialization paths. The bounded Rust session rejects lazy loads and already
owns the eager row count, so returning that cache is an intentional
implementation choice with the same observable result and no extra backend
operation. Lazy/materialized count behavior remains deferred. The public API
exposes no connection, statement, Arrow value, raw pointer, or foreign
lifetime.

## Test contract

Add runtime tests for:

- fresh-session exact no-active-dataset error and backend non-initialization;
- parser-to-session eager `use` followed by `count`, returning three rows and
  preserving the active dataset metadata;
- repeated `count` stability;
- a failed replacement `use` followed by `count`, preserving the prior
  dataset.

Keep existing language parser tests and eager-`use`/`describe` state tests
unchanged. Run the pinned oracle probe, all locked workspace checks, policy
checks, and hosted baseline/runtime workflows.

## Stop conditions

Stop and record a decision if the eager cached row count is not sufficient for
the pinned success contract or if a failed replacement mutates active
metadata. Do not expand this slice into lazy materialization, transforms, or
preview/value semantics.
