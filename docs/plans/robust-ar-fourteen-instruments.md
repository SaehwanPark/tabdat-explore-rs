# Robust scalar AR confidence sets through fourteen instruments

## Scope

Extend retained HC1/one-way CR1 scalar Anderson–Rubin confidence sets to
q=14 excluded instruments. Preserve one endogenous coefficient, included
exogenous nuisance treatment, fitted sample/groups, corrections, output shape
and all q<=13 behavior. The q=14 boundary has degree at most 28, with up to 28
simple real roots and 15 accepted components; q>=15 and other
endogenous/covariance dimensions remain unsupported.

## Verification plan

- Use two deterministic 336-row designs with one included control,
  K_aux=16, residual DF=320 and 24 clusters of fourteen observations.
- Add an independent Base R reference for HC1 and one-way CR1. Cross-check
  degree-28 determinant coefficients at 29 Chebyshev nodes, `polyroot` roots,
  set endpoints, metadata and direct Wald membership probes.
- Test q=14 HC1/CR1 results, instrument rescaling, retained-fit/runtime state,
  JSON/text output, and q=15 rejection before later effects.
- Extend the polynomial boundary test to 28 roots and 15 components; update
  help, result contracts, roadmap, SPEC, coverage evidence and runnable examples
  with q>=15 unsupported.
- Run focused and affected package suites, scoped Clippy, formatting and diff
  checks; regenerate q=14 reference fixtures byte-for-byte.

## Progress

- [x] Scope and verification plan recorded.
- [ ] Open draft PR #500 before substantive implementation.
- [ ] Implement q=14 support and independent reference fixtures.
- [ ] Update runtime/CLI coverage and specifications/examples.
- [ ] Verify reference regeneration and affected checks.
- [ ] Final PR review and merge.
