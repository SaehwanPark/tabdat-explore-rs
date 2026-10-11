# Runtime `ttest` validation

## Status and provenance

- Implementation revision: `e846d92` on branch `feat/runtime-ttest`, based on
  Rust main commit `47bd6a8`.
- Python oracle checkout: `../tabdat-explore`, commit
  `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
  `601b236788872323af9277d2276a236154a0f129`; `uv.lock` SHA-256
  `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.
- Reference environment: Python 3.13.3, DuckDB 1.5.2, NumPy 2.4.4, SciPy
  1.17.1. Rust checks ran with rustc 1.97.1 on macOS arm64. The runtime uses the
  workspace locked `duckdb` crate 1.10505.0. Validation date: 2026-10-11.
- Behavioral sources: Python `Executor._execute_ttest`, `TtestResult` and
  `TtestGroupStats`, and `test_statistical_testing.py::test_ttests` /
  `::test_ttest_zero_variance`.
- The shared nominal fixture is the six-row SQL relation from the Python test
  helper `_write_testing_parquet`: `x1=[1,2,3,4,5,6]`,
  `x2=[10,12,15,18,20,25]`, and `grp=[group1,group1,group2,group2,group1,group2]`.
  The grouped samples after lexical ordering are `group1=[1,2,5]` and
  `group2=[3,4,6]`. Rust runtime tests use the same rows and columns.

## Numerical comparison

The Python oracle and the independent SciPy references agree on the nominal
fixture. SciPy references use `ttest_1samp`, `ttest_rel`, `ttest_ind` with
`equal_var=True/False`, and `scipy.stats.t.cdf/ppf` for directional tails and
confidence intervals. The comparison rule is exact for names, sample counts,
flags, and degrees-of-freedom metadata; expected NaNs are checked explicitly.
Finite means, SDs, SEs, intervals, statistics, df, and probabilities use
`max(1e-9, 1e-9 * abs(reference))` absolute tolerance.

| Form | Reference statistic | df | p-left | p-two | p-right | 95% tested-mean/difference CI |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| One-sample `x1 == 3` | 0.6546536707079772 | 5 | 0.7291977196034398 | 0.5416045607931202 | 0.2708022803965602 | `[1.5366856930196764, 5.463314306980323]` for the sample mean |
| Paired `x1 == x2` | -8.821448583550177 | 5 | 0.00015544040296753734 | 0.0003108808059350747 | 0.9998445595970324 | `[-17.00345082389974, -9.32988250943359]` |
| Pooled `x1, by(grp)` | -1.1180339887498947 | 4 | 0.1630821176725301 | 0.3261642353450602 | 0.8369178823274699 | `[-5.805546660679214, 2.4722133273458806]` |
| Welch `x1, by(grp) welch` | -1.1180339887498947 | 3.6697247706422007 | 0.16567600267743096 | 0.33135200535486176 | 0.834323997322569 | `[-5.956674371741123, 2.62334103840779]` |

SciPy's two-sided probabilities differ from the pinned Python outputs by at most
`2e-15` on the four nominal forms. Rust focused tests assert each result field
against the Python/SciPy values using the declared tolerance, including the
group and combined summaries. The paired and grouped difference summaries retain
the oracle's `obs` and `std_dev` conventions.

An additional extreme-tail SciPy check exercises the oracle's separate CDF
calculations: for `|t|≈1.3093e20` with df 5, CDF rounding makes the two-sided
`2 * (1 - cdf(abs(t)))` value zero. At negative t the direct lower tail remains
about `2.4664e-100` while the upper tail rounds to one. The stats contract test
preserves these separate outputs instead of deriving all three from the
two-sided probability.

## Sample, failure, and state checks

The runtime fixture for row filtering contains seven rows: three complete finite
groups and four rows with a NULL tested value, NULL paired value, NaN, infinity,
or NULL group. Tests assert four finite one-sample values, three complete finite
pairs, and exactly two groups after filtering, ordered as `A` then `B`. Separate
fixtures check zero and one groups after filtering, unknown variables, nonnumeric
tested variables, no finite observations, no complete pairs, and the pinned
diagnostic wording.

One-observation results keep the sample mean and set SD, SE, interval endpoints,
statistic, and p-values to NaN with df zero. Constant samples with at least two
observations have point intervals and NaN statistic and p-values, preserving the
appropriate df. The runtime suite checks active dataset metadata and stored
regression results remain unchanged on success and failure.

Scalar group labels are checked for booleans, floating-point values, decimals,
blobs, and nanosecond timestamps. The runtime rejects nested list/struct/map,
interval, and timezone-aware temporal group types with a typed error; timezone
group parity is incomplete. Python DuckDB 1.5.2 read the `TIME WITH TIME ZONE`
Parquet fixture as `03:04:05+00:00`, whereas the bundled Rust DuckDB read it as
`01:04:05+00:00`. The pinned Python environment also lacks `pytz`, so no direct
`TIMESTAMP WITH TIME ZONE` result is available. No inference is made for those
types.

## Executed checks

- Pinned Python oracle: `uv run --no-sync pytest -q -p no:cacheprovider tests/test_statistical_testing.py::test_ttests tests/test_statistical_testing.py::test_ttest_zero_variance` — 2 passed.
- Rust stats contract: `cargo test --locked -p tabdat-stats --test ttest_contract` — 6 passed.
- Rust runtime contract: `cargo test --locked -p tabdat-runtime --test ttest_contract` — 10 passed.
- Full workspace Rust and policy gates are recorded in the final PR evidence after completion.
