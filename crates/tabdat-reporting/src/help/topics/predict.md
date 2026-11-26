# predict

Installed syntax:
`predict newvar [, xb | class | xbu | u | residuals | pearson | deviance | pr | pr se | pr interval | mean | mills | ycond | yexpected | n | n interval | hazard | cumhaz | survival | pmf(k) | pmf(k) se | pmf(k) interval | cdf(k) | cdf(k) se | cdf(k) interval | pmf(cutoff_variable) | pmf(cutoff_variable) se | pmf(cutoff_variable) interval | cdf(cutoff_variable) | cdf(cutoff_variable) se | cdf(cutoff_variable) interval]`

Append a DOUBLE prediction column using retained coefficients on **current rows**:
- Explicit-formula NLS: default/xb/mean is the nonlinear formula mean on current
  X, not a latent linear index; residuals additionally require current y. Missing
  required inputs give NULL, arithmetic failure is atomic. Parameter-only means
  need no X. No auxiliaries or uncertainty; see help nl.
- Quantile regression: xb (fitted conditional quantile) or residuals.
- Maximum score: `class` writes 0/1 values from the retained threshold rule;
  missing/nonfinite current predictors write NULL. This is sign classification,
  not a probability prediction or inferential interval.
- Supported regress/2SLS: xb (default) or residuals (outcome minus xb).
- Within FE: xb (normalized common intercept + current X beta), xbu (retained
  unit intercept + current X beta), u (normalized unit deviation), or residuals
  (current outcome minus xbu). **Use xbu for conditional fitted levels.**
- Gaussian exchangeable GEE (`xtgee`): default/xb is the population marginal
  mean under the identity link; residuals additionally require current y. No
  current group ID is needed. Missing/nonfinite required inputs give NULL;
  unrepresentable finite-input results fail atomically. See help xtgee.
- Conditional Poisson FE (`xtpoisson ..., fe id(group)`): default/xb is the
  slope-only index; n and residuals use retained fitted-group normalization on
  current X, never renormalize the current subset. Unseen/missing/zero-total IDs
  give NULL means/residuals; xb does not require ID. The group logical type must
  match the fit. These are conditional predictions, not new-group/population
  forecasts. Point-only; see help xtpoisson for sample and work bounds.
- Shared-gamma Poisson RE (`xtpoisson ..., re id(group)`): xb includes the
  intercept; n and residuals use the marginal population mean, not posterior
  group effects. No current ID is required. Point-only, with atomic failures
  for unrepresentable finite-input results; see help xtpoisson.
- Logit: pr (probability) or xb (log odds). Additive `i.var` and
  `ib(level).var` main effects use the retained typed levels when scoring
  current rows; missing factor values or levels absent at fit time yield NULL.
  `pr se` writes the pointwise delta-method SE for the current-row probability.
  `pr interval [level(95)]` appends the point estimate and
  `<newvar>_lower`/`<newvar>_upper` columns from a normal interval on the
  retained latent-index scale transformed through the logit link.
- Probit: pr (Gaussian probability) or xb (latent index), with the same
  retained factor-level prediction rules and `pr se`/`pr interval` uncertainty
  options as logit; interval endpoints use the probit inverse link.
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
- Heckman ML or explicit two-step: xb (latent X beta), pr (selection
  probability), mills (selection inverse Mills ratio), ycond
  (conditional-selected outcome mean) or yexpected (zero-filled observed
  contribution). Current y/d are unused.
  xb needs only outcome X, pr/Mills only selection Z, other modes both; relevant
  missing/nonfinite regressors give NULL, irrelevant columns need not exist.
  Numerical/Mills-range failures abort atomically. At most10000 current rows;
  point-only, no margins or auxiliary options. Not Tobit mean/count n or causal
  effects; see help heckman for formulas and identification limits.
