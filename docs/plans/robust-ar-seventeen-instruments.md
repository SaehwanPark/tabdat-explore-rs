# Robust scalar AR confidence sets through seventeen instruments

## Scope

Extend retained HC1/one-way CR1 scalar Anderson–Rubin confidence sets to q=17
excluded instruments. Preserve the one-endogenous-coefficient contract,
included-exogenous nuisance treatment, retained sample/groups, corrections,
result metadata, output shape and all q<=16 behavior. The q=17 boundary has
degree at most 34, with up to 34 simple real roots and 18 accepted components;
q>=18 and other endogenous/covariance dimensions remain unsupported.

## Verification plan

- Use two deterministic 408-row designs with one included control,
  K_aux=19, residual DF=389 and 24 clusters of seventeen observations.
- Add an independent Base R reference for HC1 and one-way CR1. Cross-check
  degree-34 determinant coefficients at 35 Chebyshev nodes, polyroot roots,
  set endpoints, metadata and direct Wald membership probes.
- Test q=17 HC1/CR1 results, instrument rescaling, retained-fit/runtime state,
  JSON/text output and q=18 rejection before later effects.
- Extend the polynomial boundary test to 34 roots and 18 components; update
  help, result contracts, roadmap, SPEC, coverage evidence and runnable examples
  with q>=18 unsupported.
- Run focused and affected package suites, scoped Clippy, formatting and diff
  checks; regenerate the q=17 reference fixtures byte-for-byte.

## Progress

- [x] Scope and verification plan recorded.
- [x] Open draft PR before substantive implementation (#503).
- [x] Implement q=17 support and independent reference fixtures.
- [x] Update runtime/CLI coverage and specifications/examples.
- [x] Verify reference regeneration and affected checks.
- [ ] Final PR review and merge.

Verification note: the independent R reference and both generated designs match
their checked-in CSVs byte-for-byte. The stats, execution, reporting, runtime
and four affected CLI suites pass. Strict Clippy passes for the affected
packages, root binary and changed CLI targets; formatting and diff checks pass.
