# Selected `estat` syntax slice summary

## Outcome

Accepted parser-only slice. PR [#66](https://github.com/SaehwanPark/tabdat-explore-rs/pull/66)
was merged to `main` as
[484148ee6a25dfe5f245fd2f7be01eec5efd27b3](https://github.com/SaehwanPark/tabdat-explore-rs/commit/484148ee6a25dfe5f245fd2f7be01eec5efd27b3).

The language layer now recognizes the bounded no-option `estat firststage`,
`estat overid`, `estat endogenous`, and `estat hausman` forms and returns an
owned typed diagnostic subcommand. The runtime continues to return explicit
`UnsupportedCommand { name: "estat" }`; no model-state, calculation,
post-estimation, or relation path was added.

Contract, migration evidence, and review are recorded in:

- [01-contract.md](01-contract.md)
- [02-evidence-migration.md](02-evidence-migration.md)
- [03-review.md](03-review.md)

The focused oracle parser test, local locked/policy checks, PR-head workflows,
and merge-head workflows all passed. The next dependency for runtime `estat` is
a typed IV/panel statistical model-state and post-estimation contract; Phase
11.1 diagnostic work remains unchecked until that contract is implemented and
validated.
