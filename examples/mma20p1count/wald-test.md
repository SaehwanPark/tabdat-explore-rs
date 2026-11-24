# General Wald restrictions after Poisson and NB2

## Contract

`test <linear restrictions>` evaluates affine restrictions on the retained
mean-model coefficients after Poisson or NB2. It uses the covariance selected
at fit time and reports an asymptotic Wald chi-square statistic with degrees
of freedom equal to the number of restrictions. The existing `TestResult`
fields stay unchanged: `df_residual` is null and `is_chi2` is true. Data edits
do not refit the estimate or redefine the test.

Poisson tests use the stored coefficient covariance. NB2 tests use the
mean-parameter block from the full joint beta/log-alpha covariance, so
dispersion uncertainty remains in the mean-coefficient covariance. `lnalpha`
is not a mean-model restriction. Model-based, independent-row robust and
one-way cluster covariance are supported. Cluster tests use an asymptotic
reference without few-cluster correction or guarantee. Linear-model F tests
are unchanged; other formal count specification tests remain outside this
slice. See the separate [linear-combination workflow](lincom.md).

Unknown or collinear/singular restrictions fail without changing the fitted
model or active data.

## Synthetic independent reference

The 36-row offset design in
[`poisson_offset.csv`](../poisson-offset/poisson_offset.csv) is fit by existing
base-R Poisson and MASS NB2 references. The new
[`count_wald_reference.R`](../../crates/tabdat-stats/tests/fixtures/count_wald_reference.R)
derives one-coefficient, nonzero affine-contrast and joint two-coefficient
Wald tests from those independent coefficients and covariance matrices. The
Rust test covers Poisson/NB2 with model-based, independent-row robust and
one-way clustered covariance: 18 statistics and upper-tail probabilities in
all. NB2 uses the top-left mean-coefficient block of the full joint covariance.

`cargo test --locked -p tabdat-stats --test count_wald` also checks malformed
NB2 covariance shape and coefficient order. Runtime tests cover the same
covariance choices, state-preserving failures, retained-fit reuse after data
edits and rejection of `lnalpha` as an unknown mean coefficient. The CLI test
checks JSON and terminal output and stops after a singular test failure.

## MMA §20.3 Table20.5 cohort

[`wald_test.td`](wald_test.td) runs one slope-zero and one joint slope-zero test
after model-based and robust Poisson/NB2 fits on the exact 20,186-row,
17-predictor Table20.5 sample. The independent `poisson_reference.R` and
`nb2_reference.R` outputs supply the coefficients and covariance; the
[`wald_reference.R`](wald_reference.R) script derives the restriction tests.
This replay used Base R 4.6.2 and MASS 7.3.66 with the checksum-pinned author
ASCII. It does not add a cluster-inference claim for the repeated person-year
sample.

| Fit | Restriction | Wald chi-square | Upper-tail p-value |
| --- | --- | ---: | ---: |
| Poisson, model-based | LC = 0 | 49.4232755152921 | 2.06283597329401e-12 |
| Poisson, model-based | LC = IDP = 0 | 447.628434781480 | 6.29101058202963e-98 |
| Poisson, robust | LC = 0 | 8.03959750282030 | 0.00457657047849830 |
| Poisson, robust | LC = IDP = 0 | 68.1516829038802 | 1.58872998100082e-15 |
| NB2, model-based | LC = 0 | 15.3617157654130 | 8.87687713476577e-05 |
| NB2, model-based | LC = IDP = 0 | 110.041821283600 | 1.27268849744188e-24 |
| NB2, robust | LC = 0 | 10.4228156182854 | 0.00124467987232471 |
| NB2, robust | LC = IDP = 0 | 68.6479879414970 | 1.23959216232001e-15 |

Native CLI results agree with these R calculations within 8e-12 in the
statistic and 4e-12 relative in the upper-tail probability. Both classical and
robust fits retain chi-square inference; robust covariance does not make the
count likelihood a panel model.

This is a bounded post-estimation extension. It does not reconcile the printed
Table20.6 NB2 row, add an NB2 boundary/specification reference for robust or
cluster covariance, or complete the count chapter or book.
