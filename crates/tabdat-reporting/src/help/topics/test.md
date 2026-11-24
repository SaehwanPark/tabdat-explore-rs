# test

How to invoke:
`test <coefficient names or affine restrictions>`

What it does:
Test linear restrictions on the retained fit. After a supported linear
regression, the command reports a Wald F statistic, the number of restrictions
as numerator degrees of freedom, residual denominator degrees of freedom, and
an upper-tail p-value. After logit or probit, it uses the covariance selected
for that fit and reports an asymptotic Wald chi-square statistic with degrees
of freedom equal to the number of restrictions; residual degrees of freedom are
not supplied. A bare coefficient name tests it against zero.
Native classical within FE supports `_cons` common-intercept and slope
restrictions with full joint covariance/denominator DF N-G-p. Unit-cluster FE
supports slopes only/G-1 DF; common and individual unit-effect inference are
withheld. No individual unit-effect restrictions are added.
Restrictions use the retained fit, not current-row refitting. Binary clustered
tests use an asymptotic chi-square reference and do not provide a few-cluster
correction or guarantee.

Examples:
- `test x`
- `test x1 x2`
- `test x1 = x2`
- `test (x1 = x2) (intercept = 1)`

An applicable prior fit is required. Linear-model F tests require positive
residual degrees of freedom; binary chi-square tests do not use a residual
denominator. Unknown coefficients, nonlinear or collinear/singular restrictions
fail without changing the fitted model or active data. A new failed regression
clears the prior model. This does not implement tests for parser-only estimator families.
