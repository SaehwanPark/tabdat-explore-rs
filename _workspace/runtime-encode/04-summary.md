# Bounded eager-runtime encode closeout

Status: accepted and verified on `main` at merge commit
[`af3e3b2`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/af3e3b2726778af5c5f3b5c13c4ba84e5291da61).

## Accepted scope

The current branch executes the parsed eager local-Parquet subset of
`encode <strvar>, generate(<newvar>)` for active DuckDB sessions. It assigns
1-based codes to sorted unique nonmissing strings, preserves NULLs, appends an
integer target, and uses staged atomic publication. It validates source type,
source existence, target collision, and backend/staging failures without
mutating the prior active state.

The parser also preserves the optional `label(<lblname>)` syntax, but the
encode slice returns an explicit unsupported-label error because general
value-label metadata is outside its contract. Ordinary encode now retains a
private code-to-text map consumed by the separately accepted bounded decode
slice.

## Durable records

- [01-contract.md](01-contract.md) — pinned oracle contract and bounded scope;
- [02-evidence-migration.md](02-evidence-migration.md) — oracle, local, and
  hosted evidence;
- [03-review.md](03-review.md) — correctness, boundary, state, and scope
  review; and
- [roadmap Phase 6.3](../../docs/TABDAT_RUST_PORT_ROADMAP.md).

PR [#53](https://github.com/SaehwanPark/tabdat-explore-rs/pull/53) was opened at
the contract checkpoint, passed its hosted baseline, policy, and Linux runtime
checks, and was squash-merged as
[`af3e3b2`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/af3e3b2726778af5c5f3b5c13c4ba84e5291da61).
The companion evidence record contains the PR-head and subsequent main-branch
workflow links. Documentation closeout commit
[`26962f6`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/26962f66ecb185026b6719a812e437d83ccf4cf0)
also passed the final main CI, ReadStat, libgretl feasibility, and libgretl
OLS workflows recorded in [02-evidence-migration.md](02-evidence-migration.md).

General label metadata, the generic `label` command, lazy/materialized
execution, panel metadata, last-operation state, formatting, CLI/JSON/MCP, and
broad transform sequencing remain deferred. Bounded same-session `decode` is
accepted separately in `_workspace/runtime-decode/`.
