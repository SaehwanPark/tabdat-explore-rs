# Chapter 21 Figure 21.3: within-person source-data adaptation

This bounded terminal/script workflow builds the within-person scatter,
nonparametric fit and linear fit of Figure 21.3 (§21.3 p.712) in Cameron and
Trivedi (2005), using the March 2005
[author program](https://cameron.econ.ucdavis.edu/mmabook/mma21p1panfeandre.do).
It is a reproducible native source-data adaptation, not literal historical
Stata tied-curve, WMF or pixel parity.

## Sample, transformations and fits

Pinned `MOM.dat` SHA-256:
`2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77`.
The original contains **5,320 person-years from 532 men in 1979–88**. Select
jointly finite log hours and log wages, then for each variable plot
`(value−person mean)+observation-weighted overall mean`. This is the
within-plus-grand-mean transformation described by the
[current Stata xtdata manual](https://www.stata.com/manuals/xtxtdata.pdf),
not raw deviations centered at zero. Original data are complete. On modified
inputs this workflow uses only joint-finite model cases for both means; unlike
`xtdata`, it does not casewise-delete on nonmodel variables. There are no other
regressors or year effects.

The script first reports native person-clustered **within FE** on the eligible
level sample. For the original source, wage slope is `0.1676754886` with SE
`0.0849626112` and t(531) inference. The native FE unit-score correction is
`G/(G−1)*(N−1)/(N−p)` for p=1 slope. Its normalized common intercept is a
**point only**; individual/common FE-intercept uncertainty is not supplied.
The plotted straight line comes from a second, separate person-clustered OLS
on transformed observations. It has the same slope but uses the two-parameter
OLS correction `G/(G−1)*(N−1)/(N−2)`: original slope SE `0.0849705991`.
Do **not** call that auxiliary covariance the native FE covariance. Both
assume independent people and neither guarantees few-cluster reliability.
Within transformation removes additive person-constant effects, not all
endogeneity; neither regression establishes causality.

The plotted LOWESS uses span 0.8, nonrobust centred-rank local-linear tricube
windows, half-window 2,127 and conservative work 22,636,600. Rows are ordered
by `(id,year)` before smoothing. Of 5,320 original rows, 4,803 belong to 819
exact floating-X tie groups; equal X may produce distinct fits because stable
source-key rank order matters. The line consists of training-row fits, not an
out-of-sample function, uncertainty band or optimal span. R `stats::lowess`
uses different nearest-neighbour windows. Historical Stata tie order and float
storage remain unresolved; matching span does not establish curve parity.

## Source-scale precision and ordering

Original model values have two decimal places. Eligible loaded values must be
within ±1,000,000 and equal to their two-decimal rounding; values failing the
check are rejected. Person and overall sums use exact `DECIMAL(18,2)` followed
by one float64 division; transformed coordinates use the specified
`(raw−person mean)+overall mean` order. The final numeric quotient and
coordinates are float64: only **numerically equal resulting X values** tie
(including signed zeros), and
mathematically equal rational means from unequal counts need not be bitwise
equal. Loaded DOUBLE cannot detect additional lexical digits already lost at
CSV import. The source converter preserves the pinned original spellings; this
is not a general arbitrary-precision `xtdata` implementation.

## Run and recover

```sh
repo=/absolute/path/to/tabdat-explore-rs
work=/absolute/path/to/empty/work-directory
curl -fsSL https://cameron.econ.ucdavis.edu/mmabook/MOM.dat -o "$work/MOM.dat"
shasum -a 256 "$work/MOM.dat"  # compare with pinned hash above
(cd "$repo" && cargo build --locked --bin tabdat-explore-rs)
sh "$repo/examples/mma21p1panel/prepare.sh" "$work/MOM.dat" "$work/MOM.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma21p1panel/within_figure.td" > result.jsonl)
Rscript "$repo/examples/mma21p1panel/within_figure_reference.R" "$work/MOM.dat" "$work/r-reference"
python3 "$repo/examples/mma21p1panel/check_within_figure.py" "$work" "$work/r-reference" "$work/result.jsonl" "$work/MOM.dat"
```

Use `--text` for readable output. Both modes produce `within_figure.csv` and
`within_figure.svg` without opening a viewer. R/Python are development-only
audit tools, not production dependencies. The source converter rejects missing,
nonfinite or malformed tokens. For already prepared CSV, missing/nonfinite
model values are excluded jointly; nonmodel missing values do not exclude an
otherwise eligible row. Unique loaded integer `(id,year)`, IDs 1–532 and years
1979–88, identifiable FE and auxiliary OLS designs, and native LOWESS bounds
are required. Case-insensitive collisions with generated unit/overall/within
means or fit columns fail before fitting. Checks use loaded numeric values;
the ASCII converter separately checks source key spellings.

Exports do not overwrite. Use a fresh directory and check exit status: execution
is **not transactional**. The transformed relation retains the prior FE state
until auxiliary OLS succeeds; on success the retained model is auxiliary OLS,
not FE or LOWESS. Earlier state/data may remain after a later failure; CSV is
written before SVG, so an existing SVG can leave a newly created CSV. Input
files remain unchanged. The active data after success are the finite source
rows, unit/overall means, transformed coordinates and fit columns. Native
LOWESS, summaries, export, labels and plot do not clear the auxiliary OLS fit.

## Independent evidence and limits

Development-only R 4.6.1/`plm` 2.6.7 independently reads the source,
computes integer-hundredth person/overall sums, within-FE cluster covariance,
auxiliary OLS grouped-score CR1 covariance and weighted QR in each local window.
A separate Python integer-sum/2×2 normal-equation audit checks all **5,320
source rows**, exact transformed coordinates/ranks and all fits, both inference
conventions, **5,320 actual SVG markers and 10,640 connected vertices**,
ordering, labels, axis padding and coordinates. QR score certificates are below
`1e-12`; fixed tolerance is `1e-11+1e-10*abs(reference)` (nonzero p-values
relative `1e-9`). Original native/R transformed values and ranks are exactly
identical; maximum LOWESS difference is `1.83e-13`. Original text/JSON CSV and
SVG exports are byte-identical.

Two reproducible 96-source-row synthetic references check complete and missing
model/nonmodel input (96 and 90 model rows; 24 and 23 people). Actual-script
tests cover reversed ordering, missingness, invalid/duplicate keys, source-scale
precision and name collisions, singular fit, no-clobber/source invariants,
retained FE/OLS state and shared-wrong numerical/SVG geometry forgeries.
Historical tied-curve/float/pixel parity, remaining Table 21.2 outputs and the
full chapter/book target remain unresolved.
