# Linear combinations after Poisson and NB2

## Intent

Add retained-fit `lincom <affine expression>` after Poisson and NB2. Use the
selected covariance and report the existing `LincomResult` fields with
asymptotic standard-normal inference: two-sided p-value and 95% confidence
interval. `df_residual` is null. NB2 combinations are restricted to mean
coefficients and use the mean block of the full joint beta/lnalpha covariance.
`lnalpha` itself is not a mean coefficient. Current-data edits do not refit the
estimate. Linear Student-t behavior and logit/probit support boundaries remain
unchanged.

Zero-standard-error handling preserves the existing result convention: an
unavailable statistic/p-value and a point confidence interval at the estimate.
Unknown coefficients, invalid expressions or numerical failures must not mutate
the fitted model or active data.

## Verification plan

- Compare coefficient combinations and selected-covariance standard errors,
  z statistics, normal tails and intervals with independent R references for
  Poisson/NB2 and model-based, independent-row robust and one-way cluster fits.
- Replay combinations on the exact MMA §20.3 Table20.5 cohort from the existing
  independently validated coefficient/covariance references.
- Exercise affine constants, multi-coefficient expressions, retained-fit reuse,
  `df_residual: null`, text/JSON reporting, unknown/singular failures and
  unchanged linear Student-t behavior.

This is count post-estimation support only. It does not add alpha combinations,
few-cluster corrections, formal NB specification tests or whole-book coverage.
