# poisson

Installed syntax:
`poisson y [x1 x2 ...] [, robust noconstant]`

Fit unweighted log-link count regression, E[y|X]=exp(X beta), using model-based
Poisson ML or independent-row robust QMLE. An intercept is included by default;
no predictors gives an intercept-only fit. `noconstant` requires predictors.

Example workflow:
- `use visits.csv`
- `summarize visits`
- `poisson visits age income, robust`
- `estat margins`
- `predict mean_visits, n`
- `predict index, xb`
- `head 8`
- `export visits_predictions.csv`

Sample and failure rules:
- Numeric outcome/predictors; joint finite complete cases, N>K, full rank.
- Outcome must be an exact nonnegative integer loaded value <=2^53-1. DECIMAL
  and wide integers are checked before DOUBLE conversion, not after rounding.
  Digits already rounded by a loader cannot be recovered. Invalid finite counts
  on complete rows fail; missing/nonfinite model cells exclude the row.
- An all-zero intercept model has no finite fit. Other boundary, rank, numerical
  or iteration failures are explicit; strict absolute normalized score AND full
  Newton step <=1e-10 can reject extreme finite designs/counts. This is not an
  analytic existence classifier.
- A new failed runtime fit clears stale estimation, not data/labels/order.

Inference and interpretation:
- Model-based covariance is inverse Poisson Fisher information, no N-K scaling;
  distributional inference assumes independent Poisson observations, variance=mean.
- `robust` is an independent-row score sandwich with N/(N-1), NOT OLS HC1,
  Pearson-dispersion scaling or clustered/panel inference. Coefficient intervals
  are fixed normal 95%, not Student t. Zero robust variance yields unavailable
  z/p/CI, not exact significance or point intervals.
- Intercept null is the estimated overall mean. Without an intercept the null is
  all beta=0, mean=1. Robust LR p/reference are unavailable; likelihood/McFadden
  are descriptive. Robust zero-slope Wald uses selected covariance/chi-square
  with explicit rank/unavailability guards. Intercept-only has no slope test.
- Deviance, Pearson chi-square and Pearson/(N-K) dispersion are descriptive,
  not a panel/specification test or automatic covariance correction.
- Means (`predict ..., n`) are not probabilities and are never clipped at one.
  Bare predict remains xb. `estat margins` reports retained-X continuous column
  derivatives with full conditional-X delta normal inference, not discrete changes.

Not installed for Poisson: weights/survey, cluster inference, exposure/offset,
count probability prediction, factors/discrete changes, general count test/lincom,
interval prediction or named Poisson `estimates store`. Legacy `cluster(var)`
syntax parses but execution rejects it explicitly. Do not substitute robust for
clustering of repeated person-years.

NB2 is separately installed by `nbreg`; see `help nbreg` for its distinct
variance, joint dispersion covariance and narrower numerical domain.

See `help predict`, `help estat`, `examples/mma20p1count/README.md`, and the
structured result contracts in `docs/reference/command-results.md`.
