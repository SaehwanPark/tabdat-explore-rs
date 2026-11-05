# Joint pooled, fixed-effects and random-effects residual summary

[residual_summary.td](residual_summary.td) implements the bounded combined
snapshot/report in Cameron–Trivedi (2005) §21.3.4 author
[residual program](residual_inventory.md) lines77–101. On the same complete
sample it fits pooled intercept OLS, within FE and balanced Swamy–Arora RE,
retains their predictions in one ordered long dataset, and reports descriptive
overall and year-specific summaries. These summaries are **not** a test of serial
correlation, a covariance correction or causal inference.

## Sample, residual definitions and state

Original MOM.dat contains532 complete people x10 years1979–1988, N5320. This
bounded workflow accepts3..532 complete people with unique exact integer
source_row/id/year in1..5320/1..532/1979..1988 and finite lnhr/lnwg bounded by
abs20; no missing deletion, weights or imputation. Within and between wage
designs must be identified and FE error variance positive. Swamy–Arora negative
raw unit variance is truncated to zero; theta0 remains valid.

The three residuals are deliberately different:

- `pooled_residual` = y minus intercept-plus-wage pooled fitted value.
- `fe_residual` = y minus native FE conditional fitted value xb+unit effect; FE
  unit effects use the estimation-observation-weighted zero-mean normalization.
- Native RE population residual = y−xb = u+e, **not** author `predict ..., e`.
  With T=10, s=T*sigma_u²/(sigma_e²+T*sigma_u²)=1−(1−theta)², the SQL snapshot
  computes uhat_i=s*mean(y−xb)_i and `re_e`=(y−xb)−uhat_i. This is an
  original-estimation-sample composition, not an owned general RE unit-effect
  predictor or a promise for new observations.

The first export has all original fields, source keys, three fitted/residual
families, FE unit effects and RE components/shrinkage in30 columns, ordered by
source_row. Unique `(id,year)` and source keys provide an explicit long-form
representation of the same sample used by the author's saved `mom3` and reshape;
this does not create a generic DTA save/reshape or claim identical binary files.
The second export and displayed table have55 rows: five variables (`lnhr`,
`lnwg`, pooled/FE/RE residuals), each overall and for each year, with count,
mean, **sample** SD (N−1), minimum and maximum. Overall year is null; rows are
ordered by scope, year and variable. A constant year is a valid summary with SD0,
not an undefined-correlation failure.

Each fit replaces the previous active model, as normal; earlier model states are
not claimed retained. After materializing the summary, the script returns to the
source snapshot and verifies the final RE model's native population xb prediction
still matches the original. SQL guards currently surface generic `sql failed`;
invalid keys, samples or model components fail before export. Use a fresh
directory: neither export overwrites existing owners/source, but the two-export
script is not transactional. A collision on the second target leaves the first
export; remove or rename it before retrying.

## Independent and historical evidence

The development-only [R4.6.2/plm2.6.7 reference](residual_summary_reference.R)
uses pooled QR, full individual-dummy QR with zero-mean FE normalization, and
independent Swamy–Arora moments plus `plm::ranef`. The
[auditor](check_residual_summary.py) checks64 fit/coefficient/full-covariance/
component/inference fields across the three models (abs1e-9, p-relative1e-8,
exact underflow0), all5,320x30 source/prediction cells (max absolute difference
6.19e-13), residual/shrink identities and all55 R/table/export summaries (max
2.93e-13). Original source and author-log hashes are pinned.

The historical author log prints50 year-specific summaries and three overall
residual summaries. A **separate** plausible float32 source-and-prediction-storage
R path matches all53 counts and **211 of212** numeric facts at their individual
printed rounding precision. The remaining overall RE residual mean is
−9.0083764e−11 in that R reconstruction versus printed −9.00e−11: a difference
of8.38e−14, both effectively zero, but **not exact printed-digit parity**.
Sequential/grouped calculations and several float32 intermediate alternatives did
not recover that final historical digit; its precise cause is unknown. The auditor
reports this exception separately with an absolute1e-12 near-zero check, not as a
printed-precision match. It does not widen the canonical float64 R tolerances or
claim proof of the historical implementation/storage.

Interior and negative-raw/theta0 [interleaved dyadic controls](manual_gls_covariance_synthetic.py)
add128 R fields,320x30 cells and110 summary rows; R emitted an “essentially
perfect fit” warning for the boundary control. A separate constant-year control
also matches64 R fields,160x30 cells and55 summaries. Five
[actual-script tests](../../tests/mma21_residual_summary_workflow.rs) cover both
CLI modes, component and person identities, retained RE state, zero sample SD,
eleven invalid/sample/rank/component failures and both collisions including the
partial first export. Seventeen forged model-covariance/sample/table/retained
state/note/export/raw/residual/component/year/count/SD/float32 traces were
rejected by the standalone auditor. Original text/JSON exports are byte-identical.
Publisher data/program are not redistributed; R/Python are development-only.

## Reproduce

From repo root with a current binary (`cargo build --locked`), locally obtained
publisher MOM.dat and author output, and R with plm for independent reference:

```sh
repo=$PWD
raw=/absolute/path/to/MOM.dat
log=/absolute/path/to/mma21p3panresiduals.txt
work=$(mktemp -d)
sh examples/mma21p1panel/prepare.sh "$raw" "$work/MOM.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json \
  -f "$repo/examples/mma21p1panel/residual_summary.td" > residual_summary.jsonl)
Rscript examples/mma21p1panel/residual_summary_reference.R "$raw" "$work/reference"
python3 examples/mma21p1panel/check_residual_summary.py "$work" "$work/reference" "$raw" --author-log "$log"
```

Use a separate fresh directory and `--text` for readable output. Converter/export
refuse existing targets; R requires a fresh reference directory and raw eight-
column MOM.dat, not prepared MOM.csv. Synthetic audit uses `--synthetic`, waiving
original input hash and author-log requirement only. The external FD `mom` binary
identity, GLS companion inventory and full textbook coverage remain open.
