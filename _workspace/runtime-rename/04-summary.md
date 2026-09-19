# Bounded eager-runtime `rename` closeout

Status: accepted and verified on `main` at `0bd547f`; documentation closeout
is pending its final main CI run.

## Accepted scope

The runtime executes the parsed eager local-Parquet subset of
`rename <old> <new>` for active DuckDB sessions. It validates the source and
target names before staging, preserves source position, logical type, row
order, row count, SQL NULL values, source path, and eager metadata, and
publishes a quoted source-order projection atomically through `__tabdat_next`.
Unknown sources, target collisions including same-name requests, and staging
or publication failures preserve the previously published state.

## Durable records

- [`01-contract.md`](01-contract.md) — pinned oracle contract and bounded scope;
- [`02-evidence-migration.md`](02-evidence-migration.md) — oracle, local,
  PR-head, merge-head, and hosted workflow evidence;
- [`03-review.md`](03-review.md) — three-pass review disposition and residual
  deferrals;
- [SPEC verified-slice record](../../SPEC.md); and
- [roadmap Phase 6.3](../../docs/TABDAT_RUST_PORT_ROADMAP.md).

PR [#49](https://github.com/SaehwanPark/tabdat-explore-rs/pull/49) was opened
as a draft at the contract checkpoint, marked ready after local review and
green PR-head workflows, and squash-merged as
[`0bd547f`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/0bd547f15db7fef01e2a60554fe52a8eb4a4f129).
The temporary branch was deleted locally and remotely.

## Hosted verification

PR-head CI and runtime workflows passed before merge, and the post-merge
merge-head CI and runtime workflows also passed. Their direct links are
recorded in [`02-evidence-migration.md`](02-evidence-migration.md). The final
documentation-only main CI link will be added after this closeout is pushed.

Panel/label metadata, lazy or materialized execution, wildcard or multi-column
forms, `last_operation`, formatting, CLI/JSON/MCP, and broad transform
sequencing remain deferred.
