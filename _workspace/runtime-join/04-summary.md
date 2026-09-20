# Join syntax slice summary

## Outcome

Accepted parser-only slice. PR [#59](https://github.com/SaehwanPark/tabdat-explore-rs/pull/59)
was merged to `main` as
[`585c53fcdce135456abded9b27df75fbcbcda8cf`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/585c53fcdce135456abded9b27df75fbcbcda8cf).

The language layer now recognizes the bounded form
`join <table> on <keylist> [, how=inner|left suffix(_right)]` and returns owned
table/key/mode/suffix data with recovered defaults and bounded diagnostics.
The runtime continues to return explicit `UnsupportedCommand { name: "join" }`;
no named-table registry, SQL creation, or backend relation was added.

Contract, migration evidence, and review are recorded in:

- [`01-contract.md`](01-contract.md)
- [`02-evidence-migration.md`](02-evidence-migration.md)
- [`03-review.md`](03-review.md)

The focused oracle parser test, local locked/policy checks, PR-head workflows,
and merge-head workflows all passed. The next dependency for runtime `join` is
named-table state plus SQL/table creation; Phase 6.4 remains unchecked until
that contract is implemented and validated.
