# Patents native shared-gamma RE whole-firm bootstrap (WIP contract)

## Outcome and selection rationale

Native gamma RE fits the patents sample, but its bootstrap rejects346 firms at
its256-group cap. After completing the native conditional-FE bootstrap, remove
this separate resampling barrier through one bounded original-data workflow.
Native normal-Poisson RE remains required but is a distinct higher-risk kernel
slice; do not label it complete or postpone it behind a full inventory gate.

Raise only shared-gamma RE bootstrap's group cap to512. Preserve B2..500,
1..14 slopes/K=slopes+2<=16, G>K, G*largest_group_size<=40000 and
G*largest_group_size*B*K²<=3000000000. Existing original-fit/raw-key/count/rank/
interior-dispersion/convergence guards, first-seen complete-case order, one
continuous xoshiro256++/SplitMix64/u64-rejection stream, distinct gamma-effect
copies, fail-abort/no redraw and read-only model/data/labels/reports remain.
Conditional FE already permits512; pooled Poisson keeps256. No formula, parser,
API/schema, RNG or dependency change is planned.

## Exact sample and statistical contract

CT2005 §23.3 patents `patr7079.asc`, SHA256
`87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8`.
Reuse [native gamma source guards](native_gamma_re.md):1730 records/346 firms,
five unique1975–79 years each; exact PAT0..1000, finite |LOGR|<=20 and finite
nonmodel metadata except nullable ARDSSIC. SUMPAT is not a selection filter.

Include all346 complete-case firms, including22 zero-total firms; unlike FE,
there is no zero-total exclusion. Intercept/LOGR slope and ln(alpha) are jointly
refit in500 whole-firm replicates, seed10001, with a separate mean-one shared
Gamma effect for each copied occurrence. Alpha is the effect variance, not a
row-NB2 dispersion model or Gaussian log effect. Full joint B−1 covariance,
bootstrap means/biases/SEs and type7 marginal percentile95 are for
`_cons,logr,lnalpha`; no new normal/t/bootstrap p-values or joint confidence
region. Finite B has Monte Carlo error and inference requires appropriate
independent-firm/regularity assumptions. Original actual gamma RE remains
installed for population n/xb/residuals; no bootstrap replicate is retained.

## Acceptance and failures, declared before implementation

1. Stable257/346/512-group kernel controls accept and513 rejects, preserving
   zero-total/singleton eligibility, full joint parameter/refit order and all
   conservative row/work budgets. Existing below256 RNG/results remain; test
   distinct copies against merged-ID fitting and fail-abort/boundary failures.
2. Actual original B500 script completes load/guard/cohort → fit → bootstrap →
   retained marginal predictions/diagnostics → readable/CLI-v1/export. All1730
   source/prediction records and22 eligible zero-total firms are verified;
   permutations/all-null nonmodel metadata, state/labels/reports and file
   collision/failure semantics retain their supported guarantees.
3. Reuse unchanged development-only `gamma_re_bootstrap_reference.R`, feeding
   independently regenerated native draws (including rejection-step indices).
   Compare all500 three-parameter R BFGS/Newton refits,31 summary fields/full3x3
   covariance and original predictions. Freeze an existing small synthetic
   source's500 R vectors/reference; leave publisher data external. Fixed gates:
   scalar1e-9+1e-10*abs(reference), exact keys/membership/draws/counts. Diagnose
   sample/parameterization/stream/convergence differences rather than widen.
4. Independently reconstruct empirical covariance/quantiles and raw shared-
   gamma likelihood/score identities so matching forged native/R covariance,
   coefficients or nuisance parameters cannot pass from equality alone. Test
   wrong zero-total eligibility, merged/repeated IDs, order, draws and ownership;
   failed/boundary/singular refits must abort, not omit/redraw or install a fit.
5. New/modified focused checks precede affected stats and relevant runtime/CLI/
   reporting/help checks. Reconcile SPEC/roadmap/ledger/inventory/help and old
   gamma256-limit notices; author-review complete diff and evidence, then merge,
   check tree identity and prune branches before another slice. No full-workspace
   pass inferred from retained/filtered runs or empty CI.

Scope exclusions (still required where applicable): pooled source-sized native
bootstrap, native normal RE/bootstrap, GEE bootstrap, general/weighted/offset
resampling, source named tables, historical RNG/repeated-ID/SE/interval parity
and complete illustration inventory. Existing SQL bootstrap stays distinct; its
merged-source-ID counterfactual is sensitivity evidence, not historical proof.

## Verified implementation checkpoint (not merge-ready)

Old executable reproduces original346-group rejection at256. Conditional FE
remains512 and pooled remains256; only gamma cap/error text changed. New native
original B500 completes, all173000 draw indices/rejection steps match independent
Python RNG, and unchanged R4.6.2 validates31 fields/full3x3 covariance/all500
three-parameter refits. Maximum intercept/slope/lnalpha differences are
9.34e-11/1.75e-11/4.24e-11 under the fixed scalar gates. Bootstrap SEs are
.42502508954787294/.12339690766905602/.22334598492348126. All1730 post-bootstrap
prediction rows are byte-identical to the independently audited native gamma
source-fit baseline, including all22 zero-total firms.

Modified/new kernel module7/7 passes:257/346/512 accept with zero/singleton
eligibility and joint refits;513 rejects; prior distinct-copy-versus-merged,
singular/boundary aborts and row/work contracts remain. Fresh affected stats
crate309 pass/one pre-existing skip,58 binaries,12.531s,
run8537766a-07a3-454c-b304-05b0421838e0. Relevant runtime contracts4/4 pass,
runfec98e13-2b9c-4aa6-855c-4f4eaedf93d2. No full-workspace/CI claim.

Pending: committed independent auditor/raw-score/empirical-forgery controls,
small-source frozen500-vector R fixture, actual-script integration/failure/
readable checks, lint/fmt/docs/help/ledger reconciliation and final author
review. Do not merge WIP or claim full bootstrap/book completion.
Artifacts `/tmp/tabdat-ch23-native-gamma-bootstrap/`.
