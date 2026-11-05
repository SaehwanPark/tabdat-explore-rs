# Random-effects idiosyncratic residual correlations

[random_effects_residual_correlations.td](random_effects_residual_correlations.td)
implements the RE residual block of Cameron–Trivedi (2005) §21.3.4 pp713–715,
author [residual program](residual_inventory.md) lines85–87 and95–109, output
lines344–362. It fits native balanced Swamy–Arora RE-GLS, composes original-sample
unit predictions and idiosyncratic residuals, then reports all100 ordered
cross-person year-pair correlations. These are descriptive, **not** a serial-
correlation test, covariance correction or causal inference. Existing native RE/
SQL/predict/export only; no production API/dependency or generic RE unit-effect
prediction is added.

## Sample, prediction and state

Original MOM.dat contains532 people x10 years1979–1988, N5320. Supported inputs
have3..532 complete ten-year people, unique exact integer source_row/id/year in
ranges1..5320/1..532/1979..1988, finite abs(lnhr/lnwg)<=20, identified within/
between wage designs, positive finite within variance and finite theta in[0,1).
Each year requires positive finite cross-person idiosyncratic-residual variance;
all100 cells must be finite/identified and use the same complete G people. Missing/
incomplete samples fail instead of being silently changed; no weights, extra
regressors or imputation. Negative raw unit variance is truncated to zero: theta0
and shrink0 are valid. Source rows preserve all nine raw/key fields and original
order; the matrix/table has100 ordered `(year_a,year_b,people,correlation)` rows.

The native population prediction xb excludes unit effects, so the native
population residual y-xb is **u+e**, not the author's `predict ..., e`.
[Stata's xtreg postestimation documentation](https://www.stata.com/manuals/xtxtregpostestimation.pdf)
distinguishes `e`, `ue` and the unit component `u`. On this balanced estimation
sample, let s=T*sigma_u²/(sigma_e²+T*sigma_u²)=1-(1-theta)². The SQL composition
computes uhat_i=s*mean(y-xb)_i, conditional_xbu=xb+uhat_i and
ehat_it=(y-xb)-uhat_i. **s is not theta**. Both components and theta are computed
once from the original source sample; this is an estimation-sample snapshot, not
an owned native RE random-effect predictor for current/new rows. Independent
`plm::ranef` and the historical output validate this formula and source target.
Native RE covariance uses quasi-RSS/(N-2) and normal inference, not FE/cluster
inference. After the matrix is materialized, the script returns to its saved source
frame and verifies native RE xb prediction still matches the original; general
native RE `lincom` is not installed and is not claimed.

Year variances and all matrix cells are checked before the first export. The SQL
boundary currently reports generic `sql failed` on guard failure; the preceding
scope note explains the positive-variance requirement. Fix the input or choose a
different specified analysis when a year has undefined Pearson correlation.
Exports never overwrite existing targets or source, but the two-export script is
not a whole-session filesystem transaction: a second-target collision leaves the
first row export. Use fresh paths and remove or rename an unwanted first export
before retrying.

## Independent and failure evidence

The [R4.6.2/plm2.6.7 reference](random_effects_residual_correlations_reference.R)
uses independent within/between moments and plm Swamy–Arora coefficients/full
covariance plus `ranef`, then reshapes person/year residuals and calls `cor`. The
[auditor](check_random_effects_residual_correlations.py) checks30 RE coefficient/
full covariance/component/normal-inference fields, all5,320x24 source/prediction
cells, person means and population/conditional/e/shrink identities, retained RE
prediction and all100 matrix/table cells. Raw/key fields are exact; numerical abs
1e-9, p-relative1e-8/exact underflow0. The pinned [author output](residual_inventory.md)
checks all45 off-diagonal historical `ure` entries at printed four decimals
(absolute difference <=0.00005; max0.0000467673). This comparison does not widen
R tolerance or claim historical internal-precision parity. Original reference
regeneration and both text/JSON exported files are byte-identical across modes.

Two [interleaved dyadic16-person controls](manual_gls_covariance_synthetic.py)
cover positive unit variance and negative raw variance truncated to theta0/shrink0;
they add60 independent fields/7,680 source-prediction cells/200 matrix cells.
Frozen [references](../../tests/fixtures/mma21_re_residual_correlations/) are
consumed by [four actual-script tests](../../tests/mma21_re_residual_correlations_workflow.rs):
both CLI modes, original RE prediction after SQL, shrink-vs-theta and conditional
identities, nine duplicate/incomplete/missing/fractional/within-between rank/
component/constant-year failure variants, and both export-owner collision paths
including partial first export. Fourteen forged fit-covariance/theta/sample/table/
retained-state/note/export/raw/unit/shrink/e/matrix/fractional year/count traces are
rejected by the standalone auditor. Dyadic controls avoid the documented
[R decimal-parser discrepancy](manual_gls_covariance.md).

## Reproduce

From repo root with a current binary (`cargo build --locked`), locally obtained
publisher MOM.dat and author output, and R/plm for independent reference only:

```sh
repo=$PWD
raw=/absolute/path/to/MOM.dat
log=/absolute/path/to/mma21p3panresiduals.txt
work=$(mktemp -d)
sh examples/mma21p1panel/prepare.sh "$raw" "$work/MOM.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json \
  -f "$repo/examples/mma21p1panel/random_effects_residual_correlations.td" > random_effects_residual_correlations.jsonl)
Rscript examples/mma21p1panel/random_effects_residual_correlations_reference.R "$raw" "$work/reference"
python3 examples/mma21p1panel/check_random_effects_residual_correlations.py "$work" "$work/reference" "$raw" --author-log "$log"
```

Use a separate fresh directory and `--text` for readable output. Converter/export
refuse existing targets; R requires a fresh reference directory and raw eight-column
MOM.dat, not prepared MOM.csv. Synthetic audit uses `--synthetic`, waiving original
input hash and author-log requirement only. No publisher data/program is
redistributed; R/Python are not production dependencies. FD/level/individual
correlation blocks and full program/chapter/book coverage remain open.
