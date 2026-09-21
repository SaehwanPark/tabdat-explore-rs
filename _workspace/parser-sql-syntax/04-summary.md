# Bounded `sql` syntax slice summary

## Outcome

Accepted bounded language-layer `sql` syntax slice. PR
[#77](https://github.com/SaehwanPark/tabdat-explore-rs/pull/77) was merged to
`main` as
[`feeeaf4`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/feeeaf4).

The language layer now exposes an owned `SqlCommand` for direct and
triple-quoted queries, with optional trailing case-insensitive `into <table>`
clauses. The parser validates identifier and reserved-name constraints
(`active`, `__tabdat_*`), preserves query text opaquely, and reports exact
Python-compatible diagnostics. Runtime execution remains an explicit
unsupported-command result.

No backend execution, DuckDB query execution, relation mutation, named-table
lifecycle, multiline script block parsing, reporting, CLI/JSON/MCP surface,
or broad Python database parity was added.

Contract, migration evidence, and review are recorded in:

- [01-contract.md](01-contract.md)
- [02-evidence-migration.md](02-evidence-migration.md)
- [03-review.md](03-review.md)

The pinned oracle, local locked/policy checks, PR-head workflows, squash merge,
and merge-head workflows all passed. The parser-only `sql` boundary dependency
is closed; runtime `sql` execution and named-table persistence remain
unchecked in the roadmap.
