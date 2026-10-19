# nbreg

Installed syntax:
`nbreg y [x1 x2 ...] [, robust noconstant]`

Fit unweighted **NB2** count regression: E[y|X]=mu=exp(X beta),
Var[y|X]=mu+alpha*mu^2, alpha=exp(lnalpha)>0, size=1/alpha.
This is not NB1 or an arbitrary variance-power model. An intercept is default;
no predictors fits an intercept-only NB2. `noconstant` requires predictors.

Example workflow:
- `use visits.csv`
- `summarize visits`
- `nbreg visits age income, robust`
- `estat margins`
- `predict nb_mean_visits, n`
- `predict nb_index, xb`
- `head 8`
- `export nb_visits_predictions.csv`

Sample and numerical contract:
- Numeric outcome/predictors, joint complete-finite cases, N>K+1 (K mean-model
  parameters plus dispersion), full-rank mean design. Loaded counts must be exact
  integers **0..100000**; no clipping/truncation. DECIMAL/wide integer identities
  are checked before f64 conversion; already-rounded DOUBLE digits are not recovered.
- The count bound limits per-iteration integer gamma-ratio/derivative recurrence
  work. Supported lnalpha is strictly **(-12,12)**. All-zero, Poisson-limit,
  boundary, rank, nonrepresentable variance and iteration failures are errors,
  not a silently substituted Poisson or clipped-alpha fit.
- Joint Newton with bounded backtracking/intermediate damping; final UNDAMPED
  observed information must be positive definite. Both absolute scaled score and
  full undamped step <=1e-10, at most100 outer Newton iterations per full/null
  fit. Each can evaluate additional candidates (up to20 damping attempts,
  50 backtracking trials each); the reported iteration count excludes candidates
  and restricted-fit work. Mean columns have unit norms; lnalpha scales by sqrt(N).
  Numerical admission is not an analytic existence guarantee.
- Parameter names are distinct, including intercept and reserved `lnalpha`.
  Failed new runtime fits clear stale inference, not data/labels/order.

Inference and interpretation:
- Model-based covariance is inverse **full JOINT observed information** for beta
  and lnalpha, including cross terms. Conditional GLM/Fisher covariance with alpha
  held fixed is not substituted. `robust` uses the same joint bread and row-score
  sandwich **N/(N-1)**, not HC1 or Pearson-dispersion scaling.
- Coefficients have asymptotic normal z/two-sided p/fixed95% CIs, not Student t.
  Zero variance gives unavailable z/p/CI, not exact significance/point intervals.
- lnalpha has its joint SE/normal interval; alpha SE is delta-method and its CI
  exponentiates the log interval, not alpha +/- z*SE. No normal alpha=0 or boundary
  LR test is supplied: zero dispersion is outside the interior parameterization.
- The nested NB null reestimates dispersion: intercept-only mean by default,
  fixed unit mean (beta=0) without intercept. Positive-DF model-based LR uses
  chi-square; robust LR p/reference are unavailable. Robust zero-slope Wald
  excludes lnalpha and has explicit rank/unavailability guards.
- NB residual deviance at fitted alpha and Pearson/(N-K-1) dispersion are
  descriptive, not formal specification/panel tests or covariance multipliers.
- `predict ..., n` is the current-row conditional mean, not a probability or
  integer draw; no clipping at one. Bare predict stays xb. `estat margins` uses
  retained X and full selected joint covariance for continuous column derivatives
  beta_j*mean(mu), conditional-X delta normal95, not discrete changes.

Not installed: weights/survey, clustered/panel NB2, offsets/exposure, NB1,
PMF/CDF prediction UI, factors/discrete changes, general count test/lincom,
interval prediction, formal alpha=0/specification tests or named count reports.
Legacy `cluster(var)` parses but execution rejects it. Independent-row robust
is **NOT panel-robust**; do not use it in place of person/household clustering.

See `help poisson`, `help predict`, `help estat`, and
`examples/mma20p1count/README.md` for exact sample/source and reproduction.
