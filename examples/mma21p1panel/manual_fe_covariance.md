# Manual FE auxiliary covariance comparison

[manual_fe_covariance.td](manual_fe_covariance.td) delivers P2-FE-IID/CL/HC1,
source lines240–245/254–255 in the [source-pinned manual inventory](manual_inventory.md).
It loads/preflights MOM, shows canonical FE context, constructs person-centered
lnhr/lnwg, fits three auxiliary OLS covariances, stores/reports named comparisons,
reports slope t95 intervals and exports ordered means/deviations/predictions.
Terminal/script/CLI-v1 use existing primitives; no parser/kernel/API/dependency
change. This is not complete Table21.2, historical Stata execution or chapter/book
coverage. Canonical FE named snapshots remain unsupported.

## Sample, covariance and interpretation

Original MOM has N5320/G532/T10. All model values must be finite and abs<=20;
3..532 complete ten-year people, unique id/year and source_row, exact integer
id1..532/year1979..1988/source_row1..5320. Missing model values or incomplete
people fail, not silently change the estimation cohort. Existing full-rank/work
limits apply. No weights/time effects/extra slopes. Other original columns and
source keys survive; output is ordered by source_row. Reproduction uses the
converter's physical-input-line source_row prefix.

For `wY=y-mean_person(y)`, `wX=x-mean_person(x)`, X=[1,wX], e=wY-X*b,
B=(X'X)^-1 and person score s_g=sum_person(X_i*e_i):

| Auxiliary column | Covariance | Fit residual DF | Slope inference DF |
| --- | --- | --- | --- |
| p2_fe_iid_working | RSS/(N-2)*B |5318|5318|
| p2_fe_hc1_working | N/(N-2)*B*sum_i(score_i score_i')*B |5318|5318|
| p2_fe_person_cluster | G/(G-1)*(N-1)/(N-2)*B*sum_g(s_g s_g')*B |5318|531|

The canonical baseline instead absorbs G unit means and uses4787 classical DF.
Ordinary/row-HC1 auxiliary rows are **working comparisons**, not automatically
valid absorbed-FE covariance: demeaning induces within-person dependence and
estimates means. Person-cluster CR1 assumes independent people and adequate
cluster asymptotics; Student-t(G-1) is a conventional approximation, not a general
exact finite-sample reference. Nothing here establishes nominal coverage or the
error law from a covariance calculation. Full covariance order is `intercept`,
`within_lnwg`; the canonical baseline slope is `lnwg`.

The auxiliary intercept is mathematically zero, **not** the normalized level
intercept. Generic fit/named output can show meaningless centered-intercept t/p
artifacts (especially zero/near-zero clustered SE). Both text and structured
query notes explicitly reject their interpretation; do not extract them as
scientific inference. Only slope t/p/intervals are validated. Intercept point/SE/
covariance are checked numerically; cluster intercept SE/row and all centered
intercept points must have abs<=1e-12. No variance is clipped to pass a test, no
replicates exist and no singular full-joint test is attempted. Valid vanishing
slope scores can leave floating-point SE/t/p artifacts too; small/zero empirical
variance alone does not justify a nominal interval.

Generic fit `df_residual` remains N-2; the named snapshot's
`inference_degrees_of_freedom` and slope `lincom`'s `df_residual` carry G-1 for
clustered inference. Do not confuse fit residual count with inference DF or
claim literal Stata e(df_r) field parity. Named registry sample indices and
rowwise cluster labels are checked exactly. The last cluster slope interval runs
**after** named reports, SQL notes, prediction columns and summarize, checking
that these operations preserve the owned cluster covariance/DF. Final predictions
are auxiliary centered fitted/residual values, not raw-hour level predictions or
an automatically restored FE owner.

## Independent evidence

[Development-only R reference](manual_fe_covariance_reference.R) uses original
raw person means, full dummy QR for canonical FE, separate auxiliary QR and
explicit row/person score cross-products. R4.6.2 validates68 fit/covariance/
slope-interval/DF fields on the original sample, three full covariance matrices,
all79,800 exported cells (47,880 exact raw/key fields), all retained sample indices
and cluster labels, and copied named-report provenance. Ten forged covariance,
DF/provenance/note/prediction/source/export traces are rejected; an old CSV alone
cannot substitute for successful export. Python's
[auditor](check_manual_fe_covariance.py) verifies those outputs independently of
Rust tests and pins the original raw hash from the inventory. Source code/log
hashes and target line references are in that inventory; neither is vendored.

Native float64 slope is0.1676754886293148 (minor last-bit QR differences allowed):

| Auxiliary mode | Slope SE | Two-sided t p |
| --- | --- | --- |
| IID working |0.01790315598882865|1.0893257805446315e-20|
| Row-HC1 working |0.06009420261470756|0.005286139295002397|
| Person CR1 |0.0849705990602669|0.048975458508170686|

Fixed comparisons: abs1e-9 except p relative1e-8 (exact zero for underflow).
Centered intercept t/p are excluded by analytic degeneracy, not by widening
these tolerances. Original text/JSON prediction bytes match. The existing
[16-person heterogeneous/interleaved control generator](fe_level_bootstrap_synthetic.py)
(`heterogeneous`) supplies the unchanged input fixture used by
[three actual-script integration tests](../../tests/mma21_manual_fe_covariance_workflow.rs);
[frozen R values](../../tests/fixtures/mma21_manual_fe_covariance/reference.csv)
and [all160 prediction rows](../../tests/fixtures/mma21_manual_fe_covariance/predictions.csv)
add68 fields/2400 cells. A separate exact dyadic/orthogonal quadratic control
retains positive ordinary residual variance and vanishing person slope scores;
the workflow completes without forcing an invertible joint covariance. Duplicate/
missing/incomplete/fractional keys, unidentified within designs and export
collision/source-preservation failures are exercised. An existing export owner
is preserved; earlier commands are **not** rolled back as a whole session.

## Reproduce

From the repo root, with the author MOM.dat downloaded locally and a current
Rust binary (`cargo build --locked`):

```sh
repo=$PWD
raw=/absolute/path/to/MOM.dat
work=$(mktemp -d)
sh examples/mma21p1panel/prepare.sh "$raw" "$work/MOM.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json \
  -f "$repo/examples/mma21p1panel/manual_fe_covariance.td" > manual_fe_covariance.jsonl)
Rscript examples/mma21p1panel/manual_fe_covariance_reference.R "$raw" "$work/reference"
python3 examples/mma21p1panel/check_manual_fe_covariance.py "$work" "$work/reference" "$raw"
```

Use a separate fresh work directory and `--text` for readable reports. Neither
converter/export nor R reference replaces an existing target. Synthetic audits
require the explicit `--synthetic` label, which waives only the original-file
hash and not numerical/sample/covariance checks. Python/R are reference-only,
not production runtime dependencies. Historical float32/printed-SE parity,
individual-effect uncertainty, fixed-ML auxiliary inference/bootstrap and the
full mixed canonical/auxiliary saved table remain open.
