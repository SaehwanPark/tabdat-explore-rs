# lincom

How to invoke:
`lincom <affine expression>`

What it does:
Estimate an affine combination of coefficients from the last supported linear
regression, with its covariance-based standard error, Student-t statistic,
two-sided p-value, residual degrees of freedom and 95% confidence interval.
Use the displayed coefficient names; for regress/2SLS the constant is named
`intercept`. After native within FE, only slopes are parameters: the normalized
common intercept has no inference. Classical FE uses N-G-p DF, unit-cluster FE
uses G-1 DF. See help xtreg; current-data changes do not refit these restrictions.

Examples:
- `lincom x`
- `lincom x1 - x2`
- `lincom intercept + 2 * x`
- `lincom 2 + x`

A prior successful regression is required. Nonlinear coefficient products,
unknown coefficient names and invalid expressions fail without changing the
fitted model or active data. A new failed regression clears the prior model.
Undefined inference (for example zero standard error) displays as `NA`.
This does not implement inference for parser-only estimator families.
