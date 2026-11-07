# xtgee

`xtgee y xvars, family(gaussian) link(identity) corr(exchangeable) id(group)`

Fits a bounded, unweighted Gaussian identity-link population-averaged GEE with one
exchangeable within-group working correlation. All four options are required;
apart from the bounded Poisson/log case below, other families/links are not
supported. Other correlations, weights, offsets, factors and survey design are
not supported. No time variable is needed. Complete finite outcome,
predictor and group rows are used; singletons remain. Up to40,000 candidate
rows,32 numeric slopes,10,000 groups and20,000,000 row-parameter-squared units
are allowed. There must be more groups than parameters and at least one pair.

Scale is residual sum of squares/N. Correlation is the sum of within-group
ordered residual cross-products divided by scale times the number of ordered
pairs. Admissible negative correlation is supported. Nonpositive scale,
non-positive-definite working correlation, singular design or nonconvergence
fails rather than being clipped or replaced by a random-effects model.

Both the inverse-working-bread and **uncorrected** group-sandwich covariance
are reported separately, with normal95 coefficient inference and chi-square
slope Wald tests. No HC1/CR1, N-K or G-1 correction, likelihood or survey-design
interpretation is implied.

`predict mean`, `predict index, xb` and `predict residual, residuals` use the
owned marginal mean model and current predictor values; mean and index coincide
under the identity link. Residuals also require current y. Group IDs are not
needed for prediction; no posterior group effects or prediction uncertainty are
provided. A failed Gaussian GEE fit leaves the previous estimation state unchanged.

Immutable reports require an explicit covariance choice:
`estimates store pa, covariance(working)` or
`estimates store pa_robust, covariance(sandwich)`. Plain store is rejected because
both inference variants are retained, not implicitly selected. Table comparison
labels normal inference and the covariance; it supplies no GEE likelihood/R² or
fit restoration. The active GEE model/predictions are unchanged. See `help estimates`.

## Poisson log-link: balanced selected groups

`xtgee y xvars, family(poisson) link(log) corr(exchangeable) id(group)`

Fits a bounded count marginal mean exp(X beta). Uses the same joint-finite
sample and candidate resource caps as Gaussian GEE, but requires selected
integer counts0..100000, positive total count and **balanced selected groups of
common size T>=2**. Missingness may make selected groups unbalanced; such data
fail, not silently become complete firms. Zero-total groups remain; G must exceed
the intercept-plus-slopes count. Negative admissible rho is supported.

With Pearson residuals e=(y-mu)/sqrt(mu), legacy balanced normalized correlation
is (sum_g(sum_t e)^2-sum e^2)/((T-1)*sum e^2). Pearson RSS/N only normalizes
correlation: **working variance scale is fixed1**. Both inverse-expected-GEE-bread
working covariance and separate **G/(G-1) group-score sandwich** are reported,
with normal95/z and chi-square slope-Wald inference. No extra N-K correction or
G-1 t distribution. Sandwich interpretation requires independent groups/correct
mean; working inference additionally needs its variance/correlation assumptions.
Few-group reliability or causal interpretation is not established.

Uses an internal pooled Poisson start and at most100 full expected-Fisher steps,
fit index restricted to[-40,40]. Reports final relative RMS-coordinate step and
coordinatewise score normalized by one plus absolute whitened row contributions.
Degenerate/near-boundary correlation, roundoff-dominated Pearson/sandwich variance,
singular design, nonfinite updates or nonconvergence fail; no clipping, ridge,
pseudoinverse or fallback model. No joint likelihood, LR, R² or nuisance-rho SE.

`predict mean, n`, `predict index, xb`, `predict residual, residuals` use actual
owned GEE coefficients and current numeric predictors; residuals need current y.
Current IDs are not required. Missing/nonfinite predictors yield missing output;
unrepresentable mean/residual fails before adding a column. No group effects,
PMF/CDF, prediction intervals/std or saved draws. Named estimate snapshots and
covariance selectors are not supported for this family. A failed new Poisson
GEE fit clears stale estimation but preserves data/labels/stored reports; parser,
prediction and store errors preserve the current fit. Gaussian behavior is not
changed. Full original patents workflow validation is still in progress.
