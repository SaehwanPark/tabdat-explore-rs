# predict

Installed syntax:
`predict newvar [, xb | xbu | u | residuals | pr | mean | mills | ycond | yexpected | n | hazard | cumhaz | survival | pmf(k) | cdf(k) | pmf(cutoff_variable) | cdf(cutoff_variable)]`

Append a DOUBLE prediction column using retained coefficients on **current rows**:
- Quantile regression: xb (fitted conditional quantile) or residuals.
- Supported regress/2SLS: xb (default) or residuals (outcome minus xb).
- Within FE: xb (normalized common intercept + current X beta), xbu (retained
  unit intercept + current X beta), u (normalized unit deviation), or residuals
  (current outcome minus xbu). **Use xbu for conditional fitted levels.**
- Logit: pr (probability) or xb (log odds).
- Probit: pr (Gaussian probability) or xb (latent index).
- Lower-censored Gaussian Tobit: xb (latent mean), pr (uncensoring probability),
  mean (observed censored mean E[max(lower,Y*)|X]). Not a truncated mean or count n.
  Missing/nonfinite current predictors give NULL; numerical failure aborts before
  publication. Current outcome values are unused. At most10000 current rows.
  No auxiliary prediction options; retained-X effects use `estat margins`.
- Lower-truncated Gaussian regression: xb (underlying latent mean), pr
  (population retention P(Y*>lower|X), NOT conditional probability1), mean
  (conditional truncated mean E[Y*|Y*>lower,X], NOT censored mean).
  Current X only; missing/nonfinite predictors give NULL. At most10000 current
  rows, point-only without auxiliary options. Unrepresentable conditional means
  fail atomically; finite xb/pr do not require an unnecessary Mills division.
  Retained-X conditional-mean effects use `estat margins`; see help truncreg.
- Explicit Heckman two-step: xb (latent X beta), pr (selection probability),
  mills (selection inverse Mills ratio), ycond (conditional-selected outcome
  mean) or yexpected (zero-filled observed contribution). Current y/d are unused.
  xb needs only outcome X, pr/Mills only selection Z, other modes both; relevant
  missing/nonfinite regressors give NULL, irrelevant columns need not exist.
  Numerical/Mills-range failures abort atomically. At most10000 current rows;
  point-only, no margins or auxiliary options. Not Tobit mean/count n or causal
  effects; see help heckman for formulas and identification limits.
- Native Weibull streg: xb (PH X beta), hazard (instantaneous hazard), cumhaz
  (cumulative hazard), survival (net exp(-H)). Timed modes require current X and
  the original time name, never event; xb requires current X only. Missing/nonfinite
  required inputs give NULL; complete finite nonpositive time or unrepresentable
  requested results fail atomically. Genuine survival0 is allowed. At most10000
  rows, points only, no auxiliaries; no count n/binary pr/mean/residual semantics.
  Net survival is not all-cause survival or competing-risk incidence. See help streg.
- Native Cox stcox: xb (raw PH X beta), cumhaz (Breslow cumulative hazard),
  survival (net exp(-H)); no instantaneous hazard from baseline jumps. Timed
  modes use current X/time, never event; xb needs only X. Time0/before-first
  gives H0/S1; after-last holds its last step, not a validated tail extrapolation.
  Negative finite time/schema/numerical failures publish nothing; required
  missing/nonfinite inputs yield NULL. Baseline reference is all retained
  predictor means, including dummy means. No baseline/prediction uncertainty,
  count/binary/mean/residual meanings or auxiliaries. See help stcox.
- Poisson: n (conditional mean exp(xb)) or xb (log conditional mean).
- NB2: n (conditional mean exp(xb)) or xb (log conditional mean), using mean-model coefficients, not lnalpha as a predictor.

**Bare predict remains xb for every installed family.** Request probabilities
with `, pr` or count means with `, n` explicitly. Count means are not probabilities
or integer draws; do not clip at one. `pr` is not Poisson prediction and `n` is
not supported after linear/binary fits. NB2 also supports n/xb and explicit pmf(k)/cdf(k), not binary pr. Prediction kinds cannot be combined.

