# Household mixed-fit named comparison

**Implemented and independently validated bounded native workflow.** This covers
H-REPORT-MIXED's unified native reporting path, not literal historical Table24.4
parity. The printed ML SE discrepancies remain required and unresolved.

## Illustration, data and sample

Cameron and Trivedi (2005), Table24.4 p.851, May2005 companion program
lines139–140 compares `pa fe refgls remle`; see the [source inventory](inventory.md).
The [script](household_mixed_report.td) loads/audits/prepares, fits/stores all four
models, compares inference, predicts from retained ML and exports results.

Use the [household README](README.md)'s pinned ASCII or precise numeric-code DTA
preparation. Outcome `lnexp12m`, intercept and numeric slopes in order
`lnhhexp age sex hhsize farm educ`, grouped by `commune`. Source-row keys order
households; they are not calendar time. FARM/EDUC/SEX are not factor-expanded.
Original5999 rows,993 NULL outcomes,5006 selected cases,194 communes, sizes1..39
and one singleton. Zero outcomes remain. There are no weights/offsets or survey
semantics. Optional development R/Python are not production dependencies.

The source guard requires positive BIGINT source/commune keys, unique raw source
keys, finite predictors even on excluded-outcome rows, and finite nonmissing
outcomes. Raw input is capped at40000 rows; at least8 complete-case communes are
required for the six-slope between regression. Native rank/variance/convergence
bounds still apply. Selection and source-row sorting are explicit **before all
four fits**: each fit/snapshot therefore has candidate N5006/dropped0, not5999/993.
The initial source audit separately reports the original source omissions.

## What the comparison means

- `pa`: native Gaussian exchangeable GEE; explicitly
  `estimates store pa, covariance(working)`. GEE retains both working and
  uncorrected unit-sandwich inference, with no ambiguous selected default.
  `covariance(sandwich)` is separately supported for reporting, multiplier1,
  normal z (not CR1/G−1 t). Plain GEE store errors with the two choices;
  selectors on non-GEE fits fail atomically. Negative admissible correlation
  is not forced into a positive-variance random-effects model.
- `fe`: model-based within FE, using its existing observation-weighted zero-
  effect normalization and joint common-intercept/slopes covariance. Slopes
  are copied; the common intercept uses the same retained linear-combination
  inference as native FE reporting. Student t(N−G−6)=t(4806), not pooled t.
- `refgls`: model-based explicit **equal-group-between** RE-GLS, not default
  unbalanced Swamy–Arora. Copy native selected beta covariance/normal z.
- `remle`: model-based Gaussian marginal ML; copy native observed-information
  beta covariance/normal z and actual marginal log likelihood. Zero-unit-variance
  boundary fits remain labelled ML, not silently replaced by OLS or GEE.

FE/GEE `_cons` is canonically aligned to `intercept` without changing matrix
order. Conflicting predictor names are rejected; rename/refit rather than merge
ambiguous terms. Normalized FE and population-mean intercepts have different
conventions. Optional snapshot `panel` provenance records entity, optional key,
group count and intercept convention; it is not a dataset fingerprint.

The OLS-labelled common R² row is **NA for all four models**. No within/conditional/
quasi-GLS R² is substituted; GEE/FE/GLS do not acquire a fake likelihood. There is
no cross-family LR test, arbitrary model-selection test, survey or causal claim.
The script's earlier native fit outputs retain their own available diagnostics.
Historical ML coefficient/component-SD SE differences are not hidden by a common
inference label or widened tolerance.

## Run and independently audit

Build with the pinned toolchain (`export PATH="$HOME/.cargo/bin:$PATH"`):

```sh
cargo build --locked --bin tabdat-explore-rs
```

For precise DTA, from the repository root, in a fresh working directory:

```bash
repo="$PWD"
raw=/path/to/vietnam_ex1.dta
work=$(mktemp -d)
Rscript "$repo/examples/mma24p1olscluster/dta_reference.R" "$raw" "$work/vietnam.csv" --precise
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma24p1olscluster/household_mixed_report.td" > result.jsonl)
ref=$(mktemp -d)
flags=(--dta)
for kind in gee fe re_gls re_ml; do
  extra=()
  if [ "$kind" = re_gls ]; then extra=(--equal-group-between); fi
  Rscript "$repo/examples/mma24p1olscluster/household_${kind}_reference.R" "$raw" "$ref/$kind" "${flags[@]}" "${extra[@]}"
done
python3 "$repo/examples/mma24p1olscluster/check_household_mixed_report.py" "$work" "$ref" "$raw" --dta
```

