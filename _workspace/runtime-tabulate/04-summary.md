# Bounded eager-runtime tabulate closeout

Status: accepted and verified on main at merge commit
[24405a6](https://github.com/SaehwanPark/tabdat-explore-rs/commit/24405a6878034506e18086d5092d665b4dd25159),
via [PR #56](https://github.com/SaehwanPark/tabdat-explore-rs/pull/56).

## Accepted scope

The Rust runtime now executes the bounded eager local-Parquet forms:

    tabulate <rowvar>
    tabulate <rowvar> <columnvar>
    tabulate <rowvar> [, missing] [, nolabel]
    tabulate <rowvar> <columnvar> [, row] [, col] [, missing] [, nolabel]

One-way results contain category, count, and percentage cells. Two-way results
contain count columns for observed column categories, with optional row and
column percentage columns. SQL NULL dimensions are excluded by default or
included with missing. Session-local attached value labels affect display
unless nolabel is requested. Results are owned and read-only with respect to
the active relation.

The values/stat, if, by, multi-dimensional, named-table, lazy/materialized,
persistence, formatting, CLI, JSON, MCP, and broad Python parity surfaces
remain deferred.

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

PR-head CI/runtime and policy workflows passed for cb463cd. The squash merge
head 24405a6 was then verified by
[main CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35487873526)
and [main runtime](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35487873479).
The documentation-closeout workflows for 7ff7e00 also passed: [main
CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35489029352),
[ReadStat](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35489029340),
[libgretl feasibility](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35489029344),
and [libgretl OLS](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35489029373).
