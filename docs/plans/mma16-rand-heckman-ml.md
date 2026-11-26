# MMA §16.6 RAND Heckman maximum likelihood

Status: implementing on `codex/mma16-selection`.

## Target

Add the missing bounded default maximum-likelihood mode for the Cameron and
Trivedi, *Microeconometrics: Methods and Applications* (2005), §16.6 RAND
selection illustration (`mma16p3selection.do`, pp. 553–556, 565). Preserve the
existing explicit `twostep` command and its inference contract. The bare
`heckman y x, selectdep(d) select(z)` form will execute joint IID Gaussian
maximum likelihood.

The example will also compose the already available probit and selected-sample
OLS commands as two-part reference fits. This does not add a general two-part
estimator or imply causal identification.

## Scope and observable contract

- Fit selection `d = 1[z'gamma + u > 0]` and observed outcome
  `y = x'beta + v`, with standard-normal `u`, Gaussian `v` of scale `sigma`,
  and `corr(u,v) = rho`. Outcome and selection designs may differ; the existing
  common `noconstant` switch continues to apply to both equations.
- First retain rows with finite exact-binary `d` and finite selection design.
  Reject non-binary values before conversion. A selected row also needs finite
  outcome and outcome design; when one fails, exclude it from both equations.
  Unselected outcome and outcome-design values are irrelevant. Preserve ordered
  original positions for both retained samples.
- Require both selection classes, identified full-rank designs, supported row
  and work bounds, finite likelihood evaluations, strict optimizer convergence,
  and positive-definite observed information. Failure must not leave stale fit
  state or mutate data/labels. Do not clip `rho`, use a pseudoinverse, or
  silently substitute a different covariance.
- Initialize deterministically from the existing two-step estimate, maximize
  the joint observed-data likelihood with a bounded guarded optimizer, and
  report the full inverse observed-information covariance for
  `[outcome coefficients, selection coefficients, lnsigma, atanh(rho)]`.
  Report asymptotic-normal coefficient inference and delta-method `sigma`/`rho`
  diagnostics. Also fit the same-sample independent-errors null (`rho=0`) and
  report the one-df likelihood-ratio test.
- Retain an owned ML result/state distinct from `HeckmanTwoStepResult`. Report
  the method, both samples, likelihood, convergence diagnostics, joint
  covariance order, estimates, `sigma`, `rho`, and the no-exclusion warning in
  terminal and JSON output. Existing two-step output and serialized variants
  remain compatible.
- Support existing point-only current-data `xb`, `pr`, `mills`, `ycond`, and
  `yexpected` projections for ML fits using the same Gaussian formulas.
  Prediction failure stays atomic and preserves the active fit, data, and
  labels; uncertainty on predictions is out of scope.
- Reject robust/cluster covariance, weights, factor expansions, and unsupported
  prediction options explicitly. A missing exclusion restriction means
  identification rests on Gaussian functional form and may be weak; a valid
  exclusion is a user assumption, not an automatic test. Estimates are
  associations, not causal effects.
- Add a source-pinned RAND script/converter and a concise reproducible example
  that compares two-part, explicit two-step, and default ML estimates, then
  checks predictions and exact sample positions. Do not commit author data.

## Evidence and acceptance

- Pin the official author `.do`, `.txt`, and `.asc` fingerprints and document
  the source's year-2 restriction, education exclusion, 5,574 ordered rows,
  4,281 selected outcomes, and 1,293 unselected rows. Preserve ASCII-export
  precision when comparing source fields.
- Independently reproduce the joint likelihood with base R `optim` and a
  separately evaluated Hessian. Compare exact row membership, both equation
  coefficient vectors, full covariance/SEs, likelihood, `sigma`, `rho`, the
  independence LR test, and all supported predictions. Compare printed author
  points where available and explain differences due to ASCII precision or
  optimizer stopping; do not widen tolerances to hide them.
- Add focused kernel tests for likelihood/score derivatives, null nesting,
  parameter/covariance ordering, sample identity, row permutation, affine
  outcome units, successful retained-state predictions, and invalid,
  rank-deficient, boundary, work-limit, non-convergence, and atomic-failure
  cases. Exercise the command through the runtime/JSON/text boundaries.
- Update `SPEC.md`, architecture only if component ownership changes, the active
  roadmap row, coverage ledger, Heckman help/schema, and a linked MMA example
  with supported behavior and remaining limits.
- Inspect the final diff and affected callers; run the smallest changed tests,
  then affected crate/integration boundaries, Clippy, format, and diff checks.
  Record selected test counts and any unrelated baseline failures here.

## Progress

- [x] Confirmed the default execution gap, existing two-step semantics, and
  official RAND source counts/field consistency.
- [x] Created the bounded implementation plan and draft PR.
- [ ] Implement the joint estimator, owned state, runtime, reporting, and
  current-data point projections.
- [ ] Add independent R/source evidence, focused contract tests, and current
  documentation updates.
- [ ] Complete the final review and repository verification.

