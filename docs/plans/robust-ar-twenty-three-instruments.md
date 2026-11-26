# Robust scalar AR confidence sets through twenty-three instruments

## Scope

Extend retained-fit robust scalar Anderson–Rubin confidence sets to exactly
q=23 excluded instruments for one endogenous regressor under HC1 and one-way
CR1. Preserve included-exogenous nuisance treatment, retained sample/groups,
corrections, metadata, output shape and all q<=22 behavior. The degree-46
boundary must support up to 46 simple real roots and 24 accepted components;
q>=24 and other endogenous/covariance dimensions remain unsupported.

## Verification plan

- Use two deterministic complete 552-row designs with one included control,
  K_aux=25, residual DF=527 and 24 clusters of twenty-three observations.
  Confirm exact retained row/group order in the independent reference and Rust.
- Add independent Base R HC1/CR1 references. Cross-check degree-46 determinant
  coefficients at 47 Chebyshev nodes against direct determinants at a separate
  47-node set; bracket set endpoints with direct Wald evaluations and check
  corrections, metadata, membership probes and instrument rescaling.
- Test q=23 HC1/CR1 results, retained-fit/runtime state, JSON/text output and
  q=24 rejection before later file effects. Preserve q<=22 reference cases.
- Extend the polynomial boundary test to 46 roots and 24 components. Update
  execution limits, help, command results, roadmap, SPEC, current coverage
  summary and runnable examples to state q<=23 and q>=24 unsupported. Keep older
  per-q reference results as historical evidence.
- Measure peak RSS and elapsed time for the focused stats, runtime and CLI
  workflows. Run the new high-memory tests serially; record resource use rather
  than extrapolating from the q=22 measurements.
- Run focused tests across affected packages and integration boundaries,
  regenerate fixtures byte-for-byte, and run scoped Clippy, formatting and diff
  checks.

## Progress

- [x] Record the bounded scope and acceptance checks.
- [ ] Open draft PR before implementation.
- [ ] Implement q=23 support and independent reference fixtures.
- [ ] Update runtime/CLI coverage, current limits and examples.
- [ ] Verify numerical references, retained state, failure atomicity and resource use.
- [ ] Review the final diff, mark ready, satisfy repository checks and merge.
