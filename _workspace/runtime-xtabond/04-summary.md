# `xtabond` syntax slice summary

## Outcome

Accepted parser-only slice. PR [#67](https://github.com/SaehwanPark/tabdat-explore-rs/pull/67)
was merged to `main` as
[820376d](https://github.com/SaehwanPark/tabdat-explore-rs/commit/820376d6c7ce3c4a4ff8abf24fe9a9b3ad84cea6).

The language layer now recognizes the bounded `xtabond <y> [xvars]` form with
robust and integer lag options and returns an owned typed dynamic-panel command.
The runtime continues to return explicit
`UnsupportedCommand { name: "xtabond" }`; no panel metadata lookup,
instrument construction, estimation, covariance, model-state, post-estimation,
or relation path was added.

Contract, migration evidence, and review are recorded in:

- [01-contract.md](01-contract.md)
- [02-evidence-migration.md](02-evidence-migration.md)
- [03-review.md](03-review.md)

The focused oracle parser test, local locked/policy checks, PR-head workflows,
and merge-head workflows all passed. The next dependency for runtime `xtabond`
is a typed panel metadata and statistical model-state contract; Phase 11.1
dynamic-panel work remains unchecked until that contract is implemented and
validated.
