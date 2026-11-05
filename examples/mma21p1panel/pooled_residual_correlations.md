# Table21.3 pooled residual correlations

[pooled_residual_correlations.td](pooled_residual_correlations.td) delivers the
pooled-OLS portion of Cameron–Trivedi (2005) §21.3.4 p714/Table21.3, author
March2005 [`mma21p3panresiduals.do`](https://cameron.econ.ucdavis.edu/mmabook/mma21p3panresiduals.do)
lines75–108 and [output](https://cameron.econ.ucdavis.edu/mmabook/mma21p3panresiduals.txt)
lines321–343. The [source inventory](residual_inventory.md) distinguishes this
validated block from remaining FE/RE, first-difference, level and individual
correlations. Existing SQL/OLS/predict/export only; no production API/dependency
change. The correlations are descriptive, **not** a formal serial-correlation
test, covariance correction or causal inference.

## Sample, design and state

Original MOM.dat contains532 people x10 years1979–1988, N5320. Supported inputs
have3..532 complete ten-year people, unique exact integer source_row/id/year in
ranges1..5320/1..532/1979..1988, finite abs(lnhr/lnwg)<=20, full-rank pooled OLS
intercept+lnwg and strictly positive finite cross-person residual variance in each
year. All100 year-pair correlations must be finite/identified and use the same
complete G people. Missing/incomplete samples fail rather than being silently
changed; no weights, extra regressors or imputation. The converter prefixes
physical source lines; row export preserves source order and all nine raw/key
columns plus fitted/residual values. Matrix export and the readable/structured
SQL table use100 ordered `(year_a,year_b,people,correlation)` rows. Each cell
is the Pearson correlation of the two years' pooled residuals **across people**,
not a time-series correlation for one person. No averaging of per-person
correlations is performed.

Preflight validates year variances and all matrix cells before the first export.
The SQL boundary currently reports a generic `sql failed` on guard failure; the
preceding scope note explains the positive-variance requirement. Fix the input
or choose a different, explicitly specified analysis; a constant residual year
has undefined Pearson correlation. The retained pooled OLS model supplies late
`lincom lnwg` after SQL materialization; the matrix is not itself a model.
Exports never overwrite existing targets or source. The two-export script is not
a whole-session filesystem transaction: if the second destination exists, the
first row export has already been created and remains. Use fresh paths and remove
or rename an unwanted first export before retrying.

## Independent and failure evidence

[Base R4.6.2 reference](pooled_residual_correlations_reference.R) reads raw
MOM.dat, fits independent QR, reshapes by explicit person/year keys and calls
`cor` on the person-by-year matrix. The [auditor](check_pooled_residual_correlations.py)
checks25 OLS coefficient/full covariance/fit/late-interval fields, all5,320x11
raw/prediction cells, all100 correlation/count cells and the structured SQL table.
Raw/key values are exact; numerical abs tolerance1e-9, p-relative1e-8/exact
underflow0. The author source SHA256 is
`6c4ccfa560eb8f818412e87895e4c205f51c1e00865346f3b0c532902b7a43e8`;
the author output hash is
`925aff11176cb78bbacf71abc69c2bef4876bdcc0e5544c001c2df4f5f8f56b5`.
All45 off-diagonal publisher-log numbers agree with float64 results to their
printed four decimals (absolute difference <=0.00005; maximum 0.00004809895).
This separate printed-value comparison is not a widened R tolerance or a claim
of historical internal precision parity. Original reference regeneration and both
text/JSON exported files are byte-identical across modes.

A deterministic [interleaved16-person control](manual_gls_covariance_synthetic.py)
adds25 independent QR fields/1,760 raw-prediction cells/100 matrix cells. Its
[frozen references](../../tests/fixtures/mma21_pooled_residual_correlations/) are
consumed by [three actual-script tests](../../tests/mma21_pooled_residual_correlations_workflow.rs):
both CLI modes, late-owned inference, exact ordering, duplicate/incomplete/
missing/fractional/rank/constant-year failures, and both export-owner collision
paths including the explicit partial first export. Eleven forged fit/interval/
sample/table/note/export/raw/residual/matrix/fractional-key/count traces are
rejected by the standalone auditor. The dyadic control avoids the documented
[R decimal-parser discrepancy](manual_gls_covariance.md).

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
  -f "$repo/examples/mma21p1panel/pooled_residual_correlations.td" > pooled_residual_correlations.jsonl)
Rscript examples/mma21p1panel/pooled_residual_correlations_reference.R "$raw" "$work/reference"
python3 examples/mma21p1panel/check_pooled_residual_correlations.py "$work" "$work/reference" "$raw" --author-log "$log"
```

Use a separate fresh directory and `--text` for readable output. Converter/export
refuse existing targets; R requires a fresh reference directory and raw eight-column
MOM.dat, not prepared MOM.csv. Synthetic audit uses `--synthetic`, waiving the
original input hash and author-log requirement only. No publisher data/program is
redistributed; R/Python are not production dependencies. This is not whole Table21.3
beyond its pooled residual matrix, Table21.4, or full chapter/book coverage.
