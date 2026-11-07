# xtgee

`xtgee y xvars, family(gaussian) link(identity) corr(exchangeable) id(group)`

Fits a bounded, unweighted Gaussian identity-link population-averaged GEE with one
exchangeable within-group working correlation. All four options are required;
other families, links, correlations, weights, offsets, factors and survey design
are not supported. No time variable is needed. Complete finite outcome,
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
provided. A failed GEE fit leaves the previous estimation state unchanged.

Immutable reports require an explicit covariance choice:
`estimates store pa, covariance(working)` or
`estimates store pa_robust, covariance(sandwich)`. Plain store is rejected because
both inference variants are retained, not implicitly selected. Table comparison
labels normal inference and the covariance; it supplies no GEE likelihood/R² or
fit restoration. The active GEE model/predictions are unchanged. See `help estimates`.
