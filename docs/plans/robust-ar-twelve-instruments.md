# Robust scalar AR confidence sets through twelve instruments

## Scope

Extend the retained HC1/one-way CR1 scalar Anderson–Rubin set from q=11 to
q=12 excluded instruments. Keep one endogenous coefficient and all existing
sample, nuisance, correction, output and lower-dimensional contracts unchanged.
The q=12 boundary has degree at most 24, up to 24 simple real roots and 13
accepted components. q>=13 and other endogenous/covariance dimensions remain
unsupported.

## Verification plan

- Add an independent Base R q=12 reference for two designs under HC1 and CR1;
  cross-check degree-24 polynomial coefficients at 25 Chebyshev nodes, roots,
  set endpoints, metadata and direct Wald membership probes.
- Test instrument rescaling, retained-fit/runtime state, q=13 rejection,
  JSON/text output and the degree-24/24-root/13-component boundary.
- Update help, command result contracts, roadmap, SPEC, coverage ledger and
  executable examples with the remaining q>=13 boundary explicit.
- Run focused and affected crate suites, scoped Clippy, formatting and diff
  checks; regenerate all independent fixtures byte-for-byte.

## Progress

- [x] Scope and verification plan recorded.
- [x] q=12 implementation, independent Base R fixtures, runtime/CLI coverage and docs.
- [x] Final verification: affected crate suites, CLI contracts, Clippy, formatting, diff checks and byte-for-byte R fixture regeneration.
- [x] PR #498 reviewed and merged to `main` as `f2932fc`.
