# Binary grouped goodness of fit

## Planned bounded workflow

Implement `estat gof, group(10)` after a retained, unweighted, model-based
binary logit or probit fit. The command will use the fit's retained estimation
sample, response and fitted probabilities. It will not refit or read current
dataset values.

The requested number of groups will be an integer from 3 through 20. Sort
observations by fitted event probability and form approximately equal-sized
quantile groups. Observations with the same fitted probability stay together,
so ties can reduce the actual group count. At least three nonempty groups are
required.

For each group, let `n_g` be its row count, `O_g` its observed event count,
`E_g = sum(p_i)` its expected event count, and `E0_g = sum(1-p_i)` its expected
non-event count. Report

```text
HL = sum_g (O_g - E_g)^2 / (E_g * E0_g / n_g)
df = actual_group_count - 2
```

The p-value uses the upper tail of the asymptotic chi-square distribution. The
output will retain per-group observed and expected event/non-event counts and
probability upper boundaries for inspection. Robust or clustered fits will be
rejected because this calibration does not account for dependence or a robust
covariance. Bare `estat gof` remains outside this slice; its covariate-pattern
Pearson definition is not substituted.

## Acceptance evidence

- Parser, CLI JSON admission, retained-state execution and terminal output
  support the explicit grouped command end to end.
- Base R independently checks logit/probit group assignments, observed and
  expected counts, statistic, degrees of freedom and upper-tail p-value on a
  deterministic fixture.
- Runtime contracts cover tied fitted probabilities, fewer actual than
  requested groups, minimum groups, invalid group counts, robust/clustered
  rejection, missing fits and retained-result stability after current-data
  edits.
- `SPEC.md`, the Chapter 14 coverage ledger and `help estat` distinguish this
  grouped calibration check from other binary specification tests and preserve
  the sensitivity to group count and sparse expected cells.

The official Stata `estat gof` reference describes `group(#)` as quantile
grouping, gives the estimation-sample reference degrees of freedom `G-2`, and
warns that ties can create uneven group sizes and unreliable chi-square
calibration when groups are small:
<https://www.stata.com/manuals/restatgof.pdf>.

This is a selected diagnostic slice, not completion of the textbook's binary
specification-test coverage.
