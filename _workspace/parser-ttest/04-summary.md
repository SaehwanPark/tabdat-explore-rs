# bounded `ttest` slice summary

## Outcome

Accepted bounded language-layer `ttest` syntax slice. PR
[#69](https://github.com/SaehwanPark/tabdat-explore-rs/pull/69) was merged to
`main` as
[`4dcd892`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/4dcd892f59422774e49742019e5f4eefb459711b).

The language layer now exposes an owned `TtestCommand` for value comparisons,
paired-variable comparisons, and `by()` grouped comparisons with normalized
Welch/unequal flags. The parser preserves the pinned diagnostics and decoded
quoted names. Runtime execution remains an explicit unsupported-command result.

No statistical estimator, p-value, covariance, relation inspection, session
mutation, backend initialization, report, CLI/JSON/MCP surface, or broad
Python statistical parity was added.

Contract, migration evidence, and review are recorded in:

- [01-contract.md](01-contract.md)
- [02-evidence-migration.md](02-evidence-migration.md)
- [03-review.md](03-review.md)

The pinned oracle, local locked/policy checks, PR-head workflows, squash merge,
and merge-head workflows all passed. The parser-only `ttest` dependency is
closed; statistical `ttest` runtime work remains unchecked in the roadmap.
