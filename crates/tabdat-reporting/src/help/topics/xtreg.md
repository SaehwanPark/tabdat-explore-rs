# xtreg

Installed bounded syntax:
`xtreg y xvars, fe|re|mle id(unit) time(year)`
Both FE and RE accept `robust` or `cluster(unit)`; robust clusters on the declared unit.

`fe` fits an unweighted one-way within fixed-effects linear model, absorbing a separate
intercept for each unit. At least one numeric slope predictor is required. Unlike
`regress ..., robust`, **robust here means unit clustering**, not independent-row
HC1. `cluster()` must name the same variable as `id()`.

Example, after loading numeric y/x with person/year keys:
- `xtreg y x, fe id(person) time(year) cluster(person)`
- `lincom x`
- `test x = 0`
- `predict fitted, xbu`
- `predict unit_deviation, u`
- `predict error, residuals`
- `summarize fitted unit_deviation error`
- `export within_predictions.csv`

Within FE keys and sample:
- Both explicit key options are required for execution. There is no retained
  panel declaration: parser-accepted `panel`/`xtdata` do not install one.
- All raw unit/time keys must be nonmissing/finite, with unique unit-time pairs,
  even on rows subsequently excluded for missing model values. Time is numeric
  integer; integer/decimal keys preserve exact wide identity. DOUBLE time must
  lie within ±(2^53-1). Unit identifiers may be numeric, text or boolean, not BLOB.
- Only then select jointly finite y/X rows and compute unit centers. Gaps,
  unbalanced and interleaved input are allowed; no lags are constructed. Source
  order and data are preserved. Singletons remain in N/G and conditional fits,
  but add no within information. Insufficient within rank or nonpositive residual
  DF causes a fit failure.

Inference and reporting:
- N observations, G retained units and p slopes: classical variance uses
  RSS/(N-G-p), Student-t/F DF N-G-p. Unit clustering uses grouped within scores,
  multiplier G/(G-1)*(N-1)/(N-p) (plm Arellano sss/nested-unit convention), and
  Student-t/F inference DF G-1. Clustered fits require at least two units;
  few-cluster calibration and independence across units remain assumptions.
- Covariance and `lincom`/`test` contain **slopes only**. The normalized common
  intercept is a descriptive point: mean(y)-mean(X)beta on the estimation sample.
  Unit deviations have estimation-observation-weighted mean zero. There is no
  common/unit-intercept inference or implicit intercept constraint.
- Report within R² and conditional level-fit R² (not Stata overall/between R²),
  RSS/root MSE, sample/absorbed DF and group sizes. No panel likelihood, adjusted
  within R², Hausman or serial-correlation test is supplied. Residual summaries
  are descriptive. FE removes additive time-invariant unit effects, not all
  endogeneity; identification/strict-exogeneity assumptions still matter.
- CLI-v1 `FixedEffectsResult` exposes bounded fit/inference metadata, not stored
  observation/effect vectors. Use prediction and export for usable values.

Current-data prediction (retained slopes and unit centers, never refitted):
- `xb` (also bare predict): common normalized intercept + current X beta; no ID
  needed. This is not the conditional fitted level or an unseen-unit forecast.
- `xbu`: retained total unit intercept + current X beta.
- `u`: retained normalized unit deviation; no current y/X/time required.
- `residuals`: current y minus xbu.
- Conditional modes require the fitted ID name and exact logical type. Missing,
  nonfinite or unestimated IDs yield NULL, **not zero effects**. Missing/nonfinite
  required model cells or unrepresentable calculations also yield NULL. A unit
  excluded entirely from estimation has no retained effect. Time is not required
  for prediction. Changing current values/order does not change retained effects.
- Missing required columns, ID type mismatch, unsupported modes/intervals and
  existing targets fail without changing data/fit. Restore columns/types or
  deliberately refit; choose a fresh target. A failed new runtime FE fit clears
  stale estimation without modifying data; syntax errors preserve the old fit.

Not installed: general between/two-way effects, weights, factors/interactions,
if filters, arbitrary cluster variables, FE named report snapshots, persistent
panel state, bootstrap, panel-specific diagnostics or prediction intervals.
Legacy bare `xtreg ..., fe` remains parseable but needs explicit keys to execute;
`re` without explicit keys remains unsupported. See examples/mma21p1panel/README.md for the
independently audited Table21.2 Within slope/conditional-prediction portion, not
whole-table/chapter coverage.

Balanced random-effects GLS:
- `xtreg y xvars, re id(unit) time(year)` audits the same raw keys, then selects
  joint-finite y/X cases. Retained unit counts must be equal T>=2; G>p+1 and
  both within and between designs must identify every slope. Missing cases may
  remove an entire unit; any remaining imbalance fails. Interleaving and gaps
  are allowed; equal counts do not imply a common calendar or constructed lags.
- Swamy-Arora uses sigma_e²=within RSS/(N-G-p), raw sigma_u²=between
  RSS/(G-p-1)-sigma_e²/T, truncates negative sigma_u² to zero, and reports the
  raw value. Zero/nonfinite sigma_e² or degenerate transformations fail.
- Quasi-demean y, X and the intercept with theta=1-sqrt(sigma_e²/(sigma_e²+T*sigma_u²)).
  Model-based covariance uses transformed RSS/(N-p-1) times inverse transformed
  crossproduct, matching balanced R/plm. It does not substitute sigma_e² as the
  final scale. Report normal z/p/95% intervals and the joint zero-slope
  chi-square(p) Wald test. General test/lincom restrictions are not installed.
