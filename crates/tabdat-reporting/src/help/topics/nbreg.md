# nbreg

Installed syntax:
`nbreg y [x1 x2 ...] [, offset(var) exposure(var) robust cluster(var) noconstant]`

Fit unweighted **NB2** count regression: E[y|X]=mu=exp(X beta),
Var[y|X]=mu+alpha*mu^2, alpha=exp(lnalpha)>0, size=1/alpha.
This is not NB1 or an arbitrary variance-power model. An intercept is default;
no predictors fits an intercept-only NB2. `noconstant` requires predictors.
`robust` and one-way `cluster(var)` are mutually exclusive.

`offset(var)` adds a numeric fixed coefficient-one log-mean term, mu=exp(X beta+var).
`exposure(var)` adds its natural logarithm with fixed coefficient one,
mu=exp(X beta+ln(var)). The options are mutually exclusive; neither is a weight,
estimated parameter or addition to K/LR DF. Missing/nonfinite values join
model/group exclusions before finite count validation; finite nonpositive exposure
on a complete row is an error. Both full and restricted NB fits retain the same
fixed term and reestimate dispersion and any intercept. This is not a fixed-alpha
Poisson/GLM refit. See `examples/nb2-offset/workflow.td` and
`examples/mma20p1count/exposure.md`.

Example workflow:
- `use visits.csv`
- `summarize visits`
- `nbreg visits age income, robust`
- `estat margins`
- `nbreg visits age income`
- `estat countgof, max(10)`
- `predict nb_mean_visits, n`
- `predict response_residual, residuals`
- `predict pearson_residual, pearson`
- `predict deviance_residual, deviance`
- `predict nb_index, xb`
- `head 8`
- `export nb_visits_predictions.csv`

For repeated observations, use `nbreg visits age income, cluster(person_id)`
when independence ACROSS people is appropriate. This does not address household/
site/cross-person dependence, FE/RE or the full panel family. The original-person
RAND extension is `examples/mma20p1count/nb2_clustered.td`.

Sample and numerical contract:
- Numeric outcome/predictors/optional offset or exposure, joint complete-finite cases, N>K+1 (K mean-model
  parameters plus dispersion), full-rank mean design. Loaded counts must be exact
  integers **0..100000**; no clipping/truncation. DECIMAL/wide integer identities
  are checked before f64 conversion; already-rounded DOUBLE digits are not recovered.
- Numeric/text/boolean cluster keys require G>=2 retained groups. Joint model/ID
  SQL NULL/nonfinite exclusions precede invalid finite-count rejection. Wide
  integer/DECIMAL IDs are not rounded through f64; signed float zero is normalized.
  Actual loaded empty text is a group, subject to the loader's NULL conventions.
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
- `cluster(var)` uses the same FULL joint beta/lnalpha bread and sums full joint
  scores WITHIN groups BEFORE outer products, with **G/(G-1)** correction. No
  conditional/Fisher or row-score substitution, centering or Pearson rescaling.
  Independence ACROSS groups is required; no G-1 t/few-cluster guarantee.
- Coefficients have asymptotic normal z/two-sided p/fixed95% CIs, not Student t.
  Zero variance gives unavailable z/p/CI, not exact significance/point intervals.
- lnalpha has its joint SE/normal interval; alpha SE is delta-method and its CI
  exponentiates the log interval, not alpha +/- z*SE. A one-sided likelihood-ratio
  statistic compares the full NB2 likelihood with the Poisson likelihood on the
  same sample/design/offset. Its 50:50 chi-square(0)/chi-square(1) p-value is
  supplied only with model-based covariance; robust and clustered fits expose the
  statistic but leave its reference distribution and p-value unavailable. No
  normal-Wald alpha=0 test is supplied: zero dispersion is outside the interior
  parameterization.
- The nested NB null reestimates dispersion: intercept-only mean by default,
  fixed unit mean (beta=0) without intercept when no fixed term is specified.
  Offset/exposure nulls retain the same fixed term: jointly refit intercept and alpha when
  present; otherwise mu=exp(fixed term), refitting alpha only. No Poisson-null alpha
  or fixed-alpha conditional covariance is substituted. Positive-DF model-based LR uses
  chi-square; robust/clustered LR p/reference are unavailable. Robust/clustered
  zero-slope Wald excludes lnalpha and has explicit rank/unavailability guards:
  q>=G is unavailable, not a reduced-q test or pseudoinverse.
