# Native patents conditional Poisson FE: bounded contract

**WIP: native method is installed, but this Chapter23 source replay is not yet
implemented/validated.** Deliver an actual owned native fit rather than another
SQL-only solver. No production kernel/parser/runtime/API/schema/dependency change
is planned. Existing source/reference artifacts and Chapter24 native semantics
are authoritative. Risk is moderate: source selection, likelihood conventions,
retained normalization and file/state failures, not a new estimator.

## Exact outcome and statistical sample

Cameron–Trivedi (2005) §23.3/Table23.1 conditional-FE Poisson column, supplied
`patr7079.asc` SHA256
`87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8`.
Load/audit canonical prepared source → specify native `xtpoisson pat logr, fe
id(id)` → inspect model-based inference/exclusion/convergence → owned n/xb/
residuals → summaries, scope, text/JSON and no-clobber export/reproduction.

Original1730 firm-years/346 firms/1975–79;22 zero-total firms/110 rows excluded
from slope estimation, retaining1620 rows/324 positive-total firms. Intercept is
conditioned out; exactly one LOGR slope, no year effects, weights or offsets.
ID is source ordinal, not CUSIP. All original fields/rows, including nonmodel
missing ARDSSIC, stay in sorted source export with selected totals and membership.
SUMPAT is source metadata, not an estimation filter or substitute selected total.

The native model conditions on selected positive-group count totals. Full scalar
inverse conditional information, normal95/z/two-sided p, conditional multinomial
log likelihood (including factorial constants), same-sample zero-slope null/LR
and chi-square Wald are available. No clustered/robust/survey/causal or nuisance
intercept uncertainty. Native numerical method/limits remain unchanged; do not
impose the old SQL scalar bisection bracket or p-withholding convention on it.
Source additionally guards BIGINT id1..346/year1975..79, all five unique years,
finite integer PAT0..1000, finite |LOGR|<=20 and finite metadata except nullable
ARDSSIC; at least two positive-total groups must survive. No model-field missing
case is admitted in this source-specific workflow.

**Correction to preceding handoff:** the existing SQL composition and its R
reference already use NULL means/residuals for excluded zero-total groups, not
zeros. Preserve this existing convention; do not change production to satisfy
the mistaken handoff. Slope-only xb is finite on all complete-X source rows,
including excluded groups. Native n/residuals use each fitted positive group's
retained anchor/normalization, not a fresh normalization of the current subset.
Totals reproduce on the estimation design, not after arbitrary data changes.
Unseen/missing/zero-total IDs have NULL n/residuals; xb needs no current ID.
Current y is needed only for residuals. Group logical types must match.

Retained state must be actual ConditionalPoisson, not pooled Poisson or a report.
Failed new fits clear stale estimation while preserving data/labels/stored reports;
parser/prediction errors preserve prior state. Prediction/column and export/file
collisions are atomic. Script is ordered, not whole-transactional. No automatic
bootstrap, named snapshots, unit-effect uncertainty or new general capabilities.
Existing SQL source/bootstrap and all unresolved historical/table/book requirements
remain separate; no whole-column/table/chapter/book parity from this bridge.

## Five acceptance cases

1. Actual original source: existing unchanged R dummy-GLM/direct conditional
   reference14 quantities, full scalar covariance/information, all source fields/
   membership/prediction records, exact exclusion counts/IDs and positive-group
   total identities. Independently recompute score, conditional/null likelihood,
   LR, normal/Wald tails and conditional normalized means from raw source. Fixed
   scalar `1e-9+1e-10*abs(ref)`, nonzero p relative1e-8, score1e-8 plus existing
   scale-relative propagation guard. No tolerance widening or native oracle.
2. Existing12-firm synthetic source/reference plus positive/negative slope or
   near-bracket controls where useful: included/excluded samples, independent
   full information/mean checks, canonical permutation and all-null ARDSSIC.
   Reuse existing inspectable fixtures/R generator; do not expand public APIs.
3. CLI text/JSON actual script and byte-identical source exports; every original
   field/order/member and defined/NULL pattern checked. Source output has nine
   original fields then selected_firm_total,included_in_native_fe,native_fe_mean,
   native_fe_xb,native_fe_residual. Final table native_conditional_fe_cohort.
4. Owned repeated predictions/labels/read-only summaries; current subset preserves
   fitted-row mean values (not re-normalizing); xb with no current ID, n needs
   retained matching groups, unknown IDs/zero groups are NULL; no accidental refit.
   Export/column owner collisions preserve state/files and source bytes. Preserve
   established failed-fit behavior rather than invent a source-specific policy.
5. Exact invalid keys/counts/metadata/missing years/all-zero/rank/single-positive-
   group failures: no false ConditionalPoissonResult or partial export/prediction.
   Auditor rejects forged source/member/NULL/mean and self-consistent wrong
   native/R covariance/SE/CI, probability, likelihood and group-normalization
   artifacts through additional raw-source equations, not equality alone.

## Ordered implementation and evidence

1. Before source implementation, inspect native/SQL/R sample and normalization
   contracts and establish reference feasibility. Fresh R4.6.2 original oracle
   already reproduces1620/324 and14 quantities under
   `/tmp/tabdat-ch23-native-fe/original-reference`. Existing old SQL/R NULL pattern
   and native Chapter24 semantics agree; the handoff's zero claim was corrected.
   An ad-hoc existing-native CLI probe passes all14 original R quantities, all1730
   key/index/mean/residual records and exact110 NULL exclusions before writing
   the guarded source companion. Native slope−.0377642353572117,
   SE.03255176135039196, conditional LL−3659.592559128101;three updates.
   This establishes feasibility, not committed source guards/auditor/failures.
2. Open draft PR early; implement one guarded source script/guide/auditor and
   forgery checks, plus focused actual-script Rust acceptance/failure tests.
   Existing native estimator, SQL/R source solver and fixtures stay unchanged.
3. Run newly added/modified tests explicitly first, independently audit original
   and controls, then affected/reverse-consumer checks. Current main0ecc960 has
   a fresh2105-case/393-binary workspace pass/two pre-existing skips; do not rerun
   unchanged baselines without an affected boundary. If production/shared code
   must change, stop to reassess risk/reference/scope and broaden validation.
4. Reconcile SPEC/roadmap/ledger/SQL guide without closing required whole-table
   or historical gaps. Review final diff/evidence, commit/push, merge only the
   complete usable source workflow and verify merged tree/prune both branches.

Implement exactly this bounded source bridge. Do not broaden into native
bootstrap/general estimator changes or silently change retained means, likelihood
constants, exclusions, error state or reference tolerance. Stop/report conflicting
source/native semantics or unavailable essential evidence. Report changed files,
checks, deviations and remaining requirements. Overall goal/inventory stays open.
