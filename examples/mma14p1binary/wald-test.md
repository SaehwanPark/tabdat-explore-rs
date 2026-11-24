# General Wald restrictions after logit and probit

## Scope and contract

This method extension follows the validated §14.2 fishing-mode analysis in
[`README.md`](README.md). `test <linear restrictions>` will operate on the
retained logit or probit fit, using its model-based, robust or one-way clustered
coefficient covariance. The result is an asymptotic Wald chi-square test with
`df` equal to the number of restrictions, `df_residual: null`, and
`is_chi2: true`. It does not use an OLS residual-F reference. The existing
`TestResult` JSON fields stay unchanged.

The test uses the owned coefficients and covariance from the fit. Editing the
current dataset after estimation must not refit the model or change the test.
Unknown parameters, empty or malformed restrictions, and collinear/singular
restriction covariance are explicit failures that leave fit and dataset state
unchanged. Linear-model F tests keep their current behavior.

## Validation contract

- Use a deterministic 40-row binary design with an intercept and two predictors;
  exercise logit and probit under model-based, independent-row robust and
  one-way cluster covariance.
- Compare one- and two-restriction Wald statistics and upper-tail probabilities
  against a separate base R reference that uses `glm`, the converged observed
  information, and explicit row/group score sandwiches.
- On the MMA fishing cohort (630 retained people, one log-relative-price slope),
  check that `test lnrelp = 0` agrees with the independently referenced slope
  Wald test for the selected covariance.
- Verify retained-fit behavior after data edits, JSON reporting, unsupported
  state errors, and singular-restriction failure behavior.

This is a bounded post-estimation method extension, not new Table 14.2 output or
a claim about discrete changes, `lincom`, binary specification tests or
few-cluster accuracy.