- `test <linear restrictions>` applies general affine restrictions to retained
  mean coefficients using the selected covariance and an asymptotic Wald
  chi-square reference. It uses the mean block of the full joint beta/lnalpha
  covariance; `lnalpha` itself is not a mean restriction. Clustered tests have
  no few-cluster correction.
- `lincom <affine expression>` applies the same mean-coefficient rule and full
  joint covariance block to a linear combination, with asymptotic standard-normal
  z inference, a two-sided p-value and a 95% normal confidence interval.
- NB residual deviance at fitted alpha and Pearson/(N-K-1) dispersion are
  descriptive, not formal specification/panel tests or covariance multipliers.
- `predict residual, residuals` returns current numeric y minus the current
  conditional mean. `predict pearson, pearson` returns the unscaled
  `(y-mu)/sqrt(mu+alpha*mu^2)` residual; `predict deviance, deviance` returns
  the signed square root of
  `2*(y*log(y/mu)-(y+1/alpha)*log((1+alpha*y)/(1+alpha*mu)))`, using the
  zero-count limit. Both use current rows with retained mean coefficients,
  alpha and current fixed terms. Missing/nonfinite inputs or undefined residuals
  produce NULL; finite outcomes must be exact counts in 0..100000 or prediction
  fails atomically. They do not use covariance, Pearson scale corrections,
  studentization or leverage. At mu=0, Pearson is undefined; deviance is zero for
  y=0 and undefined for positive y.
- `predict ..., n` is the current-row conditional mean, not a probability or
  integer draw; no clipping at one. Bare predict stays xb. `estat margins` uses
  retained X and full selected joint covariance for continuous column derivatives
  beta_j*mean(mu), conditional-X delta normal95, not discrete changes. Original
  groups/V survive data edits/drop ID; current predictions need mean predictors
  and the CURRENT named offset or positive exposure when specified, not outcome/
  cluster ID. BOTH xb and n include the fixed term (xb=X beta+offset or
  X beta+ln(exposure)). Retained effects hold original fixed terms in derivatives/
  uncertainty, even if a source is also a predictor; no chain rule through that
  source is implied. Bad predictions/options preserve data/model state.

`estat countprob [, max(10)]` compares observed and mean fitted NB2 probabilities
using retained y/means/alpha (exact0..9 plus >=10 by default). Full conditional NB2
law is an assumption, not guaranteed by robust/cluster covariance. This is a
descriptive report without uncertainty/formal goodness-of-fit test; see `help estat`.

`estat countgof [, max(10)]` is a separate score-adjusted grouped goodness-of-
fit test. It supports either an unclustered model-based NB2 fit or a one-way
clustered fit, both unweighted and without an offset or exposure. The unclustered test
reports N*uncentered-R2; the clustered test uses a cluster-summed moment
covariance with G/(G-1) correction. The clustered chi-square reference requires
independent clusters and many clusters; there is no few-cluster guarantee.
Both modes use the retained 0..9 and >=10 cells and chi-square(10) by default.
See `help estat` for the assumptions, rank failure and degrees-of-freedom
difference from the November 2006 author program.

Immutable `estimates store/table/drop` reports retain the full joint beta/lnalpha
covariance, `lnalpha` dispersion, offset/exposure identity, selected covariance correction,
refitted-dispersion null and the Poisson-boundary LR. They do not restore fits.
See `help estimates`.

Not installed: weights/survey/multiway/few-cluster methods, FE/RE/panel NB2,
NB1, cutoff-variable values above99, factors/discrete changes,
interval prediction or other formal specification tests. Independent-row
robust is **NOT panel-robust**; choose
actual cluster IDs and defend independence across the selected groups.

See `help poisson`, `help predict`, `help estat`, and
`examples/mma20p1count/README.md` for exact sample/source and reproduction.
