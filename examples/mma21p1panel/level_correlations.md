# Year-pair correlations of panel log hours and log wages

[level_correlations.td](level_correlations.td) reproduces Cameron–Trivedi (2005)
Chapter21 §21.3.4, author [residual program](residual_inventory.md) lines105–106:
the separate observed `lnhr` and `lnwg` 10x10 year-pair Pearson correlations
across people. These are **descriptive**, not formal serial-correlation tests,
covariance corrections, causal effects or pooled/FE/RE residual correlations.
There is no fitted model or model state in this subworkflow.

## Sample and outputs

The original published MOM.dat has532 complete people x10 years1979–1988, N5320.
Inputs require3..532 complete ten-year people; unique exact integer source_row,
id and year within1..5320,1..532 and1979..1988 respectively; finite lnhr and
lnwg bounded by abs20; and positive finite cross-person variance for **both**
variables in **every** year. No missing-value deletion, weights or imputation.
All200 cells must be finite and use the same complete G people. The first export
contains all source fields in original source_row order. The second and displayed
table contain `(variable,year_a,year_b,people,correlation)`, ordered `lnhr` then
`lnwg` and ascending year pairs,100 cells per variable.

Both year variances and all200 correlations are preflighted before either export;
constant year/variable Pearson correlation is undefined and fails, not zero or
silently dropped. SQL currently exposes generic `sql failed`; the preceding scope
note states the positive-variance requirement. Use a fresh directory: exports do
not overwrite owners or source, but the two-export script is not transactional.
If the second target already exists, the first export remains; remove or rename
it before retrying. This is not a general missing-panel or unbalanced-panel API.

## Evidence

The development-only [R reference](level_correlations_reference.R) reshapes raw
data independently by sorted person and year and calls base R `cor`, without
using DuckDB's join/aggregate. On R4.6.2, the [auditor](check_level_correlations.py)
compares every one of5,320x9 raw/key cells exactly and all200 R/table/CSV cells at
absolute tolerance1e-9 (observed max3.11e-15), plus sample counts, order,
symmetry and diagonal. It checks all90 off-diagonal author-log values at the
printed four-decimal precision (absolute difference <=0.00005, max0.0000490670),
separately from the fixed tighter R comparison. Author log and original source
hashes are pinned in the auditor. These checks reproduce the printed values, not
unobserved historical internal precision.

An interleaved [dyadic16-person control](manual_gls_covariance_synthetic.py)
independently checks all160x9 source cells and200 matrix cells, and avoids the
[R decimal-parser discrepancy](manual_gls_covariance.md). Three
[actual-script tests](../../tests/mma21_level_correlations_workflow.rs) check
R-frozen control outputs, text/JSON parity, both variable labels, symmetry and
scope, nine duplicate/incomplete/missing/nonfinite/key/constant-year failure
variants, and both existing-target collisions including partial first output.
Eight forged table/note/export/raw/variable/year/people/correlation traces were
rejected by the standalone auditor. Original text and JSON exports are also
byte-identical. Publisher data/program are not redistributed and R/Python are not
production dependencies.

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
  -f "$repo/examples/mma21p1panel/level_correlations.td" > level_correlations.jsonl)
Rscript examples/mma21p1panel/level_correlations_reference.R "$raw" "$work/reference"
python3 examples/mma21p1panel/check_level_correlations.py "$work" "$work/reference" "$raw" --author-log "$log"
```

Use a separate fresh directory and `--text` for readable output. Converter/export
refuse existing targets; R requires a fresh reference directory and raw eight-
column MOM.dat, not prepared MOM.csv. Synthetic audit uses `--synthetic`, waiving
the original input hash and author-log requirement only. FD/individual correlation
blocks and the full chapter/book inventory remain open.
