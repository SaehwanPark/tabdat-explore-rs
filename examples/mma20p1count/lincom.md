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

## Synthetic independent reference

The Base R fixture derives one-slope, nonzero affine-contrast and two-slope
sum combinations from independent Poisson/NB2 fits on the 36-row offset
design. Rust compares estimate, standard error, z, two-sided normal tail and
95% interval across model-based, independent-row robust and one-way cluster
covariance: 18 combinations. The NB2 result uses the mean block of its full
joint beta/lnalpha covariance. The fixture also covers malformed joint
covariance and zero-SE output.

The runtime and CLI contracts cover retained-fit reuse after data edits,
affine constants, null residual degrees of freedom, JSON/text output, unknown
coefficients, non-finite inference and failure atomicity. Existing linear
Student-t behavior remains covered by its linear-model contracts.

## MMA §20.3 Table20.5 cohort

[`lincom.td`](lincom.td) runs one-slope, nonzero affine-contrast and two-slope
sum combinations after model-based and independent-row robust Poisson/NB2 fits
on the exact 20,186-row, 17-predictor source design. The R comparison derives
the same combinations from the independently checked coefficient and
covariance references in `R_poisson_reference.csv` and `R_nb2_reference.csv`:

```sh
cd /path/to/audited-source-data
repo=/path/to/tabdat-explore-rs
Rscript "$repo/crates/tabdat-stats/tests/fixtures/poisson_reference.R" \
  mma20p1count.asc R_poisson_predictions.csv > R_poisson_reference.csv
Rscript "$repo/crates/tabdat-stats/tests/fixtures/nb2_reference.R" \
  mma20p1count.asc R_nb2_predictions.csv > R_nb2_reference.csv
Rscript "$repo/examples/mma20p1count/lincom_reference.R" \
  R_poisson_reference.csv R_nb2_reference.csv > R_lincom.csv
"$repo/target/debug/tabdat-explore-rs" --json \
  -f "$repo/examples/mma20p1count/lincom.td" > native.jsonl
```

The audited directory must contain the checksum-verified `mma20p1count.asc`
and the prepared `mma20p1count.csv`; follow the data preparation steps in the
[count workflow README](README.md). This replay used Base R 4.6.2 and MASS 7.3.66
with the checksum-pinned author ASCII. It does not add a cluster-inference
claim for this repeated person-year sample.

| Fit | Combination | Estimate | Std. error | z | Two-sided p | 95% confidence interval |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| Poisson, model-based | LC | -0.0427331999490561 | 0.00607854522863908 | -7.03016895353818 | 2.06283597329401e-12 | [-0.0546469296755865, -0.0308194702225257] |
| Poisson, model-based | 2 + LC - IDP | 2.11858370071415 | 0.0155494815465537 | 136.247867452771 | 0 | [2.08810727690463, 2.14906012452367] |
| Poisson, model-based | LC + IDP - 0.25 | -0.454050100612262 | 0.0101115922755821 | -44.9039170328024 | 0 | [-0.473868457298756, -0.434231743925768] |
| Poisson, robust | LC | -0.0427331999490561 | 0.01507121486892 | -2.83541839995798 | 0.00457657047849829 | [-0.0722722382954038, -0.0131941616027084] |
| Poisson, robust | 2 + LC - IDP | 2.11858370071415 | 0.0370297530941165 | 57.2130117997131 | 0 | [2.04600671829327, 2.19116068313503] |
| Poisson, robust | LC + IDP - 0.25 | -0.454050100612262 | 0.0253934336544955 | -17.8806106645558 | 1.66997853528986e-71 | [-0.503820316018881, -0.404279885205644] |
| NB2, model-based | LC | -0.0504404354312974 | 0.0128694197000301 | -3.91940247555836 | 8.87687713476575e-05 | [-0.0756640345452866, -0.0252168363173082] |
| NB2, model-based | 2 + LC - IDP | 2.09715734175045 | 0.0343676574389667 | 61.0212478250745 | 0 | [2.02979797093707, 2.16451671256384] |
| NB2, model-based | LC + IDP - 0.25 | -0.448038212613048 | 0.0210103658284826 | -21.3246269137145 | 6.70864395598882e-101 | [-0.489217772938885, -0.406858652287211] |
| NB2, robust | LC | -0.0504404354312974 | 0.0156237866367982 | -3.22843857279112 | 0.00124467987232472 | [-0.0810624945415601, -0.0198183763210347] |
| NB2, robust | 2 + LC - IDP | 2.09715734175045 | 0.0407199109617316 | 51.502011969558 | 0 | [2.01734778281178, 2.17696690068913] |
| NB2, robust | LC + IDP - 0.25 | -0.448038212613048 | 0.0259941788068681 | -17.2360979718532 | 1.42325149336678e-66 | [-0.498985866882204, -0.397090558343893] |

The native CLI matches these R calculations within `9.4e-13` for z statistics,
`7.7e-16` for two-sided p-values and `6.3e-15` for all other numeric fields.
The displayed zero p-values are floating-point underflow in both references,
not exact zero probabilities.

This is count post-estimation support only. It does not add alpha combinations,
few-cluster corrections, formal NB specification tests or whole-book coverage.
