# General Wald restrictions after Poisson and NB2

## Contract

`test <linear restrictions>` evaluates restrictions on the retained mean-model
coefficients after Poisson or NB2. It uses the covariance selected at fit time
and reports an asymptotic Wald chi-square statistic with degrees of freedom
equal to the number of restrictions. The existing `TestResult` fields remain
unchanged; `df_residual` is null and `is_chi2` is true. Current-data edits do not
refit or redefine the test.

Poisson tests use the stored coefficient covariance. NB2 tests use the mean
coefficient block from its full joint beta/log-alpha covariance, preserving the
dispersion contribution to uncertainty. `lnalpha` is not a mean-model
restriction. Model-based, independent-row robust and one-way cluster covariance
are supported. Cluster inference is asymptotic and has no few-cluster correction
or guarantee. Linear-model F tests remain unchanged; general count `lincom` and
other formal specification tests remain outside this slice.

Unknown or collinear/singular restrictions fail without changing the fitted
model or active data.

## Validation plan

- Compare one- and two-restriction tests for Poisson/NB2 under model-based,
  independent-row robust and one-way cluster covariance with independent R
  coefficient/covariance references on a deterministic count design.
- Run the same restrictions on the 20,186-row MMA §20.3 Table20.5 cohort after
  classical and robust Poisson/NB2 fits, deriving the reference from the existing
  independently validated Poisson and NB2 R outputs.
- Verify selected covariance, `df_residual: null`, retained-fit reuse, JSON and
  text output, unknown/singular failures, state preservation and existing linear
  F behavior.

This adds post-estimation inference only; it does not change count fitting,
Table20.6, NB2's separate Poisson-boundary test, panel inference, or whole-book
coverage.
