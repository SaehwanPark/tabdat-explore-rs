# Bounded eager-runtime `replace` closeout

Status: accepted and verified on `main` at `df2cad9`; documentation closeout
is complete at `6821e1c`.

## Accepted scope

The runtime executes the parsed eager local-Parquet subset of
`replace <target> = <expression> [if <condition>]` for active DuckDB sessions.
It supports numeric and string domain-preserving assignments, explicit NULL
replacement, typed boolean/missing predicates, quoted identifiers, and the
existing checked numeric expression compiler. It stages a source-order
projection, preserves schema position, row count, row order, NULL behavior,
source and eager metadata, and publishes atomically before changing session
metadata. Validation and backend failures preserve the prior published state.

## Durable records

- [`01-contract.md`](01-contract.md) — pinned oracle contract and bounded scope;
- [`02-evidence-migration.md`](02-evidence-migration.md) — oracle, local,
  PR-head, merge, branch-cleanup, and post-merge workflow evidence;
- [`03-review.md`](03-review.md) — three-pass review disposition and residual
  deferrals;
- [SPEC verified-slice record](../../SPEC.md); and
- [roadmap Phase 6.3](../../docs/TABDAT_RUST_PORT_ROADMAP.md).

PR [#48](https://github.com/SaehwanPark/tabdat-explore-rs/pull/48) was opened
as a draft at the contract checkpoint, marked ready after local review and
green PR-head workflows, and squash-merged as
[`df2cad9`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/df2cad9e8f61479ad67f118441acbbeb0704c408).
The temporary branch was deleted locally and remotely.

## Post-merge verification

The merge-head runtime workflow passed, and both the follow-on main CI and the
final docs-only CI for the closeout passed. No implementation files or runtime
behavior changed in the documentation closeout.

Function calls, unsupported boolean/other target domains, exact overflow-count
diagnostics, lazy/materialized execution, labels/panel metadata,
`last_operation`, CLI/JSON/MCP, and broad transform sequencing remain deferred.
