# Bounded eager-runtime `sort` closeout

Status: accepted and verified on
[`main` at f33987a](https://github.com/SaehwanPark/tabdat-explore-rs/commit/f33987ada50beb4030075a7eb388ec8855b64421);
documentation closeout is complete in the current `main` documentation commit.

## Accepted scope

The runtime executes the parsed eager local-Parquet subset of
`sort <varlist>` for active DuckDB sessions. It validates requested columns
before staging, applies stable ascending native-key ordering with SQL NULLs
last, preserves prior order for complete ties through a private row ordinal,
preserves all published schema/data metadata, and publishes atomically through
`__tabdat_next`. Unknown variables, empty direct variable lists, staging or
inspection failures, and publication failures preserve the previously
published state.

## Durable records

- [`01-contract.md`](01-contract.md) — pinned oracle contract and bounded scope;
- [`02-evidence-migration.md`](02-evidence-migration.md) — oracle, local,
  PR-head, merge-head, and hosted workflow evidence;
- [`03-review.md`](03-review.md) — three-pass review disposition and residual
  deferrals;
- [SPEC verified-slice record](../../SPEC.md); and
- [roadmap Phase 6.3](../../docs/TABDAT_RUST_PORT_ROADMAP.md).

PR [#50](https://github.com/SaehwanPark/tabdat-explore-rs/pull/50) was opened
as a draft at the contract checkpoint, marked ready after local review and
green PR-head workflows, and squash-merged as
[`f33987a`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/f33987ada50beb4030075a7eb388ec8855b64421).
The temporary branch was deleted locally and remotely.

## Hosted verification

PR-head acceptance passed before merge. Merge-head runtime acceptance and the
descendant main CI/feasibility workflows passed; their direct links are
recorded in [`02-evidence-migration.md`](02-evidence-migration.md).

Panel/label metadata, lazy or materialized execution, descending keys,
`gsort`, expression keys, `last_operation`, formatting, CLI/JSON/MCP, and
broad transform sequencing remain deferred.
