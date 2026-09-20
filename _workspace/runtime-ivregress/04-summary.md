# `ivregress` syntax slice summary

## Outcome

Accepted parser-only slice. PR [#64](https://github.com/SaehwanPark/tabdat-explore-rs/pull/64)
was merged to `main` as
[d35555111800b4655d30d1bc1448da8ba41d2f21](https://github.com/SaehwanPark/tabdat-explore-rs/commit/d35555111800b4655d30d1bc1448da8ba41d2f21).

The language layer now recognizes the bounded 2SLS/GMM `ivregress` form and
returns an owned typed estimator command with ordered exogenous and instrument
variables, one endogenous variable, and robust/cluster/noconstant options. The
runtime continues to return explicit `UnsupportedCommand { name: "ivregress" }`;
no estimation, covariance, model-state, post-estimation, or relation path was
added.

Contract, migration evidence, and review are recorded in:

- [01-contract.md](01-contract.md)
- [02-evidence-migration.md](02-evidence-migration.md)
- [03-review.md](03-review.md)

The focused oracle parser test, local locked/policy checks, PR-head workflows,
and merge-head workflows all passed. The next dependency for runtime
`ivregress` is a typed statistical model-state and active-relation contract;
Phase 11.1 estimator work remains unchecked until that contract is implemented
and validated.
