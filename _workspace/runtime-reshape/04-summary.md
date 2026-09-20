# Reshape syntax slice summary

## Outcome

Accepted parser-only slice. PR [#61](https://github.com/SaehwanPark/tabdat-explore-rs/pull/61)
was merged to `main` as
[`e6cc4f1768c9b55b8ead702a08a36283f2a27bee`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/e6cc4f1768c9b55b8ead702a08a36283f2a27bee).

The language layer now recognizes the bounded form
`reshape long|wide <varlist>, i(<id_vars>) j(<name>)` and returns owned
direction, variable, identifier, and j-name data with recovered diagnostics.
The runtime continues to return explicit
`UnsupportedCommand { name: "reshape" }`; no relation reshape SQL, registry,
or publication path was added.

Contract, migration evidence, and review are recorded in:

- [`01-contract.md`](01-contract.md)
- [`02-evidence-migration.md`](02-evidence-migration.md)
- [`03-review.md`](03-review.md)

The focused oracle parser test, local locked/policy checks, PR-head workflows,
and merge-head workflows all passed. The next dependency for runtime `reshape`
is relation/session semantics and atomic publication; Phase 6.4 remains
unchecked until that contract is implemented and validated.