- Assumptions include exogeneity of unit effects relative to regressors,
  independent units. Model-based covariance relies on the specified within-unit
  error-components covariance; unit-cluster covariance allows arbitrary within-unit
  dependence but supplies no few-unit guarantee. No causal identification
  follows from fitting RE. No panel likelihood or within/between/overall R² is
  supplied. `RandomEffectsResult` is distinct from OLS and FE JSON results.
- `predict fitted, xb` (or bare predict) uses retained intercept/slopes on current
  X, without requiring unit/time columns. `residuals` gives current y minus
  population xb, including the unit component; it is not a conditional residual.
  Missing/nonfinite required cells or arithmetic yield NULL. Existing targets,
  missing/non-numeric required columns or unsupported modes fail atomically.
- Failed explicit-key RE fit attempts clear the old fit while preserving data;
  parser errors and legacy bare RE unsupported errors preserve the old fit.
  Unbalanced variance estimation, unit effects/conditional
  predictions, named RE reports and Hausman remain uninstalled.

RE unit-cluster inference:
- `xtreg y xvars, re id(unit) time(year) cluster(unit)` or `robust` uses the same
  sample, Swamy-Arora components, theta, coefficients and population predictions.
  It sandwiches grouped quasi-demeaned scores with multiplier
  G/(G-1)*(N-1)/(N-K), K=p+1 including the transformed intercept (R/plm
  Arellano `sss`). The reported residual DF N-K does not select t inference.
- Coefficient/95% interval inference stays normal and the joint slope Wald
  reference stays asymptotic chi-square(p). Singular slope covariance makes the
  Wald statistic/p unavailable with a reason; requested p is not reduced and
  valid marginal SEs/predictions remain usable. Zero/nonfinite marginal variances
  remain numerical failures. No pseudoinverse or automatic covariance fallback.
- Cluster IDs must be the declared unit; other variables are unsupported.
  This assumes independent units, does not remove endogeneity of unit effects,
  and is not household/survey/multiway inference or the author's bootstrap.

Balanced Gaussian marginal ML:
- `xtreg y xvars, mle id(unit) time(year)` uses the same raw-key audit,
  joint-finite sample and population `xb`/`residuals` prediction rules. Requires
  equal retained counts T>=2, G>p+1, full within rank, at least one slope and
  no weights. Time-invariant predictors, robust/cluster ML, general restrictions
  and named RE reports are unsupported. `mle` is exclusive of `fe`/`re`.
- Gaussian marginal likelihood integrates the random intercept; this is ML,
  not REML. Unit effects must be exogenous to X and units independent, with the
  specified Gaussian error-components covariance. Interior inference uses the beta block
  of the inverse **joint observed information** for beta, log(sigma_e²) and
  log(sigma_e²+T*sigma_u²), without a residual-DF correction. It differs from
  conditional GLS covariance. Coefficients use normal z/p/95% intervals and a
  chi-square(p) slope Wald test; transformed residual DF is descriptive only.
- The bounded profile searches eta=log(1+T*sigma_u²/sigma_e²) in [0,24], on a
  0.5 grid, polishes positive-to-negative score crossings by bisection and selects
  the highest detected likelihood including eta=0. It does not guarantee the
  global maximum for arbitrary designs. An unresolved upper tail, nonstationary
  interior fit, nonpositive joint information or degenerate variance fails
  explicitly, clearing stale fit and preserving data. A selected zero-unit-variance
  optimum uses constrained information as below; there is no automatic GLS fallback.
- The intercept-only null re-estimates both variance components on the identical
  sample, allowing zero unit variance. Slope LR=2*(full LL minus null LL) uses
  chi-square(p) only when both fitted unit variances are interior; a boundary reference is withheld
  with a reason. This tests slopes, not the presence of random effects or
  exogeneity. Do not compare level and differenced likelihoods as an LR test.
- CLI-v1 retains `RandomEffectsResult` with estimator
  `balanced_gaussian_random_effects_ml`, covariance convention
  `joint_observed_information`, covariance label `observed_information`, top-level marginal `log_likelihood` and `ml`
  metadata containing the null/LR, profile score/eta, and full joint covariance
  with explicit parameter order. Existing GLS/cluster JSON fields are preserved.
  The internal `gls` fit stores the quasi-demeaned regression even for ML;
  transformed root MSE/RSS are not the ML variance estimate or population residuals.
  `raw_sigma_u_squared` equals the positive fitted variance for this interior fit.

Zero-unit-variance ML:
- When eta=0 wins the detected profile comparison, require its inward score to
  be nonpositive within the stated numerical tolerance (1e-9 per observation).
  Then sigma_u²=theta=rho=0; coefficients equal pooled Gaussian ML. Inference
  uses RSS/N, **not** the OLS RSS/(N-K) correction or OLS Student-t reference.
- Restrict log(sigma_e²+T*sigma_u²)=log(sigma_e²) before inverting observed
  information. Its active parameters are beta and log(sigma_e²); the full
  two-variance inverse is not used. Normal coefficient intervals/slope Wald are
  asymptotic Gaussian-model inference, with no finite-sample/post-selection or
  few-unit guarantee. No regular SE, interval or test of sigma_u² is provided.
- `ml.unit_variance_boundary` is explicit in CLI-v1 JSON; true selects
  `constrained_observed_information` and K+1 joint parameter names/covariance.
  Interior fits retain K+2 names and `joint_observed_information`. The covariance
  label stays `observed_information`. Any full/null boundary withholds ordinary
  slope LR p-values and supplies a reason. Population predictions and data/state
  semantics stay the same. The strict Rust `fit_random_effects_ml_interior`
  entry point still rejects zero unit variance; the ordinary ML workflow accepts it.
