# nbreg

Installed syntax:
`nbreg y [x1 x2 ...] [, robust cluster(var) noconstant]`

Fit unweighted **NB2** count regression: E[y|X]=mu=exp(X beta),
Var[y|X]=mu+alpha*mu^2, alpha=exp(lnalpha)>0, size=1/alpha.
This is not NB1 or an arbitrary variance-power model. An intercept is default;
no predictors fits an intercept-only NB2. `noconstant` requires predictors.
`robust` and one-way `cluster(var)` are mutually exclusive.

Example workflow:
- `use visits.csv`
- `summarize visits`
- `nbreg visits age income, robust`
- `estat margins`
- `predict nb_mean_visits, n`
- `predict nb_index, xb`
- `head 8`
- `export nb_visits_predictions.csv`

For repeated observations, use `nbreg visits age income, cluster(person_id)`
when independence ACROSS people is appropriate. This does not address household/
site/cross-person dependence, FE/RE or the full panel family. The original-person
RAND extension is `examples/mma20p1count/nb2_clustered.td`.

Sample and numerical contract:
- Numeric outcome/predictors, joint complete-finite cases, N>K+1 (K mean-model
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
  exponentiates the log interval, not alpha +/- z*SE. No normal alpha=0 or boundary
  LR test is supplied: zero dispersion is outside the interior parameterization.
- The nested NB null reestimates dispersion: intercept-only mean by default,
  fixed unit mean (beta=0) without intercept. Positive-DF model-based LR uses
  chi-square; robust/clustered LR p/reference are unavailable. Robust/clustered
  zero-slope Wald excludes lnalpha and has explicit rank/unavailability guards:
  q>=G is unavailable, not a reduced-q test or pseudoinverse.
- NB residual deviance at fitted alpha and Pearson/(N-K-1) dispersion are
  descriptive, not formal specification/panel tests or covariance multipliers.
- `predict ..., n` is the current-row conditional mean, not a probability or
  integer draw; no clipping at one. Bare predict stays xb. `estat margins` uses
  retained X and full selected joint covariance for continuous column derivatives
  beta_j*mean(mu), conditional-X delta normal95, not discrete changes. Original
  groups/V survive data edits/drop ID; current predictions need mean predictors
  only, not outcome/cluster ID. Bad predictions/options preserve data/model state.

Not installed: weights/survey/multiway/few-cluster methods, FE/RE/panel NB2,
offsets/exposure, NB1, PMF/CDF prediction UI, factors/discrete changes, general
count test/lincom, interval prediction, formal alpha=0/specification tests or
named count reports. Independent-row robust is **NOT panel-robust**; choose
actual cluster IDs and defend independence across the selected groups.

See `help poisson`, `help predict`, `help estat`, and
`examples/mma20p1count/README.md` for exact sample/source and reproduction.
