# Chapter 21 Figure 21.4: first-differences source-data adaptation

This bounded terminal/script workflow builds the first-differences scatter,
nonparametric fit and linear fit of Figure 21.4 (§21.3 p.713) in Cameron and
Trivedi (2005), using the March 2005
[author program](https://cameron.econ.ucdavis.edu/mmabook/mma21p1panfeandre.do).
It is a reproducible native source-data adaptation, not literal historical
Stata curve, WMF or pixel parity. The author program plots differences but
labels its axes as log levels; this workflow **deliberately labels the axes as
changes** to match the plotted variables.

## Sample, fits and interpretation

Pinned `MOM.dat` SHA-256:
`2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77`.
The original has 5,320 1979–88 level observations for 532 men. Within each
person, construct the previous observation from raw rows sorted by year,
require the previous year to be exactly one year earlier, then retain only
pairs with finite Δlog hourly wage (X) and Δlog annual hours (Y). The original
complete balanced source yields **4,788 1980–88 annual pairs and 532 people**.
Unlike deleting missing level rows before lagging, this cannot bridge a
missing endpoint or a calendar gap. On the original complete source this is
the author-program pair sample; on irregular or missing inputs the explicit
consecutive/finite rules are a bounded adaptation, not a claim about historical
row-subtraction behavior.

The plotted linear fit is pooled OLS of Δlnhr on an intercept and Δlnwg, no
other covariates or year effects. The intercept is retained as in the author
program. Original slope `0.1089851494`, intercept `0.0008283062`, and
person-cluster CR1 slope SE `0.0837266162` use
`G/(G−1)*(N−1)/(N−2)` and t(G−1)=t(531). Inference assumes independent people,
not independent person-years, and does not guarantee few-cluster reliability.
First differencing removes additive person-constant effects, not all
endogeneity; the regression and graph do not establish causality.

LOWESS uses explicitly specified span 0.8, nonrobust centred-rank local-linear
tricube windows, half-window 1,914 and conservative work 18,333,252 on the
original data. Input is sorted by `(id,year)` before smoothing. Of 4,788 pairs,
4,675 belong to 224 exact floating-X tie groups. Equal X can have different
fits because stable source-key rank order matters. The connected line consists
of training-row fits, not an out-of-sample function, uncertainty band or
optimal-span choice. R `stats::lowess` uses different nearest-neighbour windows;
the historical Stata tie sort and generated-float precision remain unresolved.

## Run and recover

```sh
repo=/absolute/path/to/tabdat-explore-rs
work=/absolute/path/to/empty/work-directory
curl -fsSL https://cameron.econ.ucdavis.edu/mmabook/MOM.dat -o "$work/MOM.dat"
shasum -a 256 "$work/MOM.dat"  # compare with pinned hash above
(cd "$repo" && cargo build --locked --bin tabdat-explore-rs)
sh "$repo/examples/mma21p1panel/prepare.sh" "$work/MOM.dat" "$work/MOM.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma21p1panel/fd_figure.td" > result.jsonl)
Rscript "$repo/examples/mma21p1panel/fd_figure_reference.R" "$work/MOM.dat" "$work/r-reference"
python3 "$repo/examples/mma21p1panel/check_fd_figure.py" "$work" "$work/r-reference" "$work/result.jsonl" "$work/MOM.dat"
```

Use `--text` for readable output. Both modes produce `fd_figure.csv` and
`fd_figure.svg` without opening a viewer. R/Python are development-only audit
tools, not production dependencies. The supplied source converter rejects
missing, nonfinite and malformed tokens. The script also accepts an already
prepared CSV containing missing/nonfinite model levels and excludes only
invalid pairs *after* raw lag construction; missing nonmodel columns do not
exclude model pairs. The output includes current and previous source-row IDs,
current and previous years, current levels, differences and both fit columns;
it is a pair relation, not the full source dataset. Input files are unchanged.

Unique loaded integer `(id,year)` keys, IDs 1–532, years 1979–88 and unique
positive integer `source_row` ≤2^53−1 are required. A singular first-
difference design, insufficient finite pairs/people, unsupported numeric types
or native LOWESS work-limit excess fails. Case-insensitive collisions with
`previous_source_row`, `previous_year`, `dlnhr`, `dlnwg`, `figure_linear` or
`figure_lowess` fail before transformation. Checks operate on loaded numeric
values; the ASCII converter separately checks source key spellings. Exports do
not overwrite existing destinations. Use a fresh directory and check exit
status: execution is **not transactional**. CSV is written before SVG, so an
existing SVG can cause failure after CSV creation; earlier in-memory effects
may persist. After success the active data are finite annual pairs and the
retained native model is person-clustered first-difference OLS, not LOWESS.

## Independent evidence and limits

Development-only R 4.6.1 independently reads the raw ASCII, pairs current and
previous years by explicit key lookup, computes grouped-score CR1 covariance,
and solves every declared local design with weighted QR. A separate Python
2×2 normal-equation implementation independently reconstructs source pairs
and all LOWESS fits. The original audit checks **all 5,320 source rows**,
**4,788 exact pair/design/fit rows**, full 2×2 OLS covariance/slope inference,
**4,788 actual SVG markers and 9,576 connected vertices**, ordering, labels,
axis padding and coordinates. QR score certificates are below `1e-12`;
floating comparisons use fixed `1e-11+1e-10*abs(reference)` (nonzero p-values
relative `1e-9`). Native/R differences are bitwise equal for original ΔX/ΔY;
maximum LOWESS difference is `5.36e-14`. Original text/JSON CSV and SVG
exports are byte-identical.

The reproducible 27-row interleaved synthetic source has 19 consecutive
candidate pairs, 14 retained pairs and six people, including a gap, missing
level and wholly excluded person. Independent R and actual-script tests check
membership/no-bridging, reversed input, mode equality, invalid/duplicate keys,
output-name collisions, singular fit, input/no-clobber behavior, retained OLS
state and shared-wrong values/SVG geometry. Historical tied-curve/float/pixel
parity remains unresolved; the author's axis labels deliberately differ. Other
Chapter 21 figures, full Table 21.2 and whole-chapter completion remain required.
