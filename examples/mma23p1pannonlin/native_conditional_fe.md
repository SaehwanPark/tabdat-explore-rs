# Chapter23 patents: native conditional Poisson firm FE

This bounded companion to the [SQL composition](conditional_fe.md) runs the
installed `xtpoisson pat logr, fe id(id)` and retains actual owned conditional-FE
state for later predictions. It validates Cameron–Trivedi (2005), §23.3/Table23.1's
model-based conditional-FE portion against the unchanged independent R oracle.
No production Rust kernel/parser/API/runtime/dependency change was needed.
It does not complete Table23.1, the nonlinear-panel chapter or the book inventory.

## Sample, model and inference

The pinned `patr7079.asc` SHA256 is
`87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8`.
Source ordinal ID is not CUSIP. All1730 observations/346 firms/five years1975–79
remain in the exported source; nullable nonmodel ARDSSIC and SUMPAT metadata
never select the fit. The22 zero-total firms/110 observations carry no slope
information and are excluded, leaving **1620 observations/324 positive-total
firms**. All model fields must be available in this source-specific workflow;
there is no additional complete-case loss. One LOGR slope, no common intercept,
year dummies, weights or offsets.

Condition on each positive firm's selected count total S_g. With
p_it=exp(b*x_it)/sum_s exp(b*x_is), the conditional likelihood is multinomial and
includes log(S_g!)-sum_t log(y_it!) constants. Native variance is inverse
conditional information; normal95/z/two-sided p, same-selected-sample zero-slope
null/LR and slope-Wald chi-square inference are distinct reported quantities.
This is **model-based**, not cluster-robust, survey-design or causal inference.
No nuisance-unit-effect uncertainty or unconditional/new-firm forecast is implied.

| Original float64 source | Native value |
| --- | ---: |
| LOGR coefficient | −0.0377642353572117 |
| Standard error | 0.03255176135039196 |
| Conditional log-likelihood | −3659.592559128101 |
| Native updates | 3 |

Four selected printed author facts (coefficient−.0377642, SE.0325518, p.246,
likelihood−3659.5926) pass half-last-printed-digit gates. This is not whole-output,
optimizer/stopping, bootstrap or historical platform parity.

## Guarded source script and prediction semantics

`native_conditional_fe.td` loads canonical `patents.csv`, validates unique ID/year
keys, BIGINT IDs1..346 and years1975..79, exactly five distinct years per firm,
finite integer PAT0..1000, finite |LOGR|<=20 and finite numeric metadata except
nullable ARDSSIC. It retains2..346 firms, computes selected totals/membership in
canonical ID/year order, then invokes native FE. Native strict identification/
convergence/resource checks remain authoritative; at least two positive-total
firms and full-rank within information are required. All-zero outcomes, a single
positive group, constant within-X, bad keys/years/counts/metadata fail rather than
produce a false native result or prediction/export. The old SQL slope bracket
and |z|<=3 p-value withholding are **not** imposed on the native estimator.
The existing R oracle's scalar bracket validates the supplied original/controls,
not every custom dataset accepted by the wider native method.

The final active table is `native_conditional_fe_cohort`, exported without
clobbering as `native_conditional_fe_predictions.csv`. Its columns are the nine
unchanged originals (`id,year,cusip,ardssic,scisect,logk,sumpat,logr,pat`) followed by
`selected_firm_total,included_in_native_fe,native_fe_mean,native_fe_xb,native_fe_residual`.
Summaries use1620 defined means/residuals; exported source and finite slope-only
indices still cover1730 observations.

For positive groups, native mean is S_g*exp[b*(x_current-c_g)] divided by the
**retained estimation-design** sum_s exp[b*(x_fit,s-c_g)], with a saved anchor c_g.
Thus estimation-design predictions reproduce each positive group's observed
total; arbitrary current subsets do not get re-normalized to the full total.
The group is required for n/residuals; slope-only xb=b*x needs no current ID.
Current outcome is needed only for residuals. Unseen/missing/zero-total IDs have
NULL means/residuals; their complete-X indices remain defined. Group logical
types must match. The existing SQL/R composition already has NULL excluded
means/residuals too; the earlier handoff's claim that it used zeros was incorrect.

Fit/prediction/report/export retain actual ConditionalPoisson state. Read-only
summaries preserve labels/model. Existing destination columns and export owners
are not overwritten. A prediction error preserves state; a failed new fit clears
stale estimation while preserving data/labels/stored reports, per existing count
handlers. The script is ordered, **not whole-transactional**: an export collision
does not roll back an earlier valid fit/predictions/tables. Use a fresh output
directory or choose a new destination rather than deleting someone else's file.

## Independent reproduction

