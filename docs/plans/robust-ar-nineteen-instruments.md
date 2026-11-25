# Robust scalar AR confidence sets through nineteen instruments

## Scope

Extend retained HC1/one-way CR1 scalar Anderson–Rubin confidence sets to
q=19 excluded instruments. Preserve the one-endogenous-coefficient contract,
included-exogenous nuisance treatment, retained sample/groups, corrections,
result metadata, output shape and all q<=18 behavior. The q=19 boundary has
degree at most 38, with up to 38 simple real roots and 20 accepted components;
q>=20 and other endogenous/covariance dimensions remain unsupported.

## Verification plan

- Use two deterministic complete 456-row designs with one included control,
  K_aux=21, residual DF=435 and 24 clusters of nineteen observations. Confirm
  exact sample membership in both reference and Rust.
- Add an independent Base R reference for HC1 and one-way CR1. Cross-check
  degree-38 determinant coefficients on 39 Chebyshev nodes against direct
  determinants on a second 39-node set; bracket set endpoints with direct Wald
  evaluations and validate metadata and membership probes.
- Test q=19 HC1/CR1 results, instrument rescaling, retained-fit/runtime state,
  JSON/text output and q=20 rejection before later effects.
- Extend the polynomial boundary test to 38 roots and 20 components; update
  help, result contracts, roadmap, SPEC, coverage evidence and runnable examples
  with q>=20 unsupported.
- Run focused and affected package suites, scoped Clippy, formatting and diff
  checks; regenerate the q=19 reference fixtures byte-for-byte.

## Progress

- [x] Scope and verification plan recorded.
- [x] Open draft PR before substantive implementation.
- [x] Implement q=19 support and independent reference fixtures.
- [x] Update runtime/CLI coverage and specifications/examples.
- [x] Verify reference regeneration and affected checks.
- [ ] Final PR review and merge.

Verification note: both generated data files and the Base R reference CSV match
the checked-in fixtures byte-for-byte. Each design has 456 complete ordered rows
and 24 groups of nineteen. The stats suite passes (121 unit tests and all package
integration targets); execution/reporting/runtime suites pass, including q=19
retained-fit state. All 60 affected root CLI tests pass. Strict Clippy passes for
affected packages, the binary and CLI targets; formatting and diff checks pass.
One existing manual source probe remains intentionally ignored.

Numerical note: the synthetic degree-38 polynomial boundary test uses a 5e-7
root-location tolerance because the product coefficients amplify rounding across
38 simple roots. Applied HC1/CR1 endpoints remain checked against direct Base R
Wald inversion within 3e-8.
