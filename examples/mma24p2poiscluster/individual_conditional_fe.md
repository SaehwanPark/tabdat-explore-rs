# Chapter 24 individual conditional Poisson fixed effects

This bounded workflow covers the individual FE column of Cameron–Trivedi (2005)
§24.7/Table 24.6 using native `xtpoisson y xvars, fe id(group)`. Source provenance,
SHA256 hashes, preparation and numeric-column interpretations are in
[README](README.md). It is not a complete table, Poisson RE, complex-survey or
causal analysis. The separate pooled workflow and Chapter23 scalar SQL
conditional-FE composition remain intact.

## Sample and statistical contract

Outcome `PHARVIS`; predictors, in order: `LNHHEXP INSURANCE SEX AGE MARRIED
ILLDAYS ACTDAYS INJURY ILLNESS EDUC`; grouping variable `COMMUNE`. These are
numeric columns, not factor expansions. Source rows identify cross-sectional
people, not temporal repeated measurements.

The script audits all 27,766 source records, then removes the one entirely
missing row (27,734). Native joint-finite selection precedes exclusion of
zero-total groups. Commune 64 contributes 94 all-zero observations, leaving
**27,671 people / 193 positive-total communes**, group sizes 51–206 and total
count 14,209. The export retains all 27,765 nonmissing source records, including
excluded commune 64, with explicit inclusion flags and selected commune totals.

The likelihood is multinomial conditional on each selected group count total:
`sum_g [lgamma(S_g+1) - sum_i lgamma(y_i+1) + sum_i y_i*log(p_i)]`, with
`p_i=exp(x_i*b)/sum_j exp(x_j*b)` within group. There is no common intercept.
Covariance is inverse conditional information, with no finite-sample multiplier;
coefficient inference is asymptotic normal95. The conditional same-total
zero-slope null supplies an LR chi-square test; a separate joint slope Wald
chi-square test is reported. Positive singleton groups remain but contribute
no slope information. No robust, clustered or survey covariance is implied.

Exact group keys are structural and audited even on omitted model rows. Counts
on joint-finite cases must be exact integers 0..100000. Candidate rows <=40000,
K 1..32, rows*K² <=20 million, candidate groups <=10000 and retained G>K are
required. Group-anchored/RMS-scaled Newton fitting uses finite positive information,
line-search safeguards and both scaled score/step tolerances 1e-10; failures
are explicit, not clipped estimates or silently dropped unidentified predictors.

## Execute and independently reproduce

Build the native binary with `cargo build --locked --bin tabdat-explore-rs`.
From an empty working directory, with `repo` set to the repository path:

```sh
python3 "$repo/examples/mma24p2poiscluster/prepare.py" /path/to/vietnam_ex2.asc vietnam_individual.csv
"$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma24p2poiscluster/individual_conditional_fe.td" > result.jsonl
Rscript "$repo/examples/mma24p2poiscluster/conditional_fe_reference.R" /path/to/vietnam_ex2.asc reference
python3 "$repo/examples/mma24p2poiscluster/check_conditional_fe.py" . reference /path/to/vietnam_ex2.asc
```

Use a separate empty directory for `--text`; both modes export byte-identical
`individual_conditional_fe_predictions.csv`. R and Python are optional preparation
and validation tools, not production dependencies. The independent reference
fits an R dummy-Poisson GLM, then cross-checks its slope covariance and fitted
means with direct conditional score/information and likelihood calculations.
It does not call the Rust optimizer.

The pinned ASCII audit checks **179 fit/covariance/inference quantities**, every
source record and every prediction/exclusion record. Fixed tolerances: absolute
1e-9, conditional log likelihood 1e-8, nonzero-p relative 1e-8, zero underflow
exact. Maximum observed absolute difference was below 2.529e-10 (Wald); maximum
relative p difference was below 2.981e-11. Native fitting took seven information
evaluations, ending with scaled score 9.218e-12 and step 4.371e-14.

For a separate historical precision check, convert the pinned DTA with
`Rscript dta_reference.R input.dta prepared.csv`, use that CSV as
`vietnam_individual.csv` in another empty native work directory, and generate a
new reference with `conditional_fe_reference.R prepared.csv dta-reference --csv`.
Then run:

```sh
python3 "$repo/examples/mma24p2poiscluster/check_conditional_fe_author.py" /path/to/vietnam_ex2.dta /path/to/mma24p2poiscluster.txt dta-reference /path/to/dta-native/result.jsonl
```

Both R and native DTA results match all **20 printed author coefficient/SE
facts**, using half of each last printed digit, not the stricter ASCII tolerance.
This does not assert full Stata optimizer, table or likelihood-output parity.
Fourteen forged ASCII outputs and two forged native-author outputs were rejected.

## Owned state, prediction and failure behavior

`predict ..., xb` is the slope-only index. `predict ..., n` and `residuals` use
the fitted positive group's retained normalization evaluated at current X, not a
fresh normalization of the current subset. Therefore totals are reproduced on
the estimation design, not promised after arbitrary data changes. Unseen,
missing and zero-total IDs return NULL means/residuals; `xb` does not require an
ID. Current missing/nonfinite X yields NULL. Residuals require a finite current
outcome. Group logical type must match the fitted type. These are conditional
fitted-group predictions, not population/new-group forecasts or unit-effect
uncertainty. Prediction work is separately bounded and failure is atomic.

Failed new fits clear stale models without mutating data; parser failures
preserve the previous model. Reporting, successful export and export collision
retain the fitted state. Source and existing export-owner files are preserved.
Scripts are intentionally nontransactional: a failed final export does not undo
earlier tables/model effects. Run in a fresh directory or choose another output
path rather than deleting someone else's file.

Tests include a 149-record independent R synthetic fixture (148 complete,
140 retained /18 positive groups), ten slopes, unbalanced/interleaved groups,
positive singleton, zero group and missing case; closed-form and group-shift/
scaling controls; invalid keys/counts, rank/separation/iteration failures;
retained normalization, changed-data/type/target/work-bound failures and actual
script export-state checks. Regression and validation commands are recorded in
the PR; hosted CI and independent code review are not claimed.

The separate [shared-gamma RE workflow](individual_gamma_re.md) now supplies
model-based individual RE, including zero-total communes and owned marginal
predictions; it does not add clustered FE or survey inference.

The separate [conditional-FE whole-commune bootstrap](individual_conditional_fe_bootstrap.md)
refits all ten slopes on each sampled group copy and does not replace the retained
FE fit. A separate [shared-gamma RE bootstrap](individual_gamma_re_bootstrap.md)
uses distinct copied effects. Still required: other GEE workflows, genuine survey
weights/strata/design semantics, remaining household outputs and remaining
table/chapter illustrations. Weights/offsets, robust/cluster covariance, factors,
generic post-estimation/margins and unit-effect uncertainty are unsupported in
this bounded estimator, not removed from the project's required roadmap.
