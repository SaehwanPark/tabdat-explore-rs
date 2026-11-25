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
  degree-38 determinant coefficients at 39 Chebyshev nodes, `polyroot` roots,
  set endpoints, metadata and direct Wald membership probes.
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
- [ ] Implement q=19 support and independent reference fixtures.
- [ ] Update runtime/CLI coverage and specifications/examples.
- [ ] Verify reference regeneration and affected checks.
- [ ] Final PR review and merge.
