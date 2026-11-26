# Robust scalar AR confidence sets through twenty-one instruments

## Scope

Extend retained-fit robust scalar Anderson–Rubin confidence sets to q=21
excluded instruments for one endogenous regressor under HC1/one-way CR1. Preserve
included-exogenous nuisance treatment, retained sample/groups, corrections,
result metadata, output shape and all q<=20 behavior. The degree-42 boundary
supports up to 42 simple real roots and 22 accepted components; q>=22 and other
endogenous/covariance dimensions remain unsupported.

## Verification plan

- Use two deterministic complete 504-row designs with one included control,
  K_aux=23, residual DF=481 and 24 clusters of twenty-one observations. Confirm
  exact sample membership and order in reference and Rust.
- Add an independent Base R reference for HC1 and one-way CR1. Cross-check
  degree-42 determinant coefficients on 43 Chebyshev nodes against direct
  determinants on a second 43-node set; bracket endpoints with direct Wald
  evaluations and validate metadata and membership probes.
- Test q=21 HC1/CR1 outputs, instrument rescaling, retained-fit/runtime state,
  JSON/text output and q=22 rejection before later effects. Preserve q<=20
  behavior.
- Extend the polynomial boundary test to 42 roots and 22 components; update
  execution/reporting limits, help, contracts, roadmap, SPEC, coverage evidence
  and runnable examples with q>=22 unsupported.
- Keep only adjacent subset-DP layers live while computing the polynomial
  determinant. One pre-change q=21 runtime test reached about 1.0 GB RSS;
  post-change q=21 stats-test samples stayed below about 0.75 GB.
- Run focused and affected package suites, scoped Clippy, formatting and diff
  checks; regenerate q=21 fixtures byte-for-byte.

## Progress

- [x] Scope and verification plan recorded.
- [x] Open draft PR before substantive implementation.
- [x] Implement q=21 support and independent reference fixtures.
- [x] Update runtime/CLI coverage and specifications/examples.
- [x] Verify reference regeneration and affected checks.
- [ ] Final PR review and merge.
