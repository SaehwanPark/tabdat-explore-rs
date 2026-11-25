# `xtsum`: panel descriptive statistics

## Objective and source

Provide a read-only, reusable panel summary command that can reproduce the
existing [Chapter 23 full-decade source audit](../../examples/mma23p1pannonlin/xtsum.md)
and handle ordinary unbalanced panels with variable-specific missingness. This
is a diagnostic, not an estimator, variance-component decomposition, or formal
test.

The definitions follow Stata's [`xtsum` manual](https://www.stata.com/manuals/xtxtsum.pdf):
overall moments use finite observations; between moments summarize the
unweighted means of contributing panels; within observations are
`x_it - xbar_i + xbar`, where `xbar` is the pooled overall mean. Stata's
[decomposition note](https://www.stata.com/support/faqs/statistics/decomposed-variances-in-xtsum/)
documents the sample corrections and the difference between the pooled mean
and the unweighted mean of panel means in unbalanced data.

## Bounded syntax

```text
xtsum [varlist] [, i(panelvar)]
```

The `i(panelvar)` option is required because this runtime does not yet retain a
general `xtset` declaration. It identifies a panel only; a time variable is not
required for this summary and no panel-time uniqueness or balance check is
implied. The identifier must be a nonmissing numeric or string column. Numeric
identifiers must be finite. Repeated rows for the same identifier are allowed.

An explicit `varlist` is summarized in its requested order, including repeated
names as the existing `summarize` command does. Without a `varlist`, summarize
all numeric columns in schema order, including a numeric panel identifier.
Selected variables must be numeric.

## Statistical and missingness contract

For each requested variable independently:

- Use its non-NULL finite values. Missing or nonfinite outcome values do not
  remove other panels or rows from another variable's summary.
- `overall_N` and `within_N` are the number of such rows. Report the pooled
  mean, sample standard deviation (`N - 1`), minimum and maximum.
- For each identifier with at least one finite value, compute its mean. `between_n`
  is the number of contributing identifiers. Report the sample standard
  deviation (`n - 1`), minimum and maximum of these panel means, with each panel
  weighted equally.
- Transform each finite row to `x_it - xbar_i + xbar` using the pooled mean for
  that variable. Report the transformed sample standard deviation (`N - 1`),
  minimum and maximum. `mean_T` is `N / between_n`, the average number of finite
  observations among contributing identifiers.

Return one row per selected variable. Counts are zero when no observations are
available. Undefined moments and extrema are `null`: for example, SD is `null`
for a sample with fewer than two observations, and `mean_T` is `null` when there
are no contributing panels. A time-invariant variable has zero within SD when
its sample contains at least two rows; its within extrema equal the pooled
mean. Singleton panels contribute to their variable's `between_n` and mean.

The command reads the active dataset and must not mutate data, labels, retained
model state, exports, or output files. Validate the identifier and every
requested variable before producing a result. Unknown columns, a nonnumeric ID,
a missing/nonfinite ID, or a nonnumeric requested variable fail with a
command-specific error. Arithmetic overflow also fails the whole command;
never publish a partial result. Existing state remains intact on success or
failure.

This slice does not include `if`/`in`, weights, a time key, `xtset`, grouped
prefixes, generalized panel balance checks, or inferential output. It does not
make claims about DTA import, labels from foreign formats, or literal Stata text
layout.

## Result and acceptance evidence

Emit a stable JSON `XtSumResult` with rows in requested/schema order and fields:
`variable`, `overall_N`, `overall_mean`, `overall_sd`, `overall_min`,
`overall_max`, `between_n`, `between_sd`, `between_min`, `between_max`,
`within_N`, `within_sd`, `within_min`, `within_max`, and `mean_T`. Undefined
numeric fields are JSON `null`. Text output names the overall, between, and
within rows and their corresponding counts.

Before release, compare the command on the checksum-pinned patents source with
the existing raw-token Base R reference and author log. Independently calculate
an uneven synthetic panel in Base R and check every field, including a case
where the pooled mean differs from the unweighted panel-mean average. Cover
empty, singleton and constant samples; panel- and variable-specific missingness;
numeric and string identifiers; order; unknown, wrong-type, null and nonfinite
identifiers; nonnumeric outcomes; and failure-state preservation. Exercise the
JSON and text adapters. Keep the prior source-specific checker as an external
oracle and compatibility guard while routing the patents script through the
general command.
