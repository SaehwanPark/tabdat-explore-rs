# MMA §14.7.5 maximum-score binary response

## Outcome

Completed a bounded native maximum-score workflow for the fishing-mode
charter-versus-pier example: binary `charter` on an intercept and one continuous
`lnrelp` predictor, using the 630-row author sample. The estimator enumerates all
one-dimensional threshold regions in both slope directions, reports a unit-norm
coefficient representative and exact empirical score, and retains the
classification for terminal, JSON and current-row prediction output.

This is point estimation and classification only. It does not add a general
multivariable maximum-score optimizer, probability predictions, marginal
effects, weights or a covariance estimator. Maximum-score coefficients are
identified only up to scale; predictions are sign classifications. Tied
optimal threshold regions remain visible and receive a deterministic selected
rule without implying unique identification.

## Implemented behavior

- `maxscore binary_y continuous_x` accepts exactly one numeric binary outcome,
  one numeric predictor and an intercept. The fit uses joint complete-finite
  rows in source order; finite outcome values other than exact 0/1 fail. There
  are no weights, `if` filters, factors, extra predictors or no-intercept form.
- Exact finite-sample enumeration includes both constant classifiers and every
  split between distinct predictor values in each direction, retaining every
  tied maximizer. The selected coefficients have Euclidean norm one; their
  scale is not identified.
- A successful fit retains the sample and selected rule. `predict target,
  class` applies the rule to current rows in physical order, writes numeric 0/1
  and returns NULL for missing/nonfinite predictors. Failed fits clear a prior
  estimator without changing data or labels; failed predictions preserve
  session state.
- CLI text and JSON identify the estimator and coefficient normalization,
  report sample/event/score counts and confusion counts for the selected rule,
  and state that inference and probability predictions are not provided. Tied
  rules can have different confusion cells. JSON admission includes batch and
  script workflows.
- The source-pinned example and independent Base R reference compare the source
  cohort, both tied maximizing regions, unit-norm point estimates, all retained
  and current-row classifications and the author output's 536/630 score and
  TP/FP/FN/TN counts of 421/63/31/115.

## Reporting-contract decision

The author program's 50-replication bootstrap standard errors are approximate,
and the ordinary nonparametric bootstrap is inconsistent for maximum-score
inference. The native result therefore contains no covariance, standard errors,
tests, p-values or confidence intervals. A separately justified inference
method remains future work.

## Verification

- `cargo test -p tabdat-stats --test maximum_score`: 7 passed in the original
  implementation commit; statistics behavior is unchanged in the CLI/runtime
  increment.
- `cargo test -p tabdat-language --test maximum_score`: 2 passed.
- `cargo test -p tabdat-runtime --test maximum_score_contract`: 3 passed.
- `cargo test -p tabdat-reporting --test maximum_score_contract`: 1 passed.
- `cargo build --locked --bin tabdat-explore-rs` passed; native `--json` and
  `--text` scripts ran from fresh output directories.
- Official `Nldata.asc` SHA-256 matched the pinned value. The checked converter
  produced 1,182 ordered source rows. The standalone native script and Base R
  4.6.2 reference matched all 630 cohort/prediction rows, both maximizing
  regions, coefficients, 536/630 score and TP/FP/FN/TN=421/63/31/115. Checker
  tolerance is 1e-10 for score/rule/coefficients and 1e-12 for `lnrelp`.
- `cargo check --workspace`, `cargo fmt --all --check`, `git diff --check`,
  `cargo clippy --workspace --lib --bins -- -D warnings` and focused test-target
-  Clippy checks passed. Final local source/diff review found no actionable
  defects. GitHub reports the PR as mergeable with no required status checks or
  reviews.
- A prior full `cargo test -p tabdat-stats` run had one unrelated failure in
  `iv_confidence::robust_multi::tests::degree_forty_two_boundary_supports_forty_two_roots_and_twenty_two_components`;
  it reproduces on the unchanged base commit `3e39d8a2`. The focused
  maximum-score stats tests pass.

## Sources

- [Author program](https://cameron.econ.ucdavis.edu/mmabook/mma14p2maxscore.lim)
  and [LIMDEP output](https://cameron.econ.ucdavis.edu/mmabook/mma14p2maxscore.out)
- Manski (1975), [“Maximum Score Estimation of the Stochastic Utility Model of
  Choice”](https://doi.org/10.1016/0304-4076(75)90032-9)
- Abrevaya and Huang (2005), [“On the Bootstrap of the Maximum Score
  Estimator”](https://doi.org/10.1111/j.1468-0262.2005.00613.x)
