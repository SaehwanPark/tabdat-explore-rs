# Figure 4.1: quantile slope bands

This native workflow implements the bounded Figure 4.1 analysis in the
Cameron–Trivedi (2005) §4.6.4 author program
[`mma04p2qreg.do`](https://cameron.econ.ucdavis.edu/mmabook/mma04p2qreg.do),
block 3. It plots the log-total-expenditure coefficient from nineteen
conditional quantiles .05 through .95, plus the same-sample OLS slope.

## Estimator and uncertainty

The ordered input has 5,999 source records. The joint finite sample retains
5,006 records and drops 993 rows with missing/nonfinite outcome or predictor;
valid zero log-medical-expenditure observations remain. Every IID row-pairs
replication uses the same 5,006 sampled row indices across all quantiles. The
original quantile slopes are plotted with `estimate ± 1.96 × SE`, where SE is
the square root of the selected diagonal in the joint bootstrap covariance and
that covariance uses B−1. The pointwise fixed-1.96 rule follows the author
program. The joint resampling preserves cross-quantile covariance in the
estimation report; the plotted pointwise intervals do not guarantee
simultaneous coverage. The OLS reference is fit to the same complete-finite
sample and is not given an interval in this figure.

The source workflow uses the author's B=200 and seed=10101. TabDat's explicit
xoshiro256++/SplitMix64 stream does not reproduce Stata 8's bootstrap draws or
printed SEs. IID row resampling is not design-based inference for this household
survey; weights, strata and PSUs are not modeled.

## Run the native figure

Download and prepare `qreg0902.asc` as described in the
[household workflow](README.md). From the directory containing `qreg.csv`, run:

```sh
/path/to/repo/target/release/tabdat-explore-rs --json -f /path/to/repo/examples/mma04p2qreg/figure_4_1.td > figure_4_1.jsonl
```

The script writes `quantile_slope_bands.svg`; existing destinations are never
replaced. `qregplot` is read-only with respect to data, labels, named reports
and retained fit. The structured result includes the nineteen slope estimates,
bootstrap SEs, pointwise endpoints, same-sample OLS reference, seed, RNG,
sample counts and SVG metadata. Its SVG exposes raw series values and path
coordinates for an independent rendering audit.

## Independent validation

The development reference uses R4.6.2, quantreg6.1 `rq(method="br")`,
Base-R `lm`, and the independent Python common-index generator already used by
the [joint-bootstrap audit](bootstrap.md). In a fresh audit directory:

```sh
python3 /path/to/repo/crates/tabdat-stats/tests/fixtures/bootstrap/draws.py --draws 10101 5006 200 draws.csv
Rscript /path/to/repo/examples/mma04p2qreg/joint_bootstrap_reference.R qreg0902.asc draws.csv reference
python3 /path/to/repo/examples/mma04p2qreg/check_figure_4_1.py native/figure_4_1.jsonl reference/figure_4_1.csv native/quantile_slope_bands.svg
```

The audit compares all nineteen R coefficient estimates, B−1 bootstrap SEs,
fixed-1.96 lower/upper endpoints and the OLS slope. Maximum absolute
difference is below 1e−8. It also checks the 5,999/5,006 sample counts, band
method and simultaneous-coverage flag, series ordering, axis bounds, all 58 raw
series vertices, and every corresponding SVG screen coordinate to 1e−8.
The companion [`check_joint_bootstrap_reference.py`](check_joint_bootstrap_reference.py)
pins all 5,999 author records and source order, then validates the exact retained
5,006-row cohort—including valid log-outcome zeros—and 25,030 prediction/design
cells. The source ASCII SHA256 is
`eb1b422fa29a096e608ff5f8912ee1b9fcecc749d227fbfd37b19d3b18db8b0b`.

In the recorded optimized local run, the figure completed in 82.50 seconds with
61,390,848 bytes maximum resident set size. This is one environment measurement,
not a runtime or platform guarantee. The output is an SVG analysis figure, not
Stata WMF/pixel parity, a survey estimate or simultaneous confidence band.