- Native Weibull/Gompertz streg: xb (PH X beta), hazard (instantaneous hazard), cumhaz
  (cumulative hazard), survival (net exp(-H)). Timed modes require current X and
  the original time name, never event; xb requires current X only. Missing/nonfinite
  required inputs give NULL; complete finite negative time (or zero for Weibull)
  or unrepresentable requested results fail atomically. Gompertz time0 gives
  h=exp(xb), H0/S1; its xb excludes gamma*time, and negative gamma retains a
  positive infinite-tail survival mass, not identified causal cure. Genuine
  survival0 is allowed. At most10000
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
- Poisson and NB2: n (conditional mean exp(xb)), xb (log conditional mean),
  residuals (current outcome minus the current conditional mean), pearson
  (unscaled Pearson residual), or deviance (signed square root of the family
  unit deviance). Pearson uses `(y-mu)/sqrt(mu)` for Poisson and
  `(y-mu)/sqrt(mu+alpha*mu^2)` for NB2, with retained alpha. Deviance uses the
  Poisson unit deviance `2*(y*log(y/mu)-(y-mu))` or NB2 unit deviance
  `2*(y*log(y/mu)-(y+1/alpha)*log((1+alpha*y)/(1+alpha*mu)))`; the `y=0`
  terms use their zero-count limits. These are scored on current rows using
  retained mean coefficients, current outcome/predictors and any current fixed
  offset/exposure. Missing/nonfinite inputs and undefined residuals give NULL;
  a present outcome outside the exact supported count domain fails atomically.
  A zero fitted mean gives NULL Pearson, zero deviance when y is zero, and NULL
  deviance when y is positive. Covariance selection does not change residuals;
  they are not standardized/studentized residuals or leverage diagnostics.

**Bare predict remains xb for regression families.** Maximum score requires
explicit `, class`. Request binary probabilities with `, pr` or count means with
`, n` explicitly. Count means are not probabilities
or integer draws; do not clip at one. `pr` is not Poisson prediction and `n` is
not supported after linear/binary fits. NB2 also supports n/xb and explicit pmf(k)/cdf(k), not binary pr. Prediction kinds cannot be combined.

Example (after loading numeric x/y):
- `regress y x`
- `predict fitted`
- `predict fitted_explicit, xb`
- `predict residual, residuals`
- `head 8`
- `export predictions.csv`
- `maxscore charter lnrelp`
- `predict predicted_charter, class`

Count example:
- `poisson visits age income, robust`
- `predict expected_visits, n`
- `predict response_residual, residuals`
- `predict pearson_residual, pearson`
- `predict deviance_residual, deviance`
- `predict log_mean, xb`
- `predict p0, pmf(0)`
- `predict p0_se, pmf(0) se`
- `predict p0_interval, pmf(0) interval`
- `predict at_most_two, cdf(2)`
- `predict at_most_two_se, cdf(2) se`
- `predict at_most_two_interval, cdf(2) interval level(90)`
- `predict future_count, n interval level(90)`

Data and model notes:
- No refitting or estimation-sample filtering: excluded rows can receive
  predictions if their current predictors are available. Outcome values do not
  matter except for residuals. Poisson/NB2 response residuals use the current
  outcome and fitted conditional mean. Weight/cluster columns are not needed,
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

Binary `se` and `interval` require explicit `pr`; they cannot be combined.
The retained model-based, independent-row robust or one-way clustered
covariance is used as selected at fit time. Predictors are treated as fixed;
these are parameter-uncertainty measures, not future-outcome prediction
intervals, simultaneous bands or covariate-distribution uncertainty. Bounds
remain in [0,1] and can be asymmetric. A zero delta variance returns zero SE
and point bounds. Missing/nonfinite predictors and unseen factor levels produce
NULL in each requested output. Target/bound collisions or numerical/publication
failures leave data and fit unchanged. Current rows excluded during fitting can
still be scored when their predictors are available.

Not installed for the other families: binary residuals, spatial_lag,
posterior_predictive, general std/interval/level()/saving() prediction, or
count cutoffs beyond the bounded integer 0..100000. `se` is limited to binary
`pr` and count PMF/CDF. Count PMF/CDF and `n` intervals are the bounded
exceptions described below.
`probabilities` is not an accepted option spelling. Discovery/parser breadth does not
imply an installed workflow for other families.

Count response residuals:
- After native Poisson/NB2, `predict residual, residuals` writes the raw current-row
  response residual `y - exp(xb)`. It uses the retained mean coefficients and
  current predictors plus the fitted offset/exposure. It does not use covariance
  or dispersion uncertainty and is not a Pearson, deviance or standardized
  residual.
- The current outcome column must be present and numeric. Missing/nonfinite
  outcomes or model inputs produce NULL. Finite outcomes must be exact counts in
  the family domain (Poisson 0..2^53-1; NB2 0..100000); other finite values fail
  atomically. Predictions cover current rows, including rows excluded at fit.

