# Bounded eager-runtime collapse closeout

Status: accepted and verified on main at merge commit
[94391af](https://github.com/SaehwanPark/tabdat-explore-rs/commit/94391afe84ab1b1c8c3c5006a55911426d6767b1),
via [PR #57](https://github.com/SaehwanPark/tabdat-explore-rs/pull/57).

## Accepted scope

The Rust runtime now executes the bounded eager local-Parquet form:

    collapse <statistic> <variable> [<variable> ...], by(<group> [<group> ...])

with `count`, `mean`, `sum`, `min`, and `max`. Group columns precede named
aggregate columns, SQL NULL groups are explicit and NULL-last ordered,
`count(variable)` ignores NULL values, and non-count statistics require numeric
aggregate variables. Successful execution atomically replaces the active
relation and returns owned `CollapseResult` metadata while preserving source
and eager execution metadata. Variable labels and value-label attachments are
pruned to surviving group columns.

The condition, weights, named-table, lazy/materialized, panel, persistence,
formatting, CLI, JSON, MCP, and broad Python parity surfaces remain deferred.

## Durable records

- [01-contract.md](01-contract.md) — pinned oracle contract and bounded scope;
- [02-evidence-migration.md](02-evidence-migration.md) — oracle, local,
  PR-head, and merge-head evidence;
- [03-review.md](03-review.md) — correctness, state, security, and scope
  review;
- parser and runtime contract tests in the language and runtime crates;
- [roadmap Phase 6.4](../../docs/TABDAT_RUST_PORT_ROADMAP.md); and
- [SPEC.md](../../SPEC.md) — current accepted behavior.

## Hosted acceptance

PR-head CI/runtime and policy workflows passed for e7f2540. The squash merge
head 94391af was then verified by [main CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35492943464)
and [main runtime](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35492943539).
Documentation-closeout workflow links will be added after the closeout commit
is accepted by its hosted checks.
