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

Random-effects GLS (balanced and unbalanced):
- `xtreg y xvars, re id(unit) time(year)` audits the same raw keys, then selects
  joint-finite y/X cases. Both designs must identify every slope, G>p+1, and
  transformed residual degrees of freedom must be positive. Interleaving and
  calendar gaps are allowed; no lags are constructed.
- Balanced Swamy-Arora requires equal T>=2, uses sigma_e²=within RSS/(N-G-p),
  and raw sigma_u²=between RSS/(G-p-1)-sigma_e²/T. Unbalanced retained counts
  use weighted between regression with weights T_i and the trace-corrected raw
  component `(sum_i T_i ebar_i² - (G-p-1) sigma_e²) /
  (N - tr((Z'WZ)^-1 Z'W²Z))`, with W=diag(T_i) and Z containing the intercept
  and unit means.
  Negative unit variance is clipped at zero and the raw value is retained.
  Unbalanced singletons are allowed when rank and residual degrees of freedom
  suffice. Zero/nonfinite sigma_e² or degenerate transformations fail.
- Quasi-demean y and X by unit-specific theta_i=1-sqrt(sigma_e² /
  (sigma_e²+T_i*sigma_u²)); transform the intercept by 1-theta_i.
  Model-based covariance uses transformed RSS/(N-p-1) times inverse transformed
  crossproduct, matching balanced R/plm. It does not substitute sigma_e² as the
  final scale. Unbalanced results retain group-size and theta ranges; displayed
  theta denotes theta at minimum T. Report normal z/p/95% intervals and the joint zero-slope
  chi-square(p) Wald test. General test/lincom restrictions are not installed.
- Assumptions include exogeneity of unit effects relative to regressors,
  independent units. Model-based covariance relies on the specified within-unit
  error-components covariance; unit-cluster covariance allows arbitrary within-unit
  dependence but supplies no few-unit guarantee. No causal identification
  follows from fitting RE. No panel likelihood or within/between/overall R² is
  supplied. `RandomEffectsResult` is distinct from OLS and FE JSON results.
- `robust` aliases `cluster(id)`. The unit-score sandwich uses the transformed
  design, multiplier G/(G-1)*(N-1)/(N-K), and asymptotic normal/chi-square
  inference; this is not few-cluster t/F inference. Unbalanced GLS is reported
  as `UnbalancedRandomEffectsGlsResult`, distinct from Gaussian ML.
- `predict fitted, xb` (or bare predict) uses retained intercept/slopes on current
  X, without requiring unit/time columns. `residuals` gives current y minus
  population xb, including the unit component; it is not a conditional residual.
  Missing/nonfinite required cells or arithmetic yield NULL. Existing targets,
  missing/non-numeric required columns or unsupported modes fail atomically.
- Failed explicit-key RE fit attempts clear the old fit while preserving data;
  parser errors and legacy bare RE unsupported errors preserve the old fit.
  Unit effects/conditional predictions, named RE reports and Hausman remain
  uninstalled.

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

Gaussian marginal ML:
- `xtreg y xvars, mle id(unit) time(year)` uses the same raw-key audit,
  joint-finite sample and population `xb`/`residuals` prediction rules. Requires
  G>p+1, positive within residual DF, full within rank, at least one slope and
  no weights. Unequal retained counts and singletons are allowed. Time-invariant predictors, robust/cluster ML, general restrictions
  and named RE reports are unsupported. `mle` is exclusive of `fe`/`re`.
- Gaussian marginal likelihood integrates the random intercept; this is ML,
  not REML. Unit effects must be exogenous to X and units independent, with the
  specified Gaussian error-components covariance. Interior inference uses the beta block
  of the inverse **joint observed information** for beta, log(sigma_e²) and
  log(sigma_e²+T*sigma_u²), without a residual-DF correction. It differs from
  conditional GLS covariance. Coefficients use normal z/p/95% intervals and a
  chi-square(p) slope Wald test; transformed residual DF is descriptive only.
- `ml.variance_component_inference` reports observed-information uncertainty for
  variance components. Standard errors use the delta method and 95% intervals
  exponentiate normal-Wald limits on log variance, so positive interior intervals
  remain nonnegative. At the zero-unit-variance boundary, its standard error and
  interval are null with an explicit reason; the constrained component has no
  regular normal-Wald inference. These are model-based intervals, not profile
  intervals or robust ML inference.
- `ml.unit_variance_test` tests `sigma_u² = 0` against `sigma_u² > 0` by a
  one-sided profile likelihood ratio. Its asymptotic reference is the 50:50
  mixture of chi-square(0) and chi-square(1), for the single variance component
  constrained at the boundary with residual variance interior. A boundary-selected
  fit has LR=0 and p=1. This is a model-based boundary test, not ordinary
  chi-square(1) or finite-sample inference.
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

Unbalanced Gaussian ML reporting:
- Each unit uses its retained T_g in quasi-demeaning and the marginal determinant.
  The profile uses eta=log(1+T_min*sigma_u²/sigma_e²) with the same bounded search.
  Interior joint information uses beta, log(sigma_e²), log(sigma_u²). The null
  re-estimates its variance-weighted intercept and variances on the same sample.
  The zero-unit boundary retains beta/log(sigma_e²) information and LR withholding.
- CLI-v1 uses `UnbalancedRandomEffectsMlResult` and estimator
  `unbalanced_gaussian_random_effects_ml`. `balanced` is false; common `group_size`
  and `theta` are null. Minimum/maximum group sizes, theta_min/theta_max,
  theta_at_minimum_group_size and eta_reference_group_size identify actual counts.
  Balanced JSON retains its existing result type and parameter names.
- Rust result fields remain compatible: group_size=0 denotes unequal ML counts,
  and theta is the value at the minimum count. Use group_size_range()/theta_range()
  for retained-sample ranges even after changing current data. The strict
  fit_random_effects_ml_interior entry point still requires balanced T>=2.
