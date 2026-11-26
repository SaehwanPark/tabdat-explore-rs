# qregplot

Installed syntax:
`qregplot y xvars, coefficient(x) quantiles(values) seed(integer) [reps(200) noconstant] saving("file.svg") noopen [title("text")]`

Fit joint conditional quantiles with common IID row-pairs bootstrap draws and
render one selected slope against quantile. The active sample is the ordered
joint complete-finite outcome/predictor sample. Predictors are numeric columns;
one listed predictor must be selected by `coefficient()`. Quantiles must contain
2–19 strictly increasing finite values from .05 through .95. The seed is an
explicit unsigned 64-bit integer; replications are 2–5000 (default 200).
The underlying fit requires N>K, N≤10,000, K≤32 and N×K²≤10,000,000;
the joint bootstrap also bounds N×K²×Q×(B+1) at 100,000,000, where K
includes the intercept, Q is the number of quantiles and B is the replication
count.

For each quantile, the point is the original sample coefficient. Its standard
error is the square root of the selected diagonal from the common-draw bootstrap
covariance, whose denominator is B−1. The displayed endpoints use the source
program's fixed `estimate ± 1.96 × SE` rule. These are pointwise 95% normal
approximation bands; they do not guarantee simultaneous coverage. The OLS
coefficient is fit on the same complete-finite sample and appears as a reference
line. It has no interval in this plot. This resampling convention assumes IID
rows and does not account for survey weights, strata, PSUs or a household
sampling design.

Examples:

```text
qregplot lnmed lntotal, coefficient(lntotal) quantiles(.05 .1 .15 .2 .25 .3 .35 .4 .45 .5 .55 .6 .65 .7 .75 .8 .85 .9 .95) seed(10101) reps(200) saving("quantile_slope_bands.svg") noopen title("Slope Estimates as Quantile Varies")
```

`saving()` must name an SVG destination, and `noopen` is required. Estimation
and rendering finish before filesystem effects. Publication writes a complete
sibling stage, then uses an atomic no-clobber hard link; existing files,
directories and symlinks are never replaced. This requires a local
hard-link-capable filesystem. Stage cleanup is best effort after filesystem or
crash failures, so publication is not a crash-durability guarantee. Parent
directories may be created after rendering. Data, labels, named reports and any
retained fit remain unchanged on success or failure. No clustered, weighted,
factor, interaction, robust or survey variant is installed.

The figure workflow and independent source audit are in
`examples/mma04p2qreg/figure_4_1.md`.
