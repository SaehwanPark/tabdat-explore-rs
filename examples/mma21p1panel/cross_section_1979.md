# Chapter21 1979 cross-sectional regression

[cross_section_1979.td](cross_section_1979.td) reproduces Cameron–Trivedi
(2005) §21.3.4 author [residual program](residual_inventory.md) line114: an
intercept OLS of 1979 log hours on 1979 log wages across people. It is a single
**cross-sectional** regression, not within-panel FE, causal identification,
cluster-robust inference or an additional residual-correlation matrix.

## Sample, inference and outputs

Original MOM.dat is532 complete people x10 years1979–1988; this bounded
composition accepts3..532 complete ten-year people, unique exact integer
source_row/id/year in1..5320/1..532/1979..1988, and finite lnhr/lnwg bounded
by abs20. No missing-value deletion, imputation, weights or irregular-panel
support. Exactly one1979 row per person is selected, preserving original source
row ordering. Thus original N532 and classical residual df530; the 1979 wage
design must be identified. Other years are validated before selection rather
than silently ignored.

The intercept and wage slope use unweighted OLS, nonrobust homoskedastic
RSS/(N−2) covariance, Student-t(N−2) coefficient inference and95% slope interval,
and a slope-zero F(1,N−2) test. The F statistic equals the squared slope t; this
working reference does not account for heteroskedasticity or establish causality.
The script outputs named sample/fit/test/lincom results and exports current source
fields plus fitted values and residuals ordered by source_row. It materializes the
prediction frame before `test` and `lincom`, verifying retained OLS state. The
export does not overwrite a source or existing target; use a fresh directory.
Generic SQL guard errors currently surface as `sql failed`.

## Independent and historical evidence

The development-only [R4.6.2 reference](cross_section_1979_reference.R) selects
1979 from raw MOM and fits independent QR OLS. The
[auditor](check_cross_section_1979.py) verifies29 coefficient/full covariance/
inference/lincom/F-test fields (abs1e-9, p-relative1e-8 with exact underflow zero)
and all532x11 source/fitted/residual cells, plus F distribution, sample,
ordering, prediction identity, scope and retained-state event order. Native slope
is0.000617293704923; test F is0.000569919511 with df1,530 and p0.980962906639.
Source and author-log hashes are pinned.

The historical printed intercept is `7.667738` (six decimals); the raw float64
R QR estimate7.667738522 crosses that rounding boundary. A **separate** independent
R path first rounds source lnhr/lnwg to float32 and gives7.667738479; all12
author coefficient/SE/t/p/CI facts match at their printed precision. This is a
plausible historical storage explanation, not proof of the original binary's
representation or internal precision. Canonical raw-double R tolerance remains
unchanged; no historical R/Stata runtime is needed in production.

Two [interleaved dyadic16-person controls](cross_section_1979_synthetic.py) add58
independent R fields and32x11 prediction cells. One has exactly orthogonal wage
and outcome by design: native floating slope4.84e-16, F3.28e-30 and p1, not an
artificially discarded zero. Four [actual-script tests](../../tests/mma21_cross_section_1979_workflow.rs)
exercise both controls in text and JSON, missing data, non-1979 nonfinite data,
invalid keys, incomplete panel, singular1979 design and export-owner preservation. Twelve
forged covariance/sample/F-statistic/F-distribution/lincom/note/export/raw/year/
fitted/residual/float32 traces were rejected by the standalone auditor. Original
text and JSON exports are byte-identical. Data/program are not redistributed;
R/Python are development-only.

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
  -f "$repo/examples/mma21p1panel/cross_section_1979.td" > cross_section_1979.jsonl)
Rscript examples/mma21p1panel/cross_section_1979_reference.R "$raw" "$work/reference"
python3 examples/mma21p1panel/check_cross_section_1979.py "$work" "$work/reference" "$raw" --author-log "$log"
```

Use a separate fresh directory and `--text` for readable output. Converter/export
refuse existing targets; R requires a fresh reference directory and raw eight-
column MOM.dat, not prepared MOM.csv. Synthetic audit uses `--synthetic`, waiving
the original input hash and author-log requirement only. The combined residual
report, full companion inventory and complete chapter/book coverage remain open.
