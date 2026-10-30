# Chapter 21 Figure 21.1: pooled source-data adaptation

This bounded terminal/script workflow builds the pooled scatter,
nonparametric fit and linear fit of Figure 21.1 (§21.3, p.712) in Cameron and
Trivedi (2005), using the March 2005
[author program](https://cameron.econ.ucdavis.edu/mmabook/mma21p1panfeandre.do).
It is a native source-data adaptation, not a claim of literal historical Stata
curve, WMF or pixel parity.

## Sample, fits and interpretation

Pinned `MOM.dat` SHA-256:
`2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77`.
The original has **5,320 person-years from 532 men in 1979–88**. Each scatter
point is a person-year: X is log hourly wage and Y is log annual hours. The
model is pooled `lnhr ~ 1 + lnwg`, with no year effects or other covariates.
On the original sample the slope is `0.0827435453`, intercept `7.4415164578`,
and person-cluster CR1 slope SE `0.0292711464`. The correction is
`G/(G−1)*(N−1)/(N−2)`, with t(G−1)=t(531) inference. This assumes independent
people and is not a guarantee for few clusters. Neither fit nor inference
establishes causal wage effects or a correct conditional mean.

The **LOWESS** uses explicitly specified span 0.8, nonrobust centred-rank
local-linear tricube fits and half-window 2,127. Input is sorted by person ID
and year before smoothing. Of the original 5,320 observations, 5,276 are in
223 exact-X tie groups; within a tie the rank-window fits can differ and
stable source-key order matters. The connected line represents ordered training
rows, not an out-of-sample function, uncertainty band or optimal span. R
`stats::lowess` uses different nearest-neighbour windows. Historical Stata
within-tie sort and float storage are unresolved; span 0.8 alone does not
establish identical historical tied vertices.

## Capacity and method boundary

The source sample requires conservative window work
`N*min(N,2*k+1)=5320*4255=22,636,600`. To support this illustration, the native
LOWESS limit increased from **20,000,000 to 23,000,000**; the unchanged limits
are 2–10,000 retained rows, finite span strictly between 0 and 1 and
`N*span>=0.5`. The cap is a conservative bounded work measure, not a promise
of identical elapsed time or unlimited input. Focused tests accept work
22,995,000 and reject 23,005,000, and runtime failure preserves data/model
state. The original native debug run took about **3 seconds and 92 MB peak
RSS** on the development host; other hardware, profiles and data may differ.
No smoothing algorithm, endpoint, fallback or inference convention changed.

## Run and recover

```sh
repo=/absolute/path/to/tabdat-explore-rs
work=/absolute/path/to/empty/work-directory
curl -fsSL https://cameron.econ.ucdavis.edu/mmabook/MOM.dat -o "$work/MOM.dat"
shasum -a 256 "$work/MOM.dat"  # compare with pinned hash above
(cd "$repo" && cargo build --locked --bin tabdat-explore-rs)
sh "$repo/examples/mma21p1panel/prepare.sh" "$work/MOM.dat" "$work/MOM.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma21p1panel/pooled_figure.td" > result.jsonl)
Rscript "$repo/examples/mma21p1panel/pooled_figure_reference.R" "$work/MOM.dat" "$work/r-reference"
python3 "$repo/examples/mma21p1panel/check_pooled_figure.py" "$work" "$work/r-reference" "$work/result.jsonl" "$work/MOM.dat"
```

Use `--text` for readable output. Both modes produce `pooled_figure.csv` and
`pooled_figure.svg` without opening a viewer. R/Python are development-only
audit tools, not production dependencies. The source converter rejects missing,
nonfinite and malformed tokens. For an already prepared CSV, the script drops
rows with missing/nonfinite X or Y **jointly before fitting or plotting**;
missing nonmodel fields do not exclude a row. It requires unique loaded
integer `(id,year)`, IDs 1–532 and years 1979–88, a full-rank design, at least
two represented people, adequate model rows, and the native LOWESS bounds.
Case-insensitive collisions with `figure_linear`/`figure_lowess` fail before
fitting. CSV key checks operate on loaded numeric values; the supplied ASCII
converter separately checks source key spellings before float conversion.

Exports do not overwrite existing destinations. Use a fresh directory and
check exit status: execution is **not transactional**. CSV export precedes
SVG publication, so an existing SVG can cause failure after CSV creation;
earlier in-memory columns and the model may remain. Input files are unchanged.
After success, active data contain only jointly finite source rows plus both
fit columns and labels. The retained native model is the clustered pooled OLS,
not LOWESS; smoothing, summaries, export, labels and plotting do not clear it.

## Independent evidence and limits

Development-only R 4.6.1 independently reads the original ASCII, computes
pooled OLS and the grouped-score CR1 covariance, and solves each local weighted
design with QR. A separate Python 2×2 normal-equation implementation checks
every LOWESS fit. Original evidence checks **all 5,320 exact source/model
rows**, full 2×2 OLS covariance/slope inference and all local fits, **5,320
actual SVG markers and 10,640 connected line vertices**, including labels,
rank order, padded axes and coordinates. QR score certificates are below
`1e-12`; floating comparisons use the fixed `1e-11+1e-10*abs(reference)` and
nonzero p-values relative `1e-9`. Maximum native/R LOWESS difference is
`6.05e-13`. Original text/JSON CSV and SVG exports are byte-identical.

A reproducible 24-person/96-row fixture includes tied wage values; a separate
R missing-data fixture validates 90 model rows from 23 people and preserves
missing nonmodel data. Focused tests cover reversed input, joint missingness,
invalid/duplicate keys, case-insensitive collision, singular design, input and
no-clobber invariants, retained OLS state and forged values/SVG geometry.
Historical tied-curve/float/pixel parity, other Chapter 21 figures, full
Table 21.2 and chapter completion remain required.
