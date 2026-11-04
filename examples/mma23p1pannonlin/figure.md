# Chapter 23 Figure 23.1: pooled log-patents/R&D source-data adaptation

This terminal/script workflow recreates the **data and three visual layers**
of Figure 23.1 in Cameron and Trivedi (2005), §23.3: 1,730 raw observations,
a nonparametric local fit, and a pooled linear fit. The March 2005
[author program](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.do)
mislabels the graph "Figure 21.1" in one comment but identifies it as Chapter 23
Figure 23.1 and says its axes correct the book. As in the executed program,
horizontal X is log R&D spending (LOGR) and vertical Y is log patents (LPAT).
The SVG has accessible title/description and labelled, distinct raw/line
series; its layout and pixels are native, not the Stata WMF.

## Sample, fits and interpretation

The pinned author `patr7079.asc` SHA-256 is
`87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8`.
Source-record order determines firm IDs. Use all **346 firms × five years
1975–79**, including **337 zero-count observations** and 22 all-zero firms;
missing nonmodel ARDSSIC does not exclude a row. The author's ad hoc plotted
outcome is `ln(PAT)` for positive counts and `ln(0.5)` for zero; it is not a
logarithm of zero, and the arbitrary substitution affects the shape.

The script sorts by `(id,year)` before fitting. Native `lowess` uses explicitly
specified span **0.8**, nonrobust local-linear tricube weights and a centred
rank window, shrinking at endpoints. For N=1,730 the half-window is 691; it is
a rank fraction, not a fixed LOGR distance. The 77 original rows in 37 exact-X
tie groups retain source key order and may have different fits at the same X.
The exported points and connected curve are **ordered row fits**, not an
out-of-sample function, confidence band or selected optimal bandwidth. R's
`stats::lowess`, even without robustness/interpolation, uses different windows
and is not the oracle for this method. The author specified default Stata
LOWESS, but its historical tie ordering and input precision are not established:
no literal historical tied-curve or pixel parity is claimed.

The straight line is pooled OLS with an intercept, the same coefficients as the
[linear panel workflow](linear_panel.md): intercept `0.7954785111`, LOGR slope
`0.8340744986`. The separately reported slope SE `0.0228654911` is firm-cluster
CR1, with `G/(G−1)*(N−1)/(N−2)` correction and t(345) inference; the curve has
no SE or confidence band. Clustering assumes independent firms and does not
guarantee valid few-cluster inference. A graphical association and either
fitted curve do not establish a causal R&D effect, the conditional mean model,
or a correct count likelihood.

## Run and recover

```sh
repo=/absolute/path/to/tabdat-explore-rs
work=/absolute/path/to/empty/work-directory
curl -fsSL https://cameron.econ.ucdavis.edu/mmabook/patr7079.asc -o "$work/patr7079.asc"
shasum -a 256 "$work/patr7079.asc"  # compare with the pinned hash above
(cd "$repo" && cargo build --locked --bin tabdat-explore-rs)
sh "$repo/examples/mma23p1pannonlin/prepare.sh" "$work/patr7079.asc" "$work/patents.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma23p1pannonlin/figure.td" > result.jsonl)
Rscript "$repo/examples/mma23p1pannonlin/figure_reference.R" "$work/patr7079.asc" "$work/r-reference"
python3 "$repo/examples/mma23p1pannonlin/check_figure.py" "$work" "$work/r-reference" "$work/result.jsonl" "$work/patr7079.asc"
```

Use `--text` for readable output; both modes write `figure_data.csv` and
`figure.svg` without opening a viewer. R/Python are **development-only audit
tools**, not production dependencies. The script requires 2–346 firms, one
unique row for each 1975–79 year, finite integer IDs 1–346, finite integer PAT
0–1,000 and finite `|LOGR|≤20`. Duplicate/missing keys, missing or unsupported
model values and case-insensitive collisions with `lpat`, `figure_linear` or
`figure_lowess` are rejected before fits. Existing destinations are never
clobbered: use a new directory to retry. Execution is **not transactional**:
`figure_data.csv` is written before SVG creation, so an existing SVG causes
failure after CSV creation. Earlier in-memory columns/fit may remain after a
later error; check exit status, not just emitted reports. Input is unchanged.

On success the active data retain both fit columns and labels; the native
retained model is pooled cluster OLS. `lowess`, summaries, export, labels and
plotting do not install a smoothing estimator or clear that OLS fit. The
individual native commands have their own documented limits; this bounded
workflow is not a general graph or panel API.

## Independent evidence and limits

Development-only R 4.6.1 reconstructs the original ASCII sample and computes
per-window weighted QR (`lm.wfit`) and pooled OLS/firm score covariance. A
separate Python direct 2×2 normal-equation oracle checks every LOWESS fit,
including its rank order, span, radius and local-constant convention. The audit
checks all **1,730 exact source/design rows**, all original R/native/independent
fits, **1,730 actual SVG marker coordinates**, and **3,460 connected SVG line
vertices** including ordering, axis padding and labels. QR score certificates
are below `1e-12`; numerical comparison is fixed at
`1e-11 + 1e-10*abs(reference)` (nonzero slope p-value relative `1e-9`). Maximum
original native/R LOWESS difference is `1.51e-14`. Original JSON/text CSV and
SVG bytes match. A reproducible 40-firm, 200-row R fixture has 16 tied-X rows;
tests exercise reversed source order, input/case-insensitive name guards,
no-clobber, retained pooled state, forged shared-wrong numeric fits and SVG
path/marker geometry. The validation checks this explicitly defined native
adaptation; historical tie/precision geometry, individual FE intercept
uncertainty, other Table 23.1 outputs and the full chapter remain required.