Example (after loading numeric x/y):
- `regress y x`
- `predict fitted`
- `predict fitted_explicit, xb`
- `predict residual, residuals`
- `head 8`
- `export predictions.csv`

Count example:
- `poisson visits age income, robust`
- `predict expected_visits, n`
- `predict log_mean, xb`

Data and model notes:
- No refitting or estimation-sample filtering: excluded rows can receive
  predictions if their current predictors are available. Outcome values do not
  matter except for linear residuals. Weight/cluster columns are not needed,
  except the fitted unit identifier for FE conditional predictions.
- xb requires the original predictor names. Renaming/dropping a required column
  does not rewrite the model: restore it or deliberately refit. Retained-X
  `estat margins` remains separate from these current-row predictions.
- A clustered Poisson/NB2 prediction still needs mean predictors only, not the
  outcome or current cluster ID. Original groups/covariance remain retained for
  `estat margins`; clustered fitting does not add prediction intervals.
- Within-FE xbu/u/residuals require the original ID name and exact logical type,
  but no current time column. Missing/nonfinite/unestimated IDs yield NULL, not
  zero effects; u needs no current predictors/outcome. xb needs no ID. Effects
  and centers come from the joint estimation sample and never change with current
  rows. Unsupported options, missing columns or ID type mismatch fail atomically.
  FE missing/nonfinite required values and overflowing calculations yield NULL.
- OLS/2SLS/quantile fits retain native SQL NULL/NaN/infinity arithmetic. Binary/count
  nonfinite or unrepresentable indices yield NULL. Stable binary links avoid
  exponential overflow. Poisson/NB2 overflowing means yield NULL; finite negative
  exponential underflow is zero. Neither changes row order or the fit.
- `newvar` must not exist. Choose a fresh name or deliberately drop an output;
  there is no replacement. A prediction failure leaves data/labels/model intact.
  A failed new runtime fit clears stale estimation, so successfully refit before
  predicting again.
- Terminal/--text confirms dataset dimensions. CLI-v1 `PredictionResult` contains
  dataset metadata, not vectors; inspect with head/tail or save/export values.

Not installed: binary/Poisson residuals, NB2 residuals, spatial_lag, posterior_predictive,
std/interval/level()/saving() prediction, or count cutoffs beyond the bounded integer0..100000.
`probabilities` is not an accepted option spelling. Discovery/parser breadth does not
imply an installed workflow for other families.

Count distributional probabilities:
- After Poisson/NB2, `predict p, pmf(k)` means P(Y=k); `cdf(k)` means P(Y<=k),
  conditional on current X and any fitted offset, using retained coefficients
  and NB2 alpha. k may be an integer literal0..100000 or the name of a current numeric
  cutoff variable. Choose one prediction kind. Missing cutoff values produce NULL;
  noninteger, negative or greater-than100000 values fail before adding the column.
- For cutoffs above99, a single prediction may add at most10,000,000 cutoff
  increments across rows with usable model inputs. Exceeding this limit fails
  before adding the column. `estat countprob` keeps its separate1..100 tail limit.
- These assume the full fitted law, not merely a valid conditional mean or
  robust covariance. They have no parameter uncertainty, calibration guarantee
  or formal test. `estat countprob` remains an ORIGINAL retained-sample summary;
  these predictions apply to current rows, including rows excluded at fitting.
- Current outcome/cluster columns are unnecessary; fitted offsets are required.
  Missing/nonfinite or overflowing indices/means produce NULL. Exponential
  underflow to zero uses a point mass at zero. Probability numerical failure
  aborts before publication, leaving data/labels/fit unchanged. Empty data works.
- Example: `poisson visits age, offset(log_time)` then `predict p0, pmf(0)`
  and `predict at_most_two, cdf(2)`. For current cutoffs, generate a numeric
  integer column and use `predict at_most_k, cdf(k)`. Inspect/export the appended
  DOUBLE columns.
  See examples/count-probability/README.md for a complete Poisson/NB2 workflow.
