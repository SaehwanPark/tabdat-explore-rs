# General Wald restrictions after logit and probit

## Contract

`test <linear restrictions>` evaluates affine restrictions on the coefficients
owned by the retained logit or probit fit. For a restriction `Rβ = r`, the statistic is
\((R\hat\beta-r)'(R\hat V R')^{-1}(R\hat\beta-r)\), using the covariance
selected when the model was fit. The reference is asymptotic chi-square with
degrees of freedom equal to the number of restrictions. CLI `TestResult` keeps
its existing fields and reports `df_residual: null` and `is_chi2: true`.

The selected covariance may be model-based, independent-row robust, or one-way
clustered. The test reuses the retained estimate and covariance after current
data edits. Unknown or collinear/singular restrictions fail without changing
the fit or active data. Existing linear-model F tests are unchanged. General
binary `lincom`, specification tests, finite-sample and few-cluster reference
corrections are not included.

## Synthetic independent reference

`crates/tabdat-stats/tests/fixtures/binary_wald_reference.R` fits a deterministic
40-row design with an intercept and two predictors in base R 4.6.2. It computes
model-based covariance from converged observed information, robust covariance
from row scores with N/(N−1) scaling, and one-way cluster covariance from
aggregated group scores with G/(G−1) scaling. The design has nine groups in the
clustered variant. One restriction and a joint two-restriction test are checked
for each combination of logit/probit and model-based/robust/cluster covariance:
12 statistic and upper-tail probability comparisons in total.

The committed inputs and expected values are
[`binary_wald_data.csv`](../../crates/tabdat-stats/tests/fixtures/binary_wald_data.csv)
and
[`binary_wald_reference.csv`](../../crates/tabdat-stats/tests/fixtures/binary_wald_reference.csv).
`cargo test --locked -p tabdat-stats --test binary_wald` compares both statistics
and probabilities at 1e-8 absolute tolerance, verifies robust/cluster joint
zero-slope tests against the existing omnibus results, and rejects malformed,
nonfinite and singular restrictions.

## MMA §14.2 fishing cohort

[`wald_test.td`](wald_test.td) prepares the original 630-person mode-2/4 cohort
and tests `lnrelp = 0` after classical and robust logit and probit. The checked
source is `Nldata.asc` (SHA-256
`1fb6f7d5abaf790d4df2c6a13a499ca42c0db4f950d941ba92565c30b4ab6c22`), converted
with the existing token-based converter. The independent
`reference_comparison.R` coefficient and covariance output, reproduced with
Base R 4.6.2, gives these tests:

| Fit | Wald chi-square(1) | Upper-tail p-value |
| --- | ---: | ---: |
| Classical logit | 158.929605201185 | 1.93874847795392e-36 |
| Robust logit | 194.280378518602 | 3.69887865747074e-44 |
| Classical probit | 192.300872674365 | 1.00026649672042e-43 |
| Robust probit | 232.020896467226 | 2.16086165111997e-52 |

The native CLI results agree within 8e-12 in the statistic and 4e-12 relative in
the upper-tail probability. The robust results reproduce the independently
validated slope omnibus Wald tests. This MMA workflow has one restriction and
does not make a fishing-cohort cluster-inference claim.

The runtime and CLI contracts additionally verify model-based, robust and
cluster covariance selection for both families; retained-fit behavior after
data edits; JSON and text fields; and state preservation on unknown or singular
restrictions. These checks establish the bounded method contract, not broader
binary-model or textbook coverage.
