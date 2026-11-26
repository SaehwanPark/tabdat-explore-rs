# Robust scalar AR confidence sets through twenty-four instruments

## Scope

Extend retained-fit robust scalar Anderson–Rubin confidence sets to exactly
q=24 excluded instruments for one endogenous regressor under HC1 and one-way
CR1. Preserve included-exogenous nuisance treatment, retained sample/groups,
corrections, metadata, output shape and all q<=23 behavior. The boundary
polynomial has degree at most 48, so its algebraic limit is 48 real roots/25
components; numerically ambiguous roots must continue to fail closed. q>=25 and
other endogenous/covariance dimensions remain unsupported.

## Verification plan

- Use two deterministic complete 576-row designs with one included control,
  K_aux=26, residual DF=550 and 96 clusters of six observations. Confirm exact
  retained row/group order in the independent reference and Rust.
- Add independent Base R HC1/CR1 references. Cross-check degree-48 determinant
  coefficients at 49 Chebyshev nodes against direct determinants at a separate
  49-node set; directly check cluster covariance at near-real determinant-root
  candidates. Bracket set endpoints with direct Wald evaluations and check
  corrections, metadata, membership probes and instrument rescaling.
- Test q=24 HC1/CR1 results, retained-fit/runtime state, JSON/text output and
  q=25 rejection before later file effects. Preserve q<=23 reference cases.
- Test that an ill-conditioned degree-48 maximal-root product fails closed;
  cross-check applied degree-48 HC1/CR1 boundaries against the independent
  design reference without loosening tolerances. Update execution limits, help,
  command results, roadmap, SPEC, current coverage summary and runnable examples
  to state q<=24 and q>=25 unsupported. Keep older per-q evidence intact.
- Measure peak RSS and elapsed time for focused stats, runtime and CLI workflows.
  Run high-memory tests serially and include a q=23 reference regression because
  the polynomial boundary helper is shared.
- Run focused tests across affected packages and integration boundaries,
  regenerate fixtures byte-for-byte, and run scoped Clippy, formatting and diff
  checks.

## Progress

- [x] Record the bounded scope and acceptance checks.
- [x] Open draft PR before implementation.
- [ ] Implement q=24 support and independent reference fixtures.
- [ ] Update runtime/CLI coverage, current limits and examples.
- [ ] Verify numerical references, retained state, failure atomicity and resource use.
- [ ] Review the final diff, mark ready, satisfy repository checks and merge.