For ASCII, prepare `vietnam.csv` in a separate fresh `$work` using the household
README, set `$raw` to the pinned `.asc`, use `flags=()` in the reference loop and
omit the auditor's `--dta`. FE/GLS direct-DTA reference input uses
`convert.factors=FALSE`; do not round-trip through15-digit CSV to create a
supposedly independent DTA design. Set `R_LIBS` for a non-default validation
library if needed (R4.6.2, geeM0.10.1, Matrix1.7.6, foreign0.8.92 locally).
Reference source artifacts use17 digits so exact binary fields can be checked.
Reference quantity formulas are unchanged; the existing balanced CSV quantities
reproduce byte-for-byte after this source-artifact precision correction.

Use `--text` instead of `--json` for readable output. JSON's
`EstimateTableResult` retains every selected7x7 covariance matrix and full sample
provenance. Text keeps the existing coefficient/SE/statistic/p comparison with
model/covariance/reference/DF/N, conditional panel provenance and honest NA rows;
it is not a configurable publication layout or full-matrix text export.
`household_mixed_report_predictions.csv` contains selected source fields plus
`ml_xb` and `ml_residual=y−xb`, population predictions rather than posterior unit
effects. Existing output owners are never overwritten. Use a fresh directory
for each successful run; JSON/text runs export byte-identical predictions.

## Ownership, compatibility and failure behavior

Named snapshots are immutable reports, not restorable designs/models. No silent
refit occurs, including after current-data replacement or with no active data/
backend. Tables still work after the current fit is cleared. Candidate positions
refer to fit-time rows; equal N alone does not establish the same observations.
Later prediction uses the ordinary current active model, not a stored name.

Store/table/drop preserve the active model, data and labels. Unsupported
covariance/ambiguous names/malformed provenance fail before registry effects.
Old plain-store wire is unchanged; absent optional panel metadata decodes to
None and stays omitted. Old states lacking normalized FE inference or required
provenance require refitting. Existing registry bounds remain32 reports,
8 requested models and64 terms/union. Clustered FE/RE snapshots are not installed;
the native clustered fit/prediction workflows are unchanged.

Scripts are ordered, **not whole-script transactions**. Export failure preserves
its file owner and the last ML model, not a pre-script state. New-fit failures
retain established policies: GEE preserves its previous fit; failed explicit
FE/RE fit attempts clear the active fit. Neither deletes already stored reports.
No fit restoration, arbitrary named-fit post-estimation, new weights or general
survey inference is installed.

## Evidence and limits

Both pinned original modes pass **327 independent R reporting gates**, including
all four selected coefficient/SE/statistic/p vectors, full covariance matrices,
counts/DF and actual ML likelihood. All5999 source and5006 prediction rows are
audited, including exact source fields/keys/order/membership. Absolute1e-9,
nonzero-p relative1e-8 and exact-zero underflow; no tolerance widening.

[Three frozen controls](../../tests/fixtures/mma24_household_mixed_report/README.md)
cover balanced192, unbalanced199/200 with singleton, and positive-interior ML179/
180-source cases. The first two have negative admissible GEE correlation and
zero-boundary ML unit variance; the third has positive GEE correlation/interior
ML variance. Runtime snapshots pass1218 independent R gates (including the
separate GEE sandwich choice). Actual-script tests cover full typed reports/all
570 frozen prediction rows, JSON/text/valid export collision, labels/current-data/
ML ownership and nine raw-source/fit failure cases. Private tests reject24
malformed/weighted/negative-variance states without backend or registry effects;
wire/parser/regression checks preserve existing named model families. The optional
`python3 examples/mma24p1olscluster/test_check_household_mixed_report.py` rejects24
forgeries, including a shared native/reference wrong statistic.

Full Table24.4 parity remains **partial**: the printed historical ML SEs are
unresolved. No whole-chapter/book, platform or release claim follows from this
bounded workflow. See the inventory/coverage ledger for all still-required work.
