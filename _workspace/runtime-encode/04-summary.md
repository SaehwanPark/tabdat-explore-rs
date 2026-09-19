# Bounded eager-runtime encode closeout

Status: implementation checkpoint; merge closeout pending.

## Accepted scope

The current branch executes the parsed eager local-Parquet subset of
`encode <strvar>, generate(<newvar>)` for active DuckDB sessions. It assigns
1-based codes to sorted unique nonmissing strings, preserves NULLs, appends an
integer target, and uses staged atomic publication. It validates source type,
source existence, target collision, and backend/staging failures without
mutating the prior active state.

The parser also preserves the optional `label(<lblname>)` syntax, but the
runtime returns an explicit unsupported-label error because value-label metadata
does not yet exist in the Rust session model.

## Durable records

- [01-contract.md](01-contract.md) — pinned oracle contract and bounded scope;
- [02-evidence-migration.md](02-evidence-migration.md) — oracle, local, and
  hosted evidence;
- [03-review.md](03-review.md) — correctness, boundary, state, and scope
  review; and
- [roadmap Phase 6.3](../../docs/TABDAT_RUST_PORT_ROADMAP.md).

Draft PR [#53](https://github.com/SaehwanPark/tabdat-explore-rs/pull/53) was
opened at the contract checkpoint. It will be marked ready only after hosted
baseline, policy, and Linux runtime checks pass; merge and main-branch docs
closeout remain pending.

Decode, label metadata, lazy/materialized execution, panel metadata,
last-operation state, formatting, CLI/JSON/MCP, and broad transform sequencing
remain deferred.
