# mlogit

Fit an unweighted multinomial logit with a shared person-level numeric design:

```text
mlogit mode income, baseoutcome(1)
predict beach_probability, pr outcome(1)
predict pier_index, xb outcome(2)
estat margins
```

Syntax: `mlogit <outcome> [<predictors>] [, baseoutcome(integer) noconstant]`.
An intercept is included unless `noconstant` is supplied. With no predictors,
only an intercept-only fit is allowed. The lowest retained category is the
default base. `baseoutcome()` must identify a category in the estimation sample.
Outcome codes must be exact integers within ±(2^53−1); text labels and factors
are not supported. Numeric value-label metadata is not converted into codes.

The estimation sample comprises rows with finite outcome and all predictors.
Domain checks occur after joint missing/nonfinite exclusion. No weights,
robust/cluster/survey covariance, factors, interactions, alternative-specific
regressors, conditional/nested logit or IIA tests are implemented here.
Explicit numeric transformed columns are treated as independent design columns.

Coefficients are relative log odds against the base, grouped by ascending
nonbase category, intercept then predictor order. The full joint inverse Fisher
information supplies asymptotic normal95 inference, with no N−K correction.
The omnibus LR uses asymptotic chi-square with `(J−1)*number_of_predictors`
degrees of freedom, excluding included intercepts. The null estimates category
shares with an intercept; with `noconstant` it gives equal probabilities. An
intercept-only fit has zero LR degrees of freedom and no LR p-value.

Bounds are 2–16 retained categories, N≤10,000, K≤32 including the intercept,
P=(J−1)K≤128, and N*P²≤20million. These limit the joint numerical work. Scaled
Newton fitting requires both the full undamped Newton step and averaged score
to converge. Collinearity, degenerate information, nonconvergence or separation
fail explicitly; a failed new fit clears the prior inference cache.

`predict newvar, pr outcome(code)` writes one category probability;
`predict newvar, xb outcome(code)` writes its base-relative index (zero for the
base). The category selector is required. Prediction uses current finite
predictors, not current outcomes or estimation-sample membership, and writes
NULL where a predictor is missing/nonfinite. Existing targets, absent retained
categories and incompatible predictor schemas fail without publishing a column.
Predictions have no intervals; retain the fit's joint covariance for inference.

`estat margins` reports continuous predictor-column average derivatives for
**all** categories on the retained estimation X, even after current-data edits.
Full-joint-covariance delta uncertainty is conditional on that X, not resampled-X
uncertainty. These are neither at-means effects, discrete changes, interaction
chain rules nor historical finite-difference/float-storage approximations.
An intercept-only model has no predictor effects. Named `estimates store/table`
reports and `lincom`/`test` are not supported for this family in this slice;
terminal fit/effect tables and structured JSON contain the results/covariance.
