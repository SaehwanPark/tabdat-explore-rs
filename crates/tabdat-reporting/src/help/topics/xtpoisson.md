# xtpoisson

Installed bounded form: `xtpoisson y xvars, fe id(group)`.

Model-based unweighted conditional Poisson fixed effects with one or more numeric
slopes. Conditions on each selected group's count total; no common intercept is
estimated. Group members need not be repeated temporal observations. There is no
retained panel declaration, time option, or automatic lag construction.

Sample and identification:
- Raw group keys must all be nonmissing and finite, including rows later omitted
  for missing model fields. Numeric exact keys, text and boolean groups are supported.
- Jointly finite y/X rows are selected; finite counts on these cases must be exact
  integers in 0..100000. Invalid finite counts on excluded incomplete rows do not
  enter the fit. Exact integer/decimal cells are checked before conversion to f64;
  digits already rounded by a DOUBLE loader cannot be recovered.
- Then exclude whole groups whose selected count total is zero. Positive-total
  singleton groups remain in N/G but supply no slope information. Rank failure,
  predictors constant within groups, separation, nonpositive/degenerate information
  or exhausted Newton/line-search iterations fail rather than returning a boundary fit.
- Candidate rows<=40000, K1..32, rows*K²<=20000000, candidate groups<=10000 and
  retained positive-total G>K. No weights, offsets, factor expansions or if filters.

Inference:
- Stable within-group softmax gives p_ig; conditional likelihood includes the
  multinomial coefficient, given the selected count total n_g. Covariance is the
  inverse sum of n_g-weighted within-group predictor covariance, without residual-
  DF scaling. Coefficients use asymptotic normal z/p/95% intervals; same-total
  zero-slope LR and joint slope Wald use chi-square(K).
- Independent groups and the conditional Poisson law are assumptions, not tested
  conclusions. Robust/cluster/survey covariance, overdispersion correction,
  general lincom/test, margins, RE and automatic bootstrap are not installed.
- Reports identify joint-finite and zero-total exclusions, retained group sizes,
  total counts, convergence and full slope covariance. No unit-effect uncertainty.

Retained current-data prediction:
- `predict slope_index, xb` (also bare predict): current X beta, **without** any
  common or group intercept. Group and outcome columns are not required.
- `predict fitted, n`: retained positive-group normalization times exp(current
  within-anchor X beta). Original fitted means sum to the selected group total.
  Current-data changes/subsets do not refit or renormalize. This is a fitted-unit
  conditional mean, not a population forecast or a new-group mean.
- `predict error, residuals`: current y minus that mean; no Poisson PMF/CDF,
  intervals, standard errors, saved draws or conditional-mode shorthand.
- Mean/residual modes need fitted group name and exact logical type. Unseen or
  excluded zero-total IDs yield NULL, not a fitted zero effect. Missing/nonfinite
  numeric cells or unrepresentable arithmetic yield NULL. Current prediction uses
  the same row/work bounds as fitting. Missing columns, logical-type mismatch,
  existing targets and unsupported modes fail without changing data/model.
- A new failed runtime fit clears the old model but leaves data unchanged; syntax
  errors preserve the old model. Export remains a separate no-clobber operation.

Example:

```text
xtpoisson visits income age, fe id(commune)
predict fe_mean, n
predict fe_index, xb
predict fe_residual, residuals
summarize fe_mean fe_residual
export conditional_predictions.csv
```

See examples/mma24p2poiscluster/individual_conditional_fe.md for bounded original
source, independent reference and historical author-precision evidence. The
existing Chapter23 scalar SQL conditional-FE workflow remains separate.
