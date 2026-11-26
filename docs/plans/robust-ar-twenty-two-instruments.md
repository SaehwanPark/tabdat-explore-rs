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
- Run focused tests across affected packages and integration boundaries, scoped
  Clippy, formatting and diff checks; regenerate q=22 fixtures byte-for-byte.

## Progress

- [x] Scope and verification plan recorded.
- [x] Open draft PR before substantive implementation.
- [x] Implement q=22 support and independent reference fixtures.
- [x] Update runtime/CLI coverage and specifications/examples.
- [x] Verify reference regeneration and affected checks.
- [ ] Final PR review and merge.

## Verification outcomes

- The two 528-row fixtures and Base R reference regenerate byte-for-byte. The
  q=22 reference reports chi-square(22)=33.9244384714438, HC1/CR1 corrections
  1.04761904761905 and 1.09109730848861, and the committed primary/secondary
  HC1 intervals; both CR1 sets are empty. Rust endpoints match within 3e-8.
- Focused stats tests pass for q=22 reference outputs, instrument rescaling and
  q=23 rejection; the q=21 independent-reference regression also passes after
  the in-place DP memory change. The degree-44 synthetic polynomial covers all
  44 roots and 23 components within 5e-6; applied design endpoints retain the
  3e-8 reference tolerance.
- Runtime retained-fit behavior, JSON/text CLI workflow, q=23 failure
  atomicity, help discovery and neighboring scalar CLI output pass. Scoped
  Clippy for affected crates and the root CLI, formatting and diff checks pass.
- Peak RSS was 1,245,282,304 bytes for the four-case q=22 stats reference test,
  1,309,360,128 bytes for runtime retained-fit coverage, and 1,174,749,184 bytes
  for the JSON/text CLI workflow. Those tests take about 18, 18 and 13 minutes,
  respectively, in the unoptimized test profile.
