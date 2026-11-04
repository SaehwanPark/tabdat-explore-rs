# Chapter 21 Figure 21.2: between-person source-data adaptation

This bounded terminal/script workflow builds the **between-person** scatter,
nonparametric fit and linear fit of Figure 21.2 (§21.3, p.712) in Cameron and
Trivedi (2005). It uses the March 2005
[author program](https://cameron.econ.ucdavis.edu/mmabook/mma21p1panfeandre.do)
and its `xtdata, be` plot, but it is a native source-data adaptation rather
than a claim of literal historical Stata curve or WMF/pixel parity.

## Sample, fits and interpretation

Pinned `MOM.dat` SHA-256:
`2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77`.
The original has **532 people × 10 years (1979–88)**. Each of the **532 raw
plot points** is one equally weighted person's mean log hourly wage (X) and
mean log annual hours (Y), not a person-year. There are no other regressors.
All original fields used by the author are finite. On this complete source,
the native jointly finite X/Y means coincide in sample membership with the
between conversion used by the author. For other inputs, this composition
filters finite X and Y *jointly before computing either mean*, and omits a
person with no eligible rows. It does not follow `xtdata`'s casewise deletion
across every included nonmodel field; the
[current Stata manual](https://www.stata.com/manuals/xtxtdata.pdf) documents
that distinction. The CSV contains only person ID, eligible-row count, first
and last eligible year, means and fitted values—not the original person-year
columns. The input file is unchanged.

The **between linear fit** is classical unweighted OLS of the person means,
with intercept, model-based `RSS/(N−2)` covariance and t(530) inference on the
original data. Its slope is `0.0668378461`, SE `0.0196634913`, and intercept
`7.4830213634`. Model-based uncertainty assumes independent, homoskedastic
person-mean errors; it is not person-cluster robust or a small-sample guarantee.
The plotted **LOWESS** uses explicit span 0.8, nonrobust centred-rank
local-linear tricube windows and half-window 212 on the original sample.
Persons are ordered by ID before smoothing; at identical numeric X the order is
stable by person ID. There are 179 original people in 84 exact-X tie groups;
within a tie the rank-window fits may differ. Points and connected lines are
training-row descriptive values, not an out-of-sample function, confidence
band, optimal-span selection or evidence of a causal wage effect.

The native smoother's centred-rank windows are not R `stats::lowess`'s
nearest-neighbour windows. The historical Stata tie sort, storage precision
and plotted curve remain unresolved; specifying the historical default span
does not establish historical tied-vertex parity.

## Exact source-scale means and ties

All original model values have two decimal places. The workflow restricts
**jointly finite loaded** model values to absolute value ≤1,000,000 and to
values equal to their two-decimal rounding, then sums each eligible person's
values as exact `DECIMAL(18,2)` and divides once in float64. Values failing
this source-specific bound are rejected, not silently rounded. Input CSV is
loaded as DOUBLE; extra lexical digits already lost when loading cannot be
recovered or detected by this numeric check. The pinned original source and
converter preserve its two-decimal spellings. This is not a generic arbitrary-
precision panel mean.

The final quotient and smoother use float64: **only numerically equal resulting
X values tie**, including signed zeros. Equal-length original panels with equal source-scale sums give
the same numeric mean; for unequal row counts, mathematically equal rational
means are not promised exact numeric equality. This convention is deterministic for
a given sample and avoids floating accumulation order changing original ties.

An initial ordered binary AVG and independent R mean differed by at most
`8.89e-16`, yet changed 56 rank positions among near ties and the largest
LOWESS fit difference was about `0.0017603` (using 17-digit R output). The revised exact-scale accumulation makes native/R original
means and ranks **exactly equal**, without widening the original tolerance.
This resolves native-method reproducibility, not historical Stata ordering.

## Run and recover

```sh
repo=/absolute/path/to/tabdat-explore-rs
work=/absolute/path/to/empty/work-directory
curl -fsSL https://cameron.econ.ucdavis.edu/mmabook/MOM.dat -o "$work/MOM.dat"
shasum -a 256 "$work/MOM.dat"  # compare with pinned hash above
(cd "$repo" && cargo build --locked --bin tabdat-explore-rs)
sh "$repo/examples/mma21p1panel/prepare.sh" "$work/MOM.dat" "$work/MOM.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma21p1panel/between_figure.td" > result.jsonl)
Rscript "$repo/examples/mma21p1panel/between_figure_reference.R" "$work/MOM.dat" "$work/r-reference"
python3 "$repo/examples/mma21p1panel/check_between_figure.py" "$work" "$work/r-reference" "$work/result.jsonl" "$work/MOM.dat"
```

Use `--text` for readable output. Both modes produce `between_figure.csv` and
`between_figure.svg` without opening a viewer. R and Python are development-
only audit tools, not production dependencies. The converter rejects missing,
nonfinite or malformed source tokens; the script also accepts an already
prepared CSV with missing/nonfinite model cells, subject to the joint-finite
rule above. It requires unique loaded integer `(id,year)`, IDs 1–532 and years
1979–88. Fewer than three usable people, a singular mean-X design, unsupported
numeric types or an existing derived/output column cause failure. A
case-insensitive preflight protects `model_rows`, `first_year`, `last_year`,
`mean_lnhr`, `mean_lnwg`, `figure_linear` and `figure_lowess` from collision.

Exports refuse existing destinations. Use a fresh directory and check exit
status: execution is **not transactional** and the CSV is written before the
SVG, so an existing SVG can cause failure after CSV creation; earlier session
changes may persist. After success, active data contain the person means and
fit columns, and the retained native model is the classical between OLS. The
LOWESS command, summaries, export, labels and plot do not install a smoother
model or clear the OLS fit. This script is not a general `xtdata` command.

## Independent evidence and limits

Development-only R 4.6.1 independently reads the raw ASCII, computes integer-
hundredth person sums and R `lm` model/covariance, then solves each declared
weighted local design with QR. Its CSV uses 17 significant digits to preserve
tie ordering. A separate Python 2×2 normal-equation implementation checks
each LOWESS fit. The original audit checks **all 5,320 exact source rows**, 532
means and exact X ranks, OLS/covariance/inference and all QR/normal-equation
fits, plus **532 actual SVG markers and 1,064 connected line vertices**,
including labels, order, axis padding and coordinates. QR score certificates
are below `1e-12`; floating comparisons use the predeclared
`1e-11+1e-10*abs(reference)` and nonzero p-values relative `1e-9`. Maximum
original native/R LOWESS difference is about `2.0e-14`. Original text/JSON CSV
and SVG bytes are identical.

A reproducible 24-person/96-row synthetic reference contains 12 paired mean-X
ties. A second R reference independently checks the same 96 source rows with
joint model/nonmodel missingness, one omitted person and 23 retained means.
Focused actual-script tests cover reversed input, missingness,
malformed/duplicate keys, representable excess precision,
case-insensitive name collision, singular design, no-clobber/source integrity,
retained OLS state and forged values/geometry. A deliberately reintroduced
floating-AVG implementation is rejected by the exact independent mean gate.
Historical Stata tie/float/pixel parity, the other Chapter 21 figures, full
Table 21.2 and whole-chapter completion remain required.
