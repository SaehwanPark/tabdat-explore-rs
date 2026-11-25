# lincom

How to invoke:
`lincom <affine expression>`

What it does:
Estimate an affine combination of coefficients from the last supported fit.
Linear estimators use their covariance-based standard error, Student-t
statistic, two-sided p-value, residual degrees of freedom and 95% confidence
interval.
After Poisson or NB2, use the selected covariance with asymptotic standard-normal
z inference, a two-sided p-value and a 95% normal confidence interval;
`df_residual` is null. NB2 uses the mean block of the full joint beta/lnalpha
covariance, and `lnalpha` is not a mean coefficient.
After logit or probit, use the selected model-based, row-robust or one-way
clustered covariance with asymptotic standard-normal z inference and no residual
degrees of freedom. The combination is on the fitted linear index: log odds for
logit and the latent probit index for probit. It does not calculate odds ratios,
probabilities, contrasts or marginal effects; use `estat margins` for
probability-scale effects.
Use the displayed coefficient names; for regress/2SLS the constant is named
`intercept`. Native classical within FE additionally exposes its normalized
common intercept as `_cons`, including full intercept–slope covariance and
N-G-p DF. Unit-cluster FE retains slope-only inference/G-1 DF; common inference
is withheld, not replaced by a classical SE. See help xtreg; current-data changes do not refit these restrictions.

Examples:
- `lincom x`
- `lincom x1 - x2`
- `lincom intercept + 2 * x`
- `lincom 2 + x`
- `lincom _cons + 2.5 * lnwg` (after classical FE)

A prior successful supported fit is required. Nonlinear coefficient products,
unknown coefficient names and invalid expressions fail without changing the
fitted model or active data. A new failed regression clears the prior model.
Undefined inference (for example zero standard error) displays as `NA`.
This does not implement inference for parser-only estimator families.
