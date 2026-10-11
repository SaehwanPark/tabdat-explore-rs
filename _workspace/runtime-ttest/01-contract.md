# Runtime `ttest` contract

## Slice and provenance

This slice executes all `ttest` forms already accepted by the Rust parser:

| Form | Contract |
| --- | --- |
| `ttest x == value` | One-sample, two-sided Student test of the mean against `value`. |
| `ttest x == y` | Paired two-sided Student test of the mean of `x - y` against zero. |
| `ttest x, by(group)` | Independent pooled equal-variance two-sided test. |
| `ttest x, by(group) welch` / `unequal` | Independent Welch two-sided test. |

The behavioral oracle is `../tabdat-explore` at commit
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`), with `uv.lock` SHA-256
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`. The contract
comes from `src/tabdat/executor.py::_execute_ttest`, result models in
`src/tabdat/models.py`, and `tests/test_statistical_testing.py::test_ttests` and
`::test_ttest_zero_variance`. Pinned oracle tests passed on 2026-10-11 with SciPy
1.17.1, DuckDB 1.5.2, and NumPy 2.4.4.

## Data, result, and error behavior

- Each command requires the active dataset. Tested columns must exist and use the
  oracle's numeric types (integer, unsigned integer, float, or decimal). A `by()`
  column may have any type but must exist.
- SQL NULL and non-finite tested values are excluded. Paired comparisons use only
  complete finite pairs. Group comparisons exclude rows with NULL group values and
  non-finite tested values; group labels are stringified and sorted lexically. The
  exactly-two-groups rule is applied after those exclusions.
- No valid one-sample observation, no complete pair, and a group count other than
  two produce the oracle's command-specific diagnostics. One observation is valid:
  its mean is returned, while sample SD, SE, confidence interval, statistic, and
  p-values are NaN; test df is zero. A constant sample with at least two
  observations has a point confidence interval and NaN statistic and p-values.
- Group and difference summaries preserve the Python result fields. Paired
  differences are `x - y`. Independent differences are group 1 minus group 2;
  pooled df is `n1 + n2 - 2`, Welch df uses Satterthwaite's formula. Independent
  difference-summary `obs` is the pooled residual df (and zero for Welch), and its
  `std_dev` is NaN, matching the oracle model.
- Sample summaries use unbiased sample SD, SE `SD / sqrt(n)`, and two-sided 95%
  Student-t confidence intervals. The test result returns left-, two-, and
  right-tail probabilities. The command is read-only: it leaves the active
  relation, dataset metadata, labels, named tables, and stored regression state
  unchanged on success and failure.

## Independent reference and comparison plan

Use SciPy 1.17.1 `scipy.stats.ttest_1samp`, `ttest_rel`, and `ttest_ind` with
`equal_var=True/False`, plus `scipy.stats.t.ppf/cdf` for the reported 95% intervals
and directional tails. The shared well-conditioned fixture is the six-row dataset
from the pinned oracle helper: `x1 = [1,2,3,4,5,6]`, `x2 = [10,12,15,18,20,25]`,
and groups `group1, group1, group2, group2, group1, group2`.

The pinned oracle outputs for the four test statistics and p-values are:

| Form | t | df | p-left | p-two | p-right |
| --- | ---: | ---: | ---: | ---: | ---: |
| One-sample (`x1 == 3`) | 0.6546536707079771 | 5 | 0.7291977196034398 | 0.5416045607931204 | 0.2708022803965602 |
| Paired (`x1 == x2`) | -8.821448583550177 | 5 | 0.00015544040296753734 | 0.0003108808059351098 | 0.9998445595970324 |
| Pooled by `grp` | -1.1180339887498947 | 4 | 0.1630821176725301 | 0.3261642353450602 | 0.8369178823274699 |
| Welch by `grp` | -1.1180339887498947 | 3.6697247706422007 | 0.16567600267743096 | 0.331352005354862 | 0.834323997322569 |

SciPy returned matching two-tailed test statistics and p-values (within
`2e-15` absolute difference on this fixture). The implementation comparison will
use exact checks for sample sizes, names, flags, df metadata, and NaN locations;
finite means, SDs, SEs, interval endpoints, statistics, df, and p-values use
`abs_tol=1e-9, rel_tol=1e-9`, suitable for the well-conditioned fixture and the
independent double-precision references. Both zero-variance and one-observation
cases are checked by explicit NaN/point-interval expectations, not by a numeric
comparator.

## Session transition and implementation boundary

`ttest` reads the active DuckDB relation into owned numeric samples, computes
statistics through `tabdat-stats`, and returns an owned runtime result. It does not
publish a new relation or modify session fields. Identifier quoting follows the
existing runtime path; `by()` group labels are fetched as DuckDB VARCHAR values and
sorted in Rust to preserve the oracle's lexical group order. Data retrieval,
validation, or statistical errors return before any session mutation.

| State | Success | Failure |
| --- | --- | --- |
| Active relation and metadata | Preserved | Preserved |
| Labels and named tables | Preserved | Preserved |
| Stored regression state | Preserved | Preserved |
| Returned result | Owned `Ttest` result | Typed runtime error |

