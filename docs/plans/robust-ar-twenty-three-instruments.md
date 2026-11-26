# Robust scalar AR confidence sets through twenty-three instruments

## Scope

Extend retained-fit robust scalar Anderson–Rubin confidence sets to exactly
q=23 excluded instruments for one endogenous regressor under HC1 and one-way
CR1. Preserve included-exogenous nuisance treatment, retained sample/groups,
corrections, metadata, output shape and all q<=22 behavior. The boundary
polynomial has degree at most 46, so its algebraic limit is 46 real roots/24
components; numerically ambiguous roots must continue to fail closed, and this
limit is not a guarantee for every maximally rooted polynomial. q>=24 and other
endogenous/covariance dimensions remain unsupported.

## Verification plan

- Use two deterministic complete 552-row designs with one included control,
  K_aux=25, residual DF=527 and 92 clusters of six observations.
  Confirm exact retained row/group order in the independent reference and Rust.
- Add independent Base R HC1/CR1 references. Cross-check degree-46 determinant
  coefficients at 47 Chebyshev nodes against direct determinants at a separate
  47-node set; directly check cluster covariance at near-real determinant-root
  candidates. Bracket set endpoints with direct Wald evaluations and check
  corrections, metadata, membership probes and instrument rescaling.
- Test q=23 HC1/CR1 results, retained-fit/runtime state, JSON/text output and
  q=24 rejection before later file effects. Preserve q<=22 reference cases.
- Test that an ill-conditioned degree-46 maximal-root product fails closed;
  cross-check applied degree-46 HC1/CR1 boundaries against the independent
  design reference without loosening tolerances. Update
  execution limits, help, command results, roadmap, SPEC, current coverage
  summary and runnable examples to state q<=23 and q>=24 unsupported. Keep older
  per-q reference results as historical evidence.
- Measure peak RSS and elapsed time for the focused stats, runtime and CLI
  workflows. Run the new high-memory tests serially; record resource use rather
  than extrapolating from the q=22 measurements.
- The focused q=23 stats reference test selected one test, passed all four
  design/covariance combinations in 1,678.77 seconds, and peaked at
  2,447,327,232 bytes RSS.
- The focused q=23 runtime-retention/q=24-rejection test selected one test,
  passed in 1,674.42 seconds, and peaked at 2,521,595,904 bytes RSS.
- The focused q=23 CLI JSON/text workflow selected one test, passed in
  1,273.59 seconds, and peaked at 2,293,432,320 bytes RSS.
- The focused q=23 HC1 instrument-rescaling test selected one test, passed in
  835.94 seconds, and peaked at 2,438,201,344 bytes RSS.
- The focused q=22 HC1/CR1 regression selected one test, passed all four
  design/covariance combinations in 771.01 seconds wall clock (766.11 seconds
  reported by the test), and peaked at 1,196,867,584 bytes RSS.
- The q=24 CLI boundary contract selected one test and passed, including a
  fit-only JSON assertion that `scalar_ar_confidence_set_available` is false
  and a failed `estat ar, ci` that leaves the later export absent.
- Formatting and diff checks pass. Scoped Clippy passes for the affected stats,
  execution, reporting and runtime packages and all changed root CLI test
  targets. Root `--all-targets` Clippy also reaches an existing warning in the
  untouched `tests/cli_json_count_exposure_contract.rs` (`double_ended_iterator_last`);
  the changed root tests pass when selected directly.
- Run focused tests across affected packages and integration boundaries,
  regenerate fixtures byte-for-byte, and run scoped Clippy, formatting and diff
  checks.

## Progress

- [x] Record the bounded scope and acceptance checks.
- [x] Open draft PR before implementation.
- [x] Implement q=23 support and independent reference fixtures.
- [x] Update runtime/CLI coverage, current limits and examples.
- [x] Verify numerical references, retained state, failure atomicity and resource use.
- [x] Review the final diff, mark ready, satisfy repository checks and merge PR #510.
