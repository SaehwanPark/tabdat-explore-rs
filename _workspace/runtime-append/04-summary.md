# Append syntax slice summary

## Outcome

Accepted parser-only slice. PR [#60](https://github.com/SaehwanPark/tabdat-explore-rs/pull/60)
was merged to `main` as
[`8ab016f1e014445288d2411cf7d058e2bd523b49`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/8ab016f1e014445288d2411cf7d058e2bd523b49).

The language layer now recognizes the bounded form `append <table>` and
returns owned table data with recovered identifier/reserved-name validation and
bounded diagnostics. The runtime continues to return explicit
`UnsupportedCommand { name: "append" }`; no named-table registry, SQL creation,
or backend relation was added.

Contract, migration evidence, and review are recorded in:

- [`01-contract.md`](01-contract.md)
- [`02-evidence-migration.md`](02-evidence-migration.md)
- [`03-review.md`](03-review.md)

The focused oracle parser test, local locked/policy checks, PR-head workflows,
and merge-head workflows all passed. The next dependency for runtime `append`
is named-table state plus SQL/table creation; Phase 6.4 remains unchecked until
that contract is implemented and validated.
