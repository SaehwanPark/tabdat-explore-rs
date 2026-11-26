# MMA §4.6.4 Figure 4.1: quantile slope bands

Status: in progress

## Target

Complete the bounded Figure 4.1 coefficient plot from the Cameron–Trivedi
`mma04p2qreg.do` program. It fits conditional quantiles .05 through .95 by .05,
uses 200 common IID row-bootstrap replications with seed 10101, plots the
selected slope estimate at each quantile with `estimate ± 1.96 × bootstrap SE`,
and adds the OLS slope for the same sample. The author's label “simultaneous
quantile regression” describes the joint fit and covariance; the plotted
normal bands do not claim simultaneous coverage.

## Scope and contract

- Add a read-only `qregplot` command for an explicitly selected coefficient,
  quantile list, bootstrap seed/count, SVG destination and title.
- Use the current ordered complete-finite outcome/design sample, the existing
  certified quantile LP, common IID pairs across quantiles, and the existing
  B−1 bootstrap covariance convention. Reuse that same sample/design for OLS.
- Render coefficient estimates and their pointwise fixed-1.96 bands against
  quantile, plus the OLS coefficient reference line. Label the SVG and result
  as pointwise normal-approximation bands with no simultaneous-coverage claim.
- Publish the SVG only after estimation and rendering succeed. Never overwrite
  an existing destination; do not mutate data, labels, or retained-fit state.
- Preserve all existing `bootstrap` and `scatter` result and command contracts.

## Evidence and acceptance

- Pin the MMA author program and the original ASCII source identity/sample.
- Independently compare the 19 estimates, B−1 standard errors, lower/upper band
  values, OLS slope and retained sample against R `quantreg`/base R using the
  same explicit common row draws. Do not claim Stata RNG parity.
- Check SVG series coordinates, ordering, axes, labels and screen geometry;
  include a synthetic parser/runtime/CLI fixture and failures for invalid or
  rank-deficient requests and existing output paths.
- Update `SPEC.md`, the roadmap, coverage ledger, command results/help and the
  MMA qreg example with assumptions, evidence, run steps and remaining limits.
- Run focused affected tests first, then relevant crate/CLI boundaries, Clippy,
  formatting and diff checks. Record selected counts and outcomes here.

## Progress

- [x] Confirmed target and conventions from the original March 2005 author
  program (`mma04p2qreg.do`, block 3).
- [ ] Implement the command, estimator-to-plot result and SVG renderer.
- [ ] Add independent source and synthetic validation and update active docs.
- [ ] Review the final diff, run affected checks, and record outcomes.
- [ ] Complete PR review and merge.
