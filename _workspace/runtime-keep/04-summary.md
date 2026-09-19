# Bounded eager-runtime `keep` closeout

Status: accepted and verified on `main` at `37f0ab6`.

PR [#42](https://github.com/SaehwanPark/tabdat-explore-rs/pull/42) was opened
as a draft before implementation, independently reviewed, marked ready after
all required checks passed, and squash-merged as
[`d43c923`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/d43c923f2b599a8ad7ebe7a3b5b0af44c1f77e00).
The temporary `feat/runtime-keep` branch was removed locally and remotely.

## Accepted scope

The slice adds `keep <explicit-varlist>` projection over the active eager
local-Parquet DuckDB relation. It preserves requested/duplicate column order and
row order, supports quoted identifiers, stages and publishes atomically, and
keeps published metadata unchanged on validation or backend failure.

Predicate-form `keep if <expression>`, expression functions and overflow/filter
semantics, lazy/materialized execution, wildcard/range expansion,
labels/panel metadata, `last_operation`, formatting, CLI/REPL, JSON/MCP, and
broader transformation sequencing remain deferred.

## Durable records

- [`01-contract.md`](01-contract.md) — pinned oracle contract and bounded scope;
- [`02-evidence-migration.md`](02-evidence-migration.md) — local, policy,
  PR-head, merge, and cleanup evidence;
- [`03-review.md`](03-review.md) — independent review and disposition;
- [SPEC verified-slice record](../../SPEC.md) and [roadmap Phase 6.3](../../docs/TABDAT_RUST_PORT_ROADMAP.md);
- [ADR 0007](../../docs/adr/0007-eager-parquet-duckdb-runtime-boundary.md) —
  bounded eager DuckDB boundary.

## Handoff

The next bounded roadmap candidate is `generate`; the inverse projection `drop`
and requested-order projection `select` slices are accepted separately in
[`../runtime-drop/04-summary.md`](../runtime-drop/04-summary.md) and
[`../runtime-select/04-summary.md`](../runtime-select/04-summary.md).
This merge does not establish general predicate transforms, relation sequencing,
or broad Phase 4 completion.

## Post-merge verification

The final main documentation closeout `37f0ab6` passed the complete matrix:

- [CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35424054768), [TabDat runtime boundary](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35424054718);
- [ReadStat feasibility](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35424054733);
- [libgretl feasibility](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35424054778); and
- [libgretl OLS](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35424054698).
