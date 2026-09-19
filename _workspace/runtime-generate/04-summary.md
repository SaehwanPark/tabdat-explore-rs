# Bounded eager-runtime `generate` closeout

Status: accepted and verified on `main` at `98979bc`.

Draft PR [#46](https://github.com/SaehwanPark/tabdat-explore-rs/pull/46) was
opened at the contract boundary, reviewed, marked ready, and squash-merged as
[`98979bc`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/98979bce06f6a85932dd51a1b46421e72b542aa7).
The temporary branch was deleted locally and remotely.

## Accepted scope

The runtime now executes the parsed numeric eager subset of
`generate <target> = <expression>` for active local-Parquet DuckDB sessions:
numeric identifiers/literals, unary minus, `+`, `-`, `*`, `/`, and quoted
identifiers/targets. It appends the generated column, preserves schema/row
order, row count, NULLs, source and eager metadata, and publishes through the
shared staged atomic path. Typed validation and backend errors preserve the
previous published state.

## Durable records

- [`01-contract.md`](01-contract.md) — pinned oracle contract and explicit
  bounded scope;
- [`02-evidence-migration.md`](02-evidence-migration.md) — oracle, local,
  PR-head, merge, branch-cleanup, and post-merge evidence;
- [`03-review.md`](03-review.md) — review disposition and residual gaps;
- [SPEC verified-slice record](../../SPEC.md) and [roadmap Phase 6.3](../../docs/TABDAT_RUST_PORT_ROADMAP.md).

Function calls, strings/booleans/NULL/comparisons, exact overflow counts,
non-finite/division normalization parity, lazy/materialized execution,
labels/panel metadata, `last_operation`, CLI/JSON/MCP, and broad transform
sequencing remain deferred.

## Post-merge verification

The merge-head [CI workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35443571579)
and [runtime workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35443571550)
both passed after the squash merge. The next roadmap loop may select the next
bounded transform contract without treating this slice as general CLI/runtime
parity.
