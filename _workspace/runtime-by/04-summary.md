# Bounded eager-runtime by closeout

Status: accepted and verified on main at merge commit
[cc818e5](https://github.com/SaehwanPark/tabdat-explore-rs/commit/cc818e5e5e395d9b0f8c86a0dee451417ca98002),
via [PR #58](https://github.com/SaehwanPark/tabdat-explore-rs/pull/58).

## Accepted scope

The Rust runtime now executes the bounded eager local-Parquet forms:

    by <group> [<group> ...]: summarize [<variable> ...]
    by <group> [<group> ...]: count

Grouped summarize returns an owned ByResult with group columns followed by
mean-variable columns. An omitted summarize variable list selects numeric
non-group columns in schema order; explicit unknown or nonnumeric variables are
rejected. Grouped count returns group columns followed by Count and includes
rows whose group values are SQL NULL. Both forms order groups ascending with
NULL values last and leave the active session state unchanged.

The parser retains typed child commands and rejects nested by, help, status, and
doctor children. Grouped tabulate and all other child execution, conditions,
weights, named tables, lazy/materialized modes, panel propagation, persistence,
formatting, CLI, JSON, MCP, and broad Python by parity remain deferred.

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

PR-head CI/runtime and policy workflows passed for f58e652. The squash merge
head cc818e5 then passed main CI
[35497533142](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35497533142)
and main runtime
[35497533149](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35497533149).
The documentation-closeout commit
[664be28](https://github.com/SaehwanPark/tabdat-explore-rs/commit/664be28eec6f6bbc456119cd3f6fcb6e90b0b9ec)
then passed [main CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35498489328),
[ReadStat feasibility](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35498489360),
[libgretl feasibility](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35498489297),
and [libgretl OLS Rust spike](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35498489337).
