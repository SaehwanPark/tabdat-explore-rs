# Robust scalar AR confidence sets through thirteen instruments

## Scope

Extend the retained HC1/one-way CR1 scalar Anderson–Rubin set from q=12 to
q=13 excluded instruments. Keep one endogenous coefficient and all existing
sample, nuisance, correction, output and lower-dimensional contracts unchanged.
The q=13 boundary has degree at most 26, up to 26 simple real roots and 14
accepted components. q>=14 and other endogenous/covariance dimensions remain
unsupported.

## Verification plan

- Add an independent Base R q=13 reference for two 312-row designs under HC1
  and CR1; use one included control, K_aux=15, residual DF=297, 24 groups of
  thirteen observations and 27 Chebyshev nodes for degree-26 determinants.
- Compare polynomial coefficients, roots, set endpoints, metadata and direct
  Wald membership probes; exercise bounded HC1 designs and empty CR1 sets.
- Test instrument rescaling, retained-fit/runtime state, q=14 rejection,
  JSON/text output and the degree-26/26-root/14-component boundary.
- Update help, command result contracts, roadmap, SPEC, coverage ledger and
  executable examples with the remaining q>=14 boundary explicit.
- Run focused and affected crate suites, scoped Clippy, formatting and diff
  checks; regenerate all independent fixtures byte-for-byte.

## Progress

- [x] Open draft PR #499 before substantive implementation.
- [x] Implement q=13 support, independent Base R fixtures, runtime/CLI coverage,
  and updated specifications and examples.
- [x] Verify the q=13 reference byte-for-byte; focused and affected package
  suites, scoped Clippy, formatting and diff checks pass.
- [ ] Final PR review and merge.
