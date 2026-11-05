# Ordinary auxiliary Hausman diagnostic

[iid_hausman.td](iid_hausman.td) implements P2-HM-IID, manual source314–316,
§21.4.3 pp717–719 in the [pinned inventory](manual_inventory.md). It loads and
preflights MOM, constructs the fixed Swamy–Arora quasi/within design, fits ordinary
auxiliary OLS, predicts, reports the within-wage contrast/interval and H=t², then
exports the ordered source/design/prediction rows. Existing primitives only;
no production Rust/API/dependency change. The robust diagnostic is unchanged.

## Sample, design and inference

The original sample is N5320/G532/T10. Supported inputs have unique exact integer
source_row/id/year in ranges1..5320/1..532/1979..1988. Joint finite lnhr/lnwg
cases define all means/components/fits; retained people must have equal T>=2,
G>3, positive within error variance and identified within/between wage designs.
No weights, extra slopes, unbalanced retained sample or time effects. Entirely
excluded people do not enter G. Balance means equal retained counts, not a common
calendar or consecutive time requirement. Keys are checked before case deletion.

Compute once e=RSS_within/(N-G-1), raw_u=RSS_between/(G-2)-e/T,
u=max(raw_u,0), theta=1-sqrt(e/(e+T*u)); finite theta must be in[0,1).
A negative raw component legitimately yields theta0. The auxiliary regression is
quasi_y ~ intercept + quasi_x + within_x, K3, with quasi variables equal to
original variables minus theta times person means. Ordinary covariance is
RSS_aux/(N-3)*(X'X)^-1, **not** the within variance component times the inverse.
The quasi_x coefficient equals the Between slope; delta (within_x coefficient)
equals within-minus-Between slope. It is **not direct FE-minus-RE**.

`lincom within_lnwg` reports delta/SE/t and Student-t95 with DF=N-3;
`test within_lnwg = 0` reports H=t² with a working F(1,N-3) tail. Estimated theta
prevents a general exact finite-sample claim. The asymptotic chi-square(1)
interpretation requires null RE efficiency/exogeneity and regularity; its tail
is not silently substituted for native F output. No general Hausman command,
robust inference, ML test or covariance subtraction is added. Predictions and
residuals belong to this auxiliary regression, not a native FE/RE model. Late
lincom/test execute after prediction/summarize to check owned covariance/DF.

## Independent evidence and failures

The [development-only R4.6.2/plm2.6.7 reference](iid_hausman_reference.R) reads raw
MOM.dat, obtains plm Swamy–Arora theta, fits independent auxiliary QR and verifies
within/Between coefficient equivalence. The [auditor](check_iid_hausman.py)
checks39 coefficients/full3x3 covariance/fit/interval/F/DF fields and all5,320x18
source/transform/prediction cells, including47,880 exact raw/key cells. Fixed
absolute tolerance1e-9, p-relative1e-8/exact underflow0; no widened tolerances.
Original H=13.69034469972815, working F(1,5317) p=0.00021773807753413903;
separate asymptotic chi-square p=0.00021555989719666303. This is native float64
method evidence, not historical Stata float32 printed parity.

Two [interleaved dyadic16-person controls](manual_gls_covariance_synthetic.py)
validate positive theta and theta0/null contrast: another78 QR fields/5,760
exported cells. Frozen references in [fixtures](../../tests/fixtures/mma21_iid_hausman/)
are consumed by [three actual-script tests](../../tests/mma21_iid_hausman_workflow.rs).
They check both CLI modes, prediction byte parity, late-owned CI/F, existing
export/source preservation and seven invalid duplicate/fractional/missing/
unbalanced/within-rank/between-rank/error-component variants. Failed preflights
do not export; the script does not provide whole-session rollback. Original
text/JSON export parity and standalone forged-trace rejection are checked too.

## Reproduce

From the repo root with a current binary (`cargo build --locked`), locally
obtained author MOM.dat, and R/plm for independent reference only:

```sh
repo=$PWD
raw=/absolute/path/to/MOM.dat
work=$(mktemp -d)
sh examples/mma21p1panel/prepare.sh "$raw" "$work/MOM.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json \
  -f "$repo/examples/mma21p1panel/iid_hausman.td" > iid_hausman.jsonl)
Rscript examples/mma21p1panel/iid_hausman_reference.R "$raw" "$work/reference"
python3 examples/mma21p1panel/check_iid_hausman.py "$work" "$work/reference" "$raw"
```

Use a separate fresh directory and `--text` for readable output. Converter/export
refuse existing targets; R requires a fresh reference directory. Pass raw eight-
column MOM.dat to R, **not prepared nine-column MOM.csv**. Synthetic audits use
`--synthetic`, waiving only the original source hash. Controls use coarse dyadic
inputs to avoid the documented [R decimal-parser discrepancy](manual_gls_covariance.md).
No publisher data/program is redistributed; Python/R are not production dependencies.
Combined theta/H reporting, fixed-ML auxiliary work, companion inventories, full
mixed comparisons and complete chapter/book coverage remain required.