R/Python are optional development tools, not production dependencies. The
unchanged R generator uses base R `glm` with firm dummies plus an independent
conditional-score root/information check; no additional R packages are needed.

```sh
repo=/absolute/path/to/tabdat-explore-rs
work=/absolute/path/to/fresh-work-directory
mkdir "$work"
curl -fsSL https://cameron.econ.ucdavis.edu/mmabook/patr7079.asc -o "$work/patr7079.asc"
(cd "$repo" && cargo build --locked --bin tabdat-explore-rs)
sh "$repo/examples/mma23p1pannonlin/prepare.sh" "$work/patr7079.asc" "$work/patents.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma23p1pannonlin/native_conditional_fe.td" > result.jsonl)
Rscript "$repo/examples/mma23p1pannonlin/conditional_fe_reference.R" "$work/patr7079.asc" "$work/r-reference"
python3 "$repo/examples/mma23p1pannonlin/check_native_conditional_fe.py" "$work" "$work/r-reference" "$work/patr7079.asc"
```

Fresh R4.6.2 reproduces all14 primary fields, including all sample/exclusion
counts, slope/SE/z/p/CI, conditional likelihood and scalar information. Native
and R comparisons use fixed `1e-9+1e-10*abs(reference)`, with nonzero p relative
1e-8; no tolerance widening. Every original field/member/key/order and all1730
mean/index/residual defined/NULL records are audited against raw source and R.
Text/JSON source exports are byte-identical. Independent raw-source equations
check score, scalar information*covariance identity, conditional/null likelihood,
LR/normal/Wald tails, normalized means, residuals, positive-group totals and
summary statistics. Score tolerance is1e-8+1e-10*sum(abs(group scores)). Anchored
log-softmax evaluates likelihood directly even when a zero-count cell's
probability underflows, without log(0) or clipping.

## Focused tests and verification scope

- Five actual-script Rust tests pass:14 frozen R fields/all60 synthetic source/
  prediction rows/10 NULL exclusions; JSON/text equality, permutation and
  all-null ARDSSIC;27 invalid/singular inputs; actual model/labels/repeated
  predictions and retained-subset normalization without current y; unseen/no
  IDs, prediction atomicity/failed-fit invalidation and file/column ownership.
- Three Python tests pass: actual controls, sign reflection of the frozen R
  model (an exact identity, **not** a claimed fresh independent second R fit),
  permutation/null metadata;31 rejected forgeries; constructed finite-likelihood
  identity under probability underflow. Shared wrong native/R covariance with
  consistent SE/z/p/CI, likelihood, scaled means and total-preserving false
  within-firm allocation are rejected by raw equations, not artifact equality.
- Commands: `cargo nextest run --locked -p tabdat-explore-rs --test
  mma23_native_conditional_fe_workflow --no-tests fail`; `python3 -m unittest
  discover -s examples/mma23p1pannonlin -p test_check_native_conditional_fe.py -v`;
  scoped new-root-target Clippy `-D warnings`, fmt and diff checks pass.
- Main0ecc960's fresh2105-test/393-binary/two-pre-existing-skip workspace run
  remains evidence for unchanged production/consumers/old tests; the five new
  Rust cases pass separately. This is **not a fresh2110-case broad run**.
  Only this independent source guide/script/auditor/tests and documentation
  links changed; no shared/API changes warrant rerunning the unchanged baseline.
  No Rust documentation example changed; doctests not rerun. Hosted CI is
  disabled; no CI/platform/release or independent code-review claim.

Evidence/logs are under `/tmp/tabdat-ch23-native-fe/` (original-reference,
workflows/original-json and original-text, root-focused.log, python-reviewed.log,
root-clippy.log). The probability-underflow auditor defect found in author review
was corrected with a dedicated constructed-identity regression; production
estimation behavior and fixed gates were unchanged.

## Separate required work

The [SQL whole-firm FE bootstrap](conditional_fe_bootstrap.md) remains a
validated separate stream. The [native B500 workflow](native_conditional_fe_bootstrap.md)
now admits the original324 positive firms under its conditional-only512-group
cap, validating all500 R refits/162000 independent draws and retaining the
original predictions. This is resampling inference, not added analytic weighted/
offset/cluster FE covariance. Pooled/gamma bootstrap caps remain256. [Native gamma RE](native_gamma_re.md),
[native balanced Poisson GEE](native_poisson_gee.md) and the
[normal-RE composition](normal_re_poisson.md) remain different methods with
explicit assumptions/historical limits. General/unbalanced GEE, native normal
RE, normal-RE bootstrap, other required illustrations and historical source/
stopping/bootstrap reconciliation stay open. No unresolved required entry is
silently excluded or marked complete by this source replay.
