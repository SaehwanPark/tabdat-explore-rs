# Robust scalar AR confidence sets through twenty-two instruments

## Scope

Extend retained-fit robust scalar Anderson–Rubin confidence sets to q=22
excluded instruments for one endogenous regressor under HC1/one-way CR1. Preserve
included-exogenous nuisance treatment, retained sample/groups, corrections,
result metadata, output shape and all q<=21 behavior. The degree-44 boundary
supports up to 44 simple real roots and 23 accepted components; q>=23 and other
endogenous/covariance dimensions remain unsupported.

## Verification plan

- Use two deterministic complete 528-row designs with one included control,
  K_aux=24, residual DF=504 and 24 clusters of twenty-two observations. Confirm
  exact sample membership and order in reference and Rust.
- Add an independent Base R reference for HC1 and one-way CR1. Cross-check
  degree-44 determinant coefficients on 45 Chebyshev nodes against direct
  determinants on a second 45-node set; bracket endpoints with direct Wald
  evaluations and validate metadata and membership probes.
- Test q=22 HC1/CR1 outputs, instrument rescaling, retained-fit/runtime state,
  JSON/text output and q=23 rejection before later effects. Preserve q<=21
  behavior.
- Extend the polynomial boundary test to 44 roots and 23 components; update
  execution/reporting limits, help, contracts, roadmap, SPEC, coverage evidence
  and runnable examples with q>=23 unsupported.
- Measure q=22 calculation RSS while running the focused reference test to keep
  the determinant DP's memory cost visible.
- Run focused and affected package suites, scoped Clippy, formatting and diff
  checks; regenerate q=22 fixtures byte-for-byte.

## Progress

- [x] Scope and verification plan recorded.
- [ ] Open draft PR before substantive implementation.
- [ ] Implement q=22 support and independent reference fixtures.
- [ ] Update runtime/CLI coverage and specifications/examples.
- [ ] Verify reference regeneration and affected checks.
- [ ] Final PR review and merge.
