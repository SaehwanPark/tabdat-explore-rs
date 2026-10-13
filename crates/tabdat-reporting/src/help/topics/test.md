# test

How to invoke:
`test <coefficient names or affine restrictions>`

What it does:
Test linear restrictions on the last supported linear regression using its
stored covariance. The current runtime reports a Wald F statistic, the number
of restrictions as numerator degrees of freedom, residual denominator degrees
of freedom, and an upper-tail p-value. A bare coefficient name tests it against zero.

Examples:
- `test x`
- `test x1 x2`
- `test x1 = x2`
- `test (x1 = x2) (intercept = 1)`

A prior successful regression and positive residual degrees of freedom are
required. Unknown coefficients, nonlinear or collinear/singular restrictions
fail without changing the fitted model or active data. A new failed regression
clears the prior model. This does not implement tests for parser-only estimator families.
