# Robust scalar AR confidence sets through twenty-five instruments

## Scope

Extend retained-fit robust scalar Anderson–Rubin confidence sets to exactly
q=25 excluded instruments for one endogenous regressor under HC1 and one-way
CR1. Preserve included-exogenous nuisance treatment, retained sample/groups,
corrections, metadata, output shape and all q<=24 behavior. The boundary
polynomial has degree at most 50, so its algebraic limit is 50 real roots/26
components; numerically ambiguous roots must continue to fail closed. q>=26 and
other endogenous/covariance dimensions remain unsupported.

## Verification plan

- Use two deterministic complete 600-row designs with one included control,
  K_aux=27, residual DF=573 and 100 clusters of six observations. Confirm exact
  retained row/group order in the independent reference and Rust.
- Add independent Base R HC1/CR1 references. Cross-check degree-50 determinant
  coefficients at 51 Chebyshev nodes against direct determinants at a separate
  51-node set; directly check cluster covariance at near-real coefficient-root
  candidates. Bracket set endpoints with direct Wald evaluations and check
  corrections, metadata, membership probes and instrument rescaling.
- Test q=25 HC1/CR1 results, retained-fit/runtime state, JSON/text output and
  q=26 rejection before later file effects. Preserve q<=24 reference cases.
- Test that an ill-conditioned degree-50 maximal-root product fails closed;
  cross-check applied degree-50 HC1/CR1 boundaries against the independent
  design reference without loosening tolerances. Update execution limits, help,
  command results, roadmap, SPEC, current coverage summary and runnable examples
  to state q<=25 and q>=26 unsupported. Keep older per-q evidence intact.
- Measure peak RSS and elapsed time for focused stats, runtime and CLI workflows.
  Run high-memory tests serially and include q=24 reference and scaling regressions
  because the polynomial boundary helper is shared.
- Run focused tests across affected packages and integration boundaries,
  regenerate fixtures byte-for-byte, and run scoped Clippy, formatting and diff
  checks.

## Progress

- [x] Record the bounded scope and acceptance checks.
- [x] Open draft PR before implementation.
- [x] Implement q=25 support and independent reference fixtures.
- [x] Update runtime/CLI coverage, current limits and examples.
- [x] Verify numerical references, retained state, failure atomicity and resource use.
- [x] Review the final diff, mark ready, satisfy repository checks and merge.

## Verification progress

- The Base R 4.6.2 generator reproduced the two q=25 data fixtures and reference
  CSV byte-for-byte.
- The degree-50 ambiguous-root rejection and q=26 stats API rejection each pass.
- The q=25 HC1/CR1 independent-reference test passes all four design/covariance
  cases in 8,473.49 seconds with 10,236,542,976 bytes maximum RSS. Run high-degree
  tests serially; the q=25 reference case is substantially more expensive than q=24.
- The runtime retained-fit/q=26 rejection test passes in 8,339.18 seconds with
  10,309,320,704 bytes maximum RSS.
- The q=25 HC1 instrument-rescaling invariance test passes in 4,112.01 seconds
  with 10,231,824,384 bytes maximum RSS.
- The q=24 HC1/CR1 reference regression passes all four design/covariance cases
  in 3,624.37 seconds with 4,995,940,352 bytes maximum RSS.
- The q=24 HC1 instrument-rescaling regression passes in 1,823.82 seconds with
  4,992,090,112 bytes maximum RSS.
- The q=25 HC1/CR1 CLI integration test passes through JSON HC1, JSON CR1 and text
  HC1 workflows in 5,900.64 seconds with 9,467,969,536 bytes maximum RSS.
- q=25 CLI discovery/JSON contracts and q=26 rejection-before-later-file-effects
  checks pass; the scoped CLI contract tests each report one passing test.
- Scoped Clippy passes with `-D warnings` for the affected library crates and
  CLI integration-test targets. Formatting and diff checks pass.
