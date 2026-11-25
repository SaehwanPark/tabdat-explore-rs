# Robust scalar AR confidence sets with eleven instruments

## Scope

Extend the retained 2SLS HC1/one-way CR1 scalar Anderson–Rubin confidence-set
method from at most ten to at most eleven excluded instruments. Keep one
endogenous regressor, unrestricted included-exogenous nuisance terms, current
covariance corrections, and every existing lower-dimensional result unchanged.
The q=11 boundary has degree at most 22; it may have up to 12 accepted
components. Higher instrument counts and other endogenous/covariance dimensions
remain unsupported.

## Verification plan

- Build an independent Base R q=11 reference with degree-22 boundaries and
  several real roots/components under HC1 and CR1.
- Compare retained sample, auxiliary design and corrections, critical value,
  root endpoints, set shape and direct AR membership probes.
- Cover instrument rescaling, malformed designs, failure-state preservation,
  runtime/CLI result metadata and q=12 rejection.
- Update help, examples, command-result documentation, roadmap and IV coverage.
- Run focused stats/runtime/reporting contracts, affected crate suites, Clippy,
  format check and diff check.
