# Table21.4 within-FE residual correlations

[within_residual_correlations.td](within_residual_correlations.td) implements
Cameron–Trivedi (2005) §21.3.4 p715/Table21.4, author March2005
[residual program](residual_inventory.md) lines81–83 and95–111, and output
lines365–388. It fits native classical FE, predicts conditional fitted levels
and idiosyncratic residuals, then reports all100 ordered year-pair residual
correlations across people. These are descriptive, **not** a serial-correlation
test, covariance correction or causal inference. Existing FE/SQL/predict/export
primitives only; no production API/dependency change.

## Sample, design and state

Original MOM.dat contains532 people x10 years1979–1988, N5320. Supported inputs
have3..532 complete ten-year people, unique exact integer source_row/id/year in
ranges1..5320/1..532/1979..1988, finite abs(lnhr/lnwg)<=20, identified native
FE slope and positive finite cross-person residual variance each year. All100
year-pair cells must be finite/identified and use the same complete G people.
Missing/incomplete samples fail rather than being silently changed; no weights,
extra regressors or imputation. The converter prefixes physical source lines;
the row export preserves source order and all nine raw/key columns plus four FE
predictions. The matrix export and readable/structured SQL table use ordered
`(year_a,year_b,people,correlation)` rows. Each cell is the Pearson correlation
of the two years' conditional FE residuals **across people**, not a time-series
correlation for one person.

The native normalized common component `fe_xb` is not the individual fit;
`fe_xbu=fe_xb+fe_u` includes each retained unit deviation, and
`fe_residual=lnhr-fe_xbu` is the relevant author `predict ..., e` counterpart.
The auditor checks both identities and residual sums per person, as well as an
independent full-dummy QR. Classical FE covariance/Student-t uses N-G-1 DF;
this is not person-clustered inference or the auxiliary OLS N-2 DF. The retained
FE model supplies late `lincom lnwg` after SQL materialization; the resulting
correlation matrix is not itself a model.

Year variances and all matrix cells are checked before the first export. The SQL
boundary currently reports a generic `sql failed` on guard failure; the preceding
scope note explains the positive-variance requirement. Fix the input or choose a
different specified analysis: a constant residual year has undefined Pearson
correlation. Exports never overwrite existing targets or source, but the two-export
script is not a whole-session filesystem transaction: a second-target collision
leaves the first row export. Use fresh paths and remove or rename an unwanted
first export before retrying.

## Independent and failure evidence

The [base R4.6.2 reference](within_residual_correlations_reference.R) reads raw
MOM.dat and fits a full person-dummy weighted-zero-sum QR (not native within
moments), then reshapes by explicit person/year keys and calls `cor`. The
[auditor](check_within_residual_correlations.py) checks25 FE slope/full normalized
intercept-slope covariance/fit/late-interval fields, all5,320x13 raw/prediction
cells, both prediction identities, per-person residual sums and all100 ordered
matrix/table cells. Raw/key fields are exact; numerical abs tolerance1e-9,
p-relative1e-8/exact underflow0. Independent author-log comparison checks all45
off-diagonal values to their printed four decimals (absolute difference
<=0.00005; max0.00004830976); this is not a widened R tolerance or historical
internal precision parity. Source/output hashes are pinned in the
[inventory](residual_inventory.md). Original reference regeneration and both
text/JSON export files are byte-identical across modes.

An [interleaved dyadic16-person control](manual_gls_covariance_synthetic.py)
adds25 independent QR fields/2,080 raw-prediction cells/100 matrix cells. Frozen
[references](../../tests/fixtures/mma21_within_residual_correlations/) are consumed
by [three actual-script tests](../../tests/mma21_within_residual_correlations_workflow.rs):
both CLI modes, owned FE inference, exact ordering and residual identities,
duplicate/incomplete/missing/fractional/rank/constant-year failures and both
export-owner collision paths including partial first export. Twelve forged fit/
CI/sample/table/note/export/raw/residual/unit-effect/matrix/fractional year/count
traces are rejected by the standalone auditor. Dyadic controls avoid the documented
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
  -f "$repo/examples/mma21p1panel/within_residual_correlations.td" > within_residual_correlations.jsonl)
Rscript examples/mma21p1panel/within_residual_correlations_reference.R "$raw" "$work/reference"
python3 examples/mma21p1panel/check_within_residual_correlations.py "$work" "$work/reference" "$raw" --author-log "$log"
```

Use a separate fresh directory and `--text` for readable output. Converter/export
refuse existing targets; R requires a fresh reference directory and raw eight-column
MOM.dat, not prepared MOM.csv. Synthetic audit uses `--synthetic`, waiving original
input hash and author-log requirement only. No publisher data/program is
redistributed; R/Python are not production dependencies. RE/FD/level/individual
correlation blocks and full program/chapter/book coverage remain open.
