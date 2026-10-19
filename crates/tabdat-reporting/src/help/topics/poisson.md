# poisson

Installed syntax:
`poisson y [x1 x2 ...] [, robust cluster(var) noconstant]`

Fit unweighted log-link count regression, E[y|X]=exp(X beta), using model-based
Poisson ML, independent-row robust QMLE or one-way clustered QMLE. `robust` and
`cluster(var)` are mutually exclusive. An intercept is default; no predictors
gives an intercept-only fit. `noconstant` requires predictors.

Example workflow:
- `use visits.csv`
- `summarize visits`
- `poisson visits age income, robust`
- `estat margins`
- `predict mean_visits, n`
- `predict index, xb`
- `head 8`
- `export visits_predictions.csv`

For repeated observations with a real person ID, use
`poisson visits age income, cluster(person_id)` instead of `robust`, provided
independence across people is appropriate. See the audited RAND source/ID
preparation and executable `examples/mma20p1count/poisson_clustered.td`.
Person clustering does not address household/site or other cross-person dependence.

Sample and failure rules:
- Numeric outcome/predictors; joint finite complete cases, N>K, full rank.
- Outcome must be an exact nonnegative integer loaded value <=2^53-1. DECIMAL
  and wide integers are checked before DOUBLE conversion, not after rounding.
  Digits already rounded by a loader cannot be recovered. Invalid finite counts
  on complete rows fail; missing/nonfinite model cells exclude the row.
- Cluster IDs may be numeric/text/boolean, with at least two retained groups.
  SQL NULL/nonfinite numeric IDs exclude rows BEFORE finite count rejection;
  integer/DECIMAL identity is not rounded through f64. Actual loaded empty text
  is a group, not NULL; CSV empty-cell loading rules still apply.
- An all-zero intercept model has no finite fit. Other boundary, rank, numerical
  or iteration failures are explicit; strict absolute normalized score AND full
  Newton step <=1e-10 can reject extreme finite designs/counts. At most100 outer
  iterations,50 backtracking candidates each; the count excludes candidates.
  This is not an analytic existence classifier.
- A new failed runtime fit clears stale estimation, not data/labels/order.

Inference and interpretation:
- Model-based covariance is inverse Poisson Fisher information, no N-K scaling;
  distributional inference assumes independent Poisson observations, variance=mean.
- `robust` is an independent-row score sandwich with N/(N-1), NOT OLS HC1,
  Pearson-dispersion scaling or clustered/panel inference.
- `cluster(var)` sums scores WITHIN each group BEFORE their outer products and
  applies G/(G-1), using the same converged inverse information. It allows
  within-group dependence but assumes independence ACROSS groups. No N/(N-K),
  G-1 Student t or automatic few-cluster correction is substituted.
- Coefficient intervals are fixed normal 95%, not Student t. Zero selected
  variance yields unavailable z/p/CI, not exact significance or point intervals.
- Intercept null is the estimated overall mean. Without an intercept the null
  is all beta=0,mean=1. Robust/clustered LR p/reference are unavailable;
  likelihood/McFadden are descriptive. Robust/clustered zero-slope Wald uses
  selected covariance/chi-square with explicit rank/unavailability guards. With
  q slopes and G clusters, q>=G is unavailable, not a reduced-q test.
  Intercept-only has no slope test. Few clusters can give unreliable inference.
- Deviance, Pearson chi-square and Pearson/(N-K) dispersion are descriptive,
  not a panel/specification test or automatic covariance correction.
- Means (`predict ..., n`) are not probabilities and are never clipped at one.
  Bare predict remains xb. `estat margins` reports retained-X continuous column
  derivatives with full selected-covariance conditional-X delta normal inference,
  not discrete changes. Current predictions need mean predictors only, not the
  outcome or cluster ID; retained margins keep original groups despite data edits.

Not installed for Poisson: weights/survey, multiway clustering, exposure/offset,
count probability prediction, factors/discrete changes, general count test/lincom,
interval prediction, few-cluster bootstrap or named Poisson `estimates store`.
Clustered covariance is not FE/RE/random-effects or the full panel family.

NB2 is separately installed by `nbreg`; see `help nbreg` for its distinct
variance, joint dispersion covariance and narrower numerical domain. NB2
clustering is not installed by this Poisson extension.

See `help predict`, `help estat`, `examples/mma20p1count/README.md`, and the
structured result contracts in `docs/reference/command-results.md`.
