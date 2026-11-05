# Annual first-difference series and residual correlations

[fd_residual_correlations.td](fd_residual_correlations.td) reproduces the bounded
first-difference block of Cameron–Trivedi (2005) §21.3.4, author
[residual program](residual_inventory.md) lines119–134. It explicitly builds
annual pairs from published MOM.dat, fits classical intercept OLS of dlnhr on
dlnwg, and reports three9x9 matrices of across-person correlations for dlnhr,
dlnwg and OLS residuals. These are **descriptive**, not a formal test of serial
correlation, a covariance correction or causal identification.

## Sample, design and outputs

Original MOM.dat contains532 complete people x10 years1979–1988. This bounded
workflow requires3..532 complete ten-year people; unique exact integer source_row,
id and year in1..5320,1..532 and1979..1988 respectively; and finite lnhr/lnwg
with absolute values <=20. No missing-value deletion, weights, imputation or
irregular annual gaps. The SQL `LAG` partitions by id, orders by year and keeps
only explicitly consecutive years; no prior value can leak across people. Each
person contributes nine pairs (original N4788), with an estimated intercept and
slope, nonrobust RSS/(N−2) covariance, Student-t(N−2) inference and 95% slope
interval. This is the author block's intercept model, not a no-intercept FD fit
or clustered/bootstrapped inference. Ordinary `lincom dlnwg` after matrix
materialization demonstrates that the fitted OLS state is retained.

The first export contains annual pairs in `(id,year)` order, current source fields,
previous source row/year/log values, differences, fitted values and residuals.
The source_row keys permit reconstruction of original row order, including when
people are interleaved. The second export and displayed table contain243 rows
`(variable,year_a,year_b,people,correlation)`, ordered dlnhr, dlnwg, fd_residual
then ascending year pairs; every cell uses the same complete G people.

Pearson correlations require resolved variation. For **each series and each
1980–1988 year**, its cross-person population standard deviation must exceed
1e-12; all243 cells must also be finite and identified. This explicit absolute
resolution limit is conservative, not a statistical significance threshold. In
a dyadic constructed case with mathematically constant1980 OLS residuals,
floating fit residuals ranged0..1.11e-16 (population SD4.60e-17): merely testing
variance>0 produces meaningless finite correlations. The guard rejects this case
and other series at or below the cutoff; above/below controls are tested. It does
not promise robust correlation for every ill-conditioned dataset. Fix the data or
choose another specified analysis rather than interpreting undefined correlations.

Both annual variances and all matrix cells are preflighted before either export.
SQL guard failure currently surfaces generic `sql failed`; the preceding scope
note provides the finite-resolution requirement. Use a fresh directory: exports
do not overwrite owners/source, but the two-export script is not a transaction.
A second-target collision leaves the first export; remove or rename it before
retrying. This does not add a general FD panel API or accommodate unbalanced data.

## Independent and historical evidence

The development-only [R4.6.2 reference](fd_residual_correlations_reference.R)
constructs sorted annual pairs directly from raw source, fits independent QR OLS,
reshapes each series by person and calls `cor`. The
[auditor](check_fd_residual_correlations.py) checks25 coefficient/full covariance/
inference and retained-lincom fields (abs1e-9, p-relative1e-8/exact underflow0),
all4,788x17 annual-source/prediction cells, pair adjacency and identities, and
all243 R/table/matrix cells (abs1e-9; max2.23e-15). It compares all108
off-diagonal author-log correlations at printed four-decimal precision, separately
from the tighter R tolerance (max absolute difference0.0000493213). Source and
historical log hashes are pinned; output precision is not a claim about unseen
historical internals.

The author program reloads external `mom`; its binary identity and physical order
are **not established**. Published MOM annual pairs reproduce the logged sample
size, fit and all108 printed correlations, providing a justified equivalent for
those **observed outputs only**, not proof of the missing binary identity/order.
The author lower slope interval is `.0671584`: raw float64 gives `.0671584617`,
which crosses that seven-decimal rounding boundary. An independent R path rounding
source values and their differences to float32 before QR gives `.0671584328`, and
all12 printed coefficient/SE/t/p/CI facts agree at their respective precision.
This is a plausible storage-precision explanation, not proof of the original
file's representation; canonical float64 comparison remains at its fixed R
tolerance. No Stata installation, external binary or historical data is bundled.

The reproducible [interleaved dyadic16-person control](fd_residual_correlations_synthetic.py)
adds25 R fields,144x17 cells and243 matrix cells. Four
[actual-script tests](../../tests/mma21_fd_residual_correlations_workflow.rs)
cover text/JSON parity, pair identities, retained lincom, missing/NaN/key/rank
failures, zero or unresolved variation in all three series, below/above the stated
resolution bound and both collisions/partial output. The above-threshold control
was also independently R-audited (max matrix difference4.45e-16). Fifteen forged
fit-covariance/sample/table/lincom/note/export/raw/previous/difference/residual/
variable/year/count/correlation/float32-fact traces were rejected. Original
text/JSON exports are byte-identical. R/Python are not production dependencies.

## Reproduce

From repo root with a current binary (`cargo build --locked`), locally obtained
publisher MOM.dat and author output, and R for independent reference only:

```sh
repo=$PWD
raw=/absolute/path/to/MOM.dat
log=/absolute/path/to/mma21p3panresiduals.txt
work=$(mktemp -d)
sh examples/mma21p1panel/prepare.sh "$raw" "$work/MOM.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json \
  -f "$repo/examples/mma21p1panel/fd_residual_correlations.td" > fd_residual_correlations.jsonl)
Rscript examples/mma21p1panel/fd_residual_correlations_reference.R "$raw" "$work/reference"
python3 examples/mma21p1panel/check_fd_residual_correlations.py "$work" "$work/reference" "$raw" --author-log "$log"
```

Use a separate fresh directory and `--text` for readable output. Converter/export
refuse existing targets; R requires a fresh reference directory and raw eight-
column MOM.dat, not prepared MOM.csv. Synthetic audit uses `--synthetic`, waiving
original input hash and author-log requirement only. Individual correlations,
the full companion inventory and complete chapter/book coverage remain open.