Count distributional probabilities:
- After Poisson/NB2, `predict p, pmf(k)` means P(Y=k); `cdf(k)` means P(Y<=k),
  conditional on current X and any fitted offset, using retained coefficients
  and NB2 alpha. k may be an integer literal0..100000 or the name of a current numeric
  cutoff variable. Choose one prediction kind. Missing cutoff values produce NULL;
  noninteger, negative or greater-than100000 values fail before adding the column.
- `predict p_se, pmf(k) se` writes the pointwise delta-method SE for P(Y=k)
  instead of the probability; `predict cdf_se, cdf(k) se` does the same for
  P(Y<=k). Use separate commands for point values. The `se` option is also
  supported with a current cutoff variable. SEs use the covariance selected by
  the retained fit, including NB2's joint beta/lnalpha covariance. Current
  predictors and fixed offsets/exposure are treated as known. This is parameter
  uncertainty for the conditional probability, not future-count variation or a
  confidence interval.
- Add `interval [level(95)]` to a PMF/CDF request to append the point estimate
  and two columns named `<target>_lower` and `<target>_upper`. These are
  pointwise logit-Wald intervals using an asymptotic normal critical value and
  the same retained covariance as the SE. Current predictors and fixed
  offsets/exposure are treated as known. Levels must be strictly between 0 and
  100 and yield a finite normal critical value. Endpoints remain in [0,1]; separate
  PMF intervals need not sum to one, and CDF intervals need not be monotone
  across cutoffs. They are parameter-uncertainty intervals, not future-count
  prediction intervals, simultaneous bands or a guarantee of the fitted law.
  Zero means and zero transformed variance return the point estimate at both
  endpoints. Name collisions among the point/lower/upper columns fail before
  any of them are published.
- Add `interval [level(95)]` to `n` after Poisson/NB2 to append the conditional
  mean and `<target>_lower`/`<target>_upper` columns atomically. Bounds are the
  discrete equal-tail quantiles under the fitted Poisson/NB2 law, with mean and
  NB2 alpha fixed and current predictors plus offsets/exposure held fixed. The
  lower endpoint is the first count whose CDF reaches `(100-level)/200`; the
  upper endpoint is the first count whose survival above it is at most that
  tail. Discreteness makes attained coverage at least nominal. This interval
  describes future-count variation and does not include parameter uncertainty
  from the retained covariance. Count bounds are integers stored as DOUBLEs;
  they are limited to 0..100000, and requests beyond that support fail before
  publication. Missing/nonfinite current model inputs give NULL in all three
  columns. Search work is capped at 10,000,000 CDF-prefix terms across current
  rows. Levels must represent two distinct equal tails.
- For cutoffs above 99, a single prediction may add at most 10,000,000 cutoff
  increments across rows with usable model inputs. SE requests count recurrence
  and gradient work; NB2 CDF SEs can charge up to four cutoff passes. Intervals
  can charge up to seven passes for NB2 CDFs. Exceeding
  the limit fails before adding the column. `estat countprob` keeps its separate
  1..100 tail limit.
- These assume the full fitted law, not merely a valid conditional mean or
  robust covariance. PMF/CDF SEs and intervals use that selected covariance
  but do not establish the distributional law; calibration guarantees and
  formal tests are not provided. `estat countprob` remains an ORIGINAL
  retained-sample summary; these predictions apply to current rows, including
  rows excluded at fitting.
- Current outcome/cluster columns are unnecessary; fitted offsets are required.
  Missing/nonfinite or overflowing indices/means produce NULL. Exponential
  underflow to zero uses a point mass at zero. Probability or gradient numerical
  failure aborts before publication, leaving data/labels/fit unchanged. Empty data works.
- Example: `poisson visits age, offset(log_time)` then
  `predict future_count, n interval`, `predict p0, pmf(0)`,
  `predict p0_se, pmf(0) se`, or `predict p0_interval, pmf(0) interval`.
  Also try `predict at_most_two, cdf(2)` and
  `predict at_most_two_interval, cdf(2) interval level(90)`. For current
  cutoffs, generate a numeric integer column and use
  `predict at_most_k_interval, cdf(k) interval` or
  `predict p_k_interval, pmf(k) interval`.
  Inspect/export the appended DOUBLE columns.
  See examples/count-probability/README.md for a complete Poisson/NB2 workflow.
- After either `logit` or `probit`, use `predict p_se, pr se` or
  `predict p_interval, pr interval level(90)`. The interval command appends
  point/lower/upper DOUBLE columns; inspect them with `head` or `export`.
  These are current-row conditional-on-X parameter intervals using the fit's
  selected covariance.
