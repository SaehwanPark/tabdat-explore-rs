# poisson

Installed syntax:
`poisson y [x1 x2 ...] [, offset(var) exposure(var) robust cluster(var) noconstant]`

Fit unweighted log-link count regression, E[y|X]=exp(X beta), using model-based
Poisson ML, independent-row robust QMLE or one-way clustered QMLE. `robust` and
`cluster(var)` are mutually exclusive. An intercept is default; no predictors
gives an intercept-only fit. `noconstant` requires predictors.

`offset(var)` adds the named numeric variable with fixed coefficient one:
mu=exp(X beta+var). `exposure(var)` instead adds its natural logarithm with fixed
coefficient one: mu=exp(X beta+ln(var)). The options are mutually exclusive,
neither is a weight or estimated parameter, and neither increases K or changes
covariance corrections. Missing/nonfinite values exclude rows jointly with
model/group cells; finite nonpositive exposure on an otherwise complete row is
an error. Predictions use current positive exposure values, while retained
effects hold the fitted log exposure fixed. See `examples/poisson-offset/workflow.td`
and the exposure contract in `examples/mma20p1count/exposure.md`.

Example workflow:
- `use visits.csv`
- `summarize visits`
- `poisson visits age income, robust`
- `estat overdispersion` — one-sided score test in the NB2 variance direction
- `estat margins`
- `predict mean_visits, n`
- `predict response_residual, residuals`
- `predict index, xb`
- `head 8`
- `export visits_predictions.csv`

For repeated observations with a real person ID, use
`poisson visits age income, cluster(person_id)` instead of `robust`, provided
independence across people is appropriate. See the audited RAND source/ID
preparation and executable `examples/mma20p1count/poisson_clustered.td`.
Person clustering does not address household/site or other cross-person dependence.

Sample and failure rules:
- Numeric outcome/predictors/optional offset or exposure; joint finite complete
  cases, N>K, full rank. A fixed term does not increase K or change the
  covariance correction.
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
- `test <linear restrictions>` uses the selected covariance and an asymptotic
  Wald chi-square reference. It tests retained mean coefficients, including
  affine restrictions; clustered tests have no few-cluster correction.
- `lincom <affine expression>` uses the same selected covariance for a linear
  combination of retained mean coefficients, with asymptotic standard-normal
  z inference and a 95% normal confidence interval. Residual DF is not supplied.
- Coefficient intervals are fixed normal 95%, not Student t. Zero selected
  variance yields unavailable z/p/CI, not exact significance or point intervals.
- Without fixed terms, the intercept null is the estimated overall mean and the
  no-intercept null is all beta=0,mean=1. With fixed terms, BOTH nulls retain the same
  offset or log exposure: intercept-only mu0_i=exp(fixed_i)*sum(y)/sum(exp(fixed)); without an
  intercept mu0_i=exp(fixed_i). Robust/clustered LR p/reference are unavailable;
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
  outcome or cluster ID; fixed-term fits also need the CURRENT numeric offset or
  positive exposure column. BOTH xb and n include it (xb=X beta+offset or
  X beta+ln(exposure)). Retained margins hold original fixed terms/groups fixed
  despite data edits; derivatives do not apply a chain rule through the fixed
  term even if its source variable is also a predictor.
- `predict residual, residuals` returns current numeric y minus the current
  conditional mean. Missing/nonfinite outcomes or model inputs produce NULL;
  finite outcomes must be exact counts in 0..2^53-1. It is a raw response
  residual, not Pearson, deviance or standardized, and does not use covariance.

`estat countprob [, max(10)]` compares observed and fitted count frequencies on
retained y/means (exact0..9 plus >=10 by default). It assumes the full Poisson
law, which robust/cluster QMLE covariance does NOT guarantee. No uncertainty or
formal goodness-of-fit test is supplied; see `help estat`.

`estat overdispersion` evaluates the Poisson score for the one-sided NB2
variance alternative `Var(Y|X)=mu+alpha*mu^2`, `alpha>0`, on the retained fit;
fixed offsets or log exposure remain in its fitted means. The selected model-based, independent-
row N/(N-1), or one-way cluster G/(G-1) score variance gives a signed asymptotic-
normal Z and one-sided p-value. Robust inference requires a correct conditional
mean; clustered inference additionally assumes independent groups and does not
guarantee reliable few-cluster inference. It does not fit NB2 or supply a
cluster-robust NB2 likelihood-ratio reference. See
`examples/nb2-overdispersion-score/README.md`.

Not installed for Poisson: weights/survey, multiway clustering,
cutoff-variable values above99, factors/discrete changes,
interval prediction or few-cluster bootstrap. Immutable `estimates store/table`
reports are supported; they cannot restore fits or establish current-data identity.
Clustered covariance is not FE/RE/random-effects or the full panel family.

NB2 is separately installed by `nbreg`; see `help nbreg` for its distinct
variance, full joint dispersion covariance and narrower numerical domain.
One-way NB2 clustering is separately installed with full joint group scores;
Poisson Fisher covariance must not be substituted for NB2 observed information.

See `help predict`, `help estat`, `examples/mma20p1count/README.md`, and the
structured result contracts in `docs/reference/command-results.md`.
