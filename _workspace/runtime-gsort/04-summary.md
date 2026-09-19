# Bounded eager-runtime `gsort` closeout

Status: accepted and verified on
[`main` at c06ed5a](https://github.com/SaehwanPark/tabdat-explore-rs/commit/c06ed5a7d913aaa5a4ddbdf58f79149a8b81a79e);
documentation closeout is complete in the current `main` documentation commit.

## Accepted scope

The runtime executes the parsed eager local-Parquet subset of
`gsort [+|-]varlist` for active DuckDB sessions. It validates requested keys
before staging, applies stable per-key native ordering with explicit ascending
or descending directions and SQL NULLs last, preserves prior order for
complete ties through a private row ordinal, preserves all published
schema/data metadata, and publishes atomically through `__tabdat_next`.
Unknown variables, empty direct key lists, staging or inspection failures, and
publication failures preserve the previously published state.

## Durable records

- [`01-contract.md`](01-contract.md) — pinned oracle contract and bounded scope;
- [`02-evidence-migration.md`](02-evidence-migration.md) — oracle, local,
  PR-head, merge-head, and hosted workflow evidence;
- [`03-review.md`](03-review.md) — three-pass review disposition and residual
  deferrals;
- [SPEC verified-slice record](../../SPEC.md); and
- [roadmap Phase 6.3](../../docs/TABDAT_RUST_PORT_ROADMAP.md).

PR [#51](https://github.com/SaehwanPark/tabdat-explore-rs/pull/51) was opened
as a draft at the contract checkpoint, marked ready after local review and
green PR-head workflows, and squash-merged as
[`c06ed5a`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/c06ed5a7d913aaa5a4ddbdf58f79149a8b81a79e).
The temporary branch was deleted locally and remotely.

## Hosted verification

PR-head CI/runtime acceptance and merge-head CI/runtime acceptance passed.
Their direct links are recorded in
[`02-evidence-migration.md`](02-evidence-migration.md), alongside the sort
merge-head and descendant main closeout evidence.

Panel/label metadata, lazy or materialized execution, `last_operation`,
formatting, CLI/JSON/MCP, and broad transform sequencing remain deferred.
