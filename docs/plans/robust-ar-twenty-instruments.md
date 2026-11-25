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
- [x] Implement q=20 support and independent reference fixtures.
- [x] Update runtime/CLI coverage and specifications/examples.
- [x] Verify reference regeneration and affected checks.
- [ ] Final PR review and merge.

Verification note: both regenerated data files and the Base R reference CSV
match the checked-in fixtures byte-for-byte. Each design has 480 complete
ordered rows and 24 groups of twenty. The stats suite passes (122 unit tests
and all package integration targets; one existing manual source probe remains
intentionally ignored). Execution/reporting/runtime suites pass, including the
q=20 retained-fit test. All 61 affected root CLI/workflow tests pass. Strict
Clippy passes for affected packages, the binary and affected CLI targets;
formatting and diff checks pass.

Numerical note: the synthetic degree-40 boundary test uses a 5e-7 root-location
tolerance because product coefficients amplify rounding across forty simple
roots. Applied HC1/CR1 endpoints match direct Base R Wald inversion within
3e-8.
