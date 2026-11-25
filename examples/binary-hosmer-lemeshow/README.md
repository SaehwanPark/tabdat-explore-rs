# Grouped binary goodness of fit

## Executable workflow

Run `logit.td` or `probit.td` to fit an additive binary response model and
evaluate `estat gof, group(10)`. The command reports the Hosmer–Lemeshow
quantile-group calibration test from the retained estimation sample. The
fixture has 160 source rows and one missing predictor value, leaving the same
159 complete cases for each estimator.

From this directory, run `tabdat-explore-rs --json -f logit.td` or
`tabdat-explore-rs --json -f probit.td`; use `--text` for the terminal report.

The fitted probabilities have four exact values because the design repeats
four predictor patterns. Requesting ten groups therefore yields four actual
groups. This demonstrates the defined tie behavior and the use of the actual
group count for the `G - 2` degrees of freedom.

## Statistical contract

The supported command requires an integer `group(#)` from 3 through 20 and a
retained, unweighted, model-based binary logit or probit fit. It uses the
original retained response and fitted probabilities; it does not refit or read
current dataset values. Robust and clustered fits are rejected because their
covariance choices do not justify the independent-Bernoulli chi-square
calibration. A bare `estat gof` is not installed here; its distinct
covariate-pattern Pearson test is not substituted.

Observations are ordered by fitted event probability and divided at
approximately equal-size quantiles. Rows with exactly equal probabilities
stay together, which can make groups uneven or reduce the actual group count.
At least three actual groups are required. For group `g`, let `n_g` be the row
count, `O_g` observed events, `E_g = sum(p_i)` expected events and
`E0_g = sum(1 - p_i)` expected non-events. The statistic and reference are

```text
HL = sum_g (O_g - E_g)^2 / (E_g * E0_g / n_g)
df = actual_group_count - 2
```

The p-value is the upper tail of the asymptotic chi-square distribution. The
group table reports inclusive upper probability boundaries and observed and
expected event/non-event counts. Groups with zero expected event or non-event
counts fail explicitly. Results are sensitive to the requested group count;
sparse expected cells can make the chi-square approximation unreliable.

## Independent reference

`reference.R` uses only Base R 4.6.2. It regenerates the synthetic CSV and
independently fits `glm(..., family=binomial(link="logit"))` and the probit
counterpart, then applies the stated tied-probability quantile rule. No R or
Python dependency is added to production. The committed
[`binary_gof_reference.csv`](binary_gof_reference.csv) contains every group's
boundary and observed/expected counts plus the test summary.

The Base R results for `group(10)` are:

| Fit | N | Actual groups | df | Hosmer–Lemeshow chi-square | Upper-tail p-value |
| --- | ---: | ---: | ---: | ---: | ---: |
| Logit | 159 | 4 | 2 | 0.000000281838495 | 0.999999859080762 |
| Probit | 159 | 4 | 2 | 0.00611597629901926 | 0.996946682733870 |

The runtime contract compares the statistic, p-value and probability boundaries
within `2e-10`, and expected group counts within `1e-9`. The slightly wider
expected-count tolerance accounts for different convergence paths in Rust's
scaled Newton/QR fit and Base R's `glm` optimizer; the observed counts and
boundaries match exactly up to the stated probability tolerance.

Run the reference generator and focused contracts from the repository root:

```sh
Rscript examples/binary-hosmer-lemeshow/reference.R
cargo test --locked -p tabdat-runtime --test binary_hosmer_lemeshow_contract
cargo test --locked -p tabdat-reporting --test binary_gof_contract
cargo test --locked --test cli_json_binary_gof_contract
```

Additional runtime cases cover missing fits, robust/cluster rejection,
insufficient distinct probabilities, and retained-fit stability after current
data changes and column removal. Parser cases reject unsupported options and
group counts outside 3..20.

This is a synthetic diagnostic fixture, not a replication of a Cameron and
Trivedi illustration. It adds one grouped binary calibration check and does not
complete the book's binary specification tests.

The grouping and `G - 2` reference follow the official Stata Base Reference
Manual's [`estat gof`](https://www.stata.com/manuals/restatgof.pdf) description.
