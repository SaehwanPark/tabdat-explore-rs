# Robust scalar AR confidence sets through twenty instruments

## Scope

Extend retained-fit robust scalar Anderson–Rubin confidence sets to q=20
excluded instruments for one endogenous regressor under HC1/one-way CR1. Preserve
included-exogenous nuisance treatment, retained sample/groups, covariance
corrections, result metadata, output shape and all q<=19 behavior. The boundary
polynomial has degree at most 40, with up to 40 simple real roots and 21 accepted
components. q>=21 and other endogenous/covariance dimensions remain unsupported.

## Verification plan

- Use two deterministic complete 480-row designs with one included control,
  K_aux=22, residual DF=458 and 24 clusters of twenty observations. Confirm
  exact sample membership and source order in both reference and Rust.
- Add an independent Base R reference for HC1 and one-way CR1. Cross-check
  degree-40 determinant coefficients on 41 Chebyshev nodes against direct
  determinants on a second 41-node set; bracket endpoints with direct Wald
  evaluations and validate metadata and membership probes.
- Test q=20 HC1/CR1 results, instrument rescaling, retained-fit/runtime state,
  JSON/text output and q=21 rejection before later effects. Preserve q<=19
  fixture and behavior checks.
- Extend the polynomial boundary test to 40 roots and 21 components; update
  execution/reporting limits, help, contracts, roadmap, SPEC, coverage evidence
  and runnable examples with q>=21 unsupported.
- Run focused and affected package suites, scoped Clippy, formatting and diff
  checks; regenerate q=20 reference fixtures byte-for-byte.

## Progress

- [x] Scope and verification plan recorded.
- [x] Open draft PR before substantive implementation.
- [ ] Implement q=20 support and independent reference fixtures.
- [ ] Update runtime/CLI coverage and specifications/examples.
- [ ] Verify reference regeneration and affected checks.
- [ ] Final PR review and merge.
