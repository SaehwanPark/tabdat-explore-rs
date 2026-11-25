# Robust scalar AR confidence sets through fifteen instruments

## Scope

Extend retained HC1/one-way CR1 scalar Anderson–Rubin confidence sets to q=15
excluded instruments. Preserve the one-endogenous-coefficient contract,
included-exogenous nuisance treatment, retained sample/groups, corrections,
result metadata, output shape and all q<=14 behavior. The q=15 boundary has
degree at most 30, with up to 30 simple real roots and 16 accepted components;
q>=16 and other endogenous/covariance dimensions remain unsupported.

## Verification plan

- Use two deterministic 360-row designs with one included control,
  K_aux=17, residual DF=343 and 24 clusters of fifteen observations.
- Add an independent Base R reference for HC1 and one-way CR1. Cross-check
  degree-30 determinant coefficients at 31 Chebyshev nodes, `polyroot` roots,
  set endpoints, metadata and direct Wald membership probes.
- Test q=15 HC1/CR1 results, instrument rescaling, retained-fit/runtime state,
  JSON/text output and q=16 rejection before later effects.
- Extend the polynomial boundary test to 30 roots and 16 components; update
  help, result contracts, roadmap, SPEC, coverage evidence and runnable examples
  with q>=16 unsupported.
- Run focused and affected package suites, scoped Clippy, formatting and diff
  checks; regenerate the q=15 reference fixtures byte-for-byte.

## Progress

- [x] Scope and verification plan recorded.
- [x] Open draft PR #501 before substantive implementation.
- [x] Implement q=15 support and independent reference fixtures.
- [x] Update runtime/CLI coverage and specifications/examples.
- [x] Verify reference regeneration and affected checks.
- [ ] Final PR review and merge.
