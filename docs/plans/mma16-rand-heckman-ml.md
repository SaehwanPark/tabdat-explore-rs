# MMA §16.6 RAND Heckman maximum likelihood

Status: ready for PR review on `codex/mma16-selection`.

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
- [x] Implement the joint estimator, owned state, runtime, reporting, and
  current-data point projections.
- [x] Add independent R/source evidence, focused contract tests, and current
  documentation updates.
- [x] Complete the final review and repository verification.

## Verification record

- The exact public `.do`, `.txt` and `.asc` files match their pinned hashes.
  The POSIX awk converter produces all 5,574 ordered records/111,480 values;
  the base-R reference and native text/JSON scripts run from a clean temporary
  directory without committing author data.
- `check_reference.py` passes exact source cells and sample membership, both
  equation estimates, probit/OLS/two-step covariance, all 1,444 entries of the
  source-sized 38×38 ML covariance, likelihood/scale/correlation/LR gates, and
  all 55,740 values across ten source-row prediction columns. Maximum observed
  Rust/R differences are 5.6e-6 for ML coefficients and 7.4e-6 for ML
  projections; stated point tolerance is1e-5, and joint covariance tolerance
  is2e-8 (maximum observed difference1.45e-8). Other estimator gates use1e-8.
- Focused passing tests: stats ML unit3/integration4, runtime Heckman7,
  reporting Heckman JSON1, all reporting tests76, shell tests67, execution
  tests59, and domain wire2. Focused Clippy passes for the changed integration
  targets; workspace `cargo check`, production `cargo clippy --workspace
  --lib --bins -- -D warnings`, `cargo fmt --all --check`, and `git diff --check`
  pass. The native example was built and run through text and JSON script mode.
- The broader `cargo test -p tabdat-stats` run had130/131 unit tests pass; the
  remaining unrelated `iv_confidence::robust_multi::tests::degree_forty_two_...`
  test also fails alone at `robust_multi.rs:2357` (`−0.5312488` vs `−0.53125`,
  tolerance1e-6). That source is unchanged. Workspace all-target Clippy is also
  blocked by existing `needless_range_loop` warnings in untouched
  `tabdat-stats/tests/conditional_logit.rs:71` and
  `tabdat-runtime/tests/truncreg_contract.rs:190`; production and changed-target
  Clippy both pass.
- Architecture/crate boundaries did not change. No publication/deployment or
  whole-Chapter16 claim is made; there is no excluded RAND predictor, robust or
  household-cluster inference, prediction uncertainty, or general two-part
  estimator in this slice.

