# Native patents Poisson GEE: preimplementation contract

**In progress, not implemented or validated as a native workflow.** High risk:
new nonlinear estimating-equation kernel, typed model state and additive public
command/result contracts. Current [SQL composition](exchangeable_gee.md) is
independently validated; Gaussian native GEE must remain unchanged.

## Exact outcome and priority

Install a bounded native Poisson/log-link exchangeable GEE for Cameron–Trivedi
(2005) §23.3/Table23.1 patents, using the existing source/design and independent
R/SQL evidence. This advances the graduate nonlinear-panel core rather than
requiring every descriptive inventory entry first. Source load/audit → specify
native family/link/correlation/ID → fit → inspect both covariance inferences and
convergence → owned mean/index/residual prediction → report/export/reproduce.
The retained model must actually be Poisson GEE, unlike the composition's pooled
Poisson starting state. Do not add another SQL-only or parser-only milestone.

## Sample, model and supported limits

Pinned `patr7079.asc` SHA256
`87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8`;
existing prepare.sh gives1730 firm-years/346 firms,1975–79, all22 zero-total
firms retained, no missing nonmodel ARDSSIC filter. Mean exp(a+b*LOGR), intercept,
no other regressors/year effects/weights/offsets in the exact illustration.

Generic first native scope: unweighted intercept,1..32 numeric slopes, selected
integer counts0..100000, rows<=40000, rows*K²<=20million (K=slopes+1), at most
10000 raw groups and selected G>K. Use established Gaussian GEE grouping and
joint-finite sample/key semantics; capture exact original retained indices and
dropped count. Missing rows must not be silently made into complete firms.
**Selected groups must be balanced with a common T>=2**; reject unbalanced,
singleton-only, weighted/no-intercept, rank-deficient or unsupported data.
Source workflow additionally validates all five unique years per loaded firm,
counts<=1000 and |LOGR|<=20, rejecting missing model fields to retain all rows.
Finite numeric/text/boolean group handling follows existing GEE; do not invent
a calendar-time interpretation for exchangeable IDs.

Both zero-total and positive-total selected groups remain. This is not conditional
FE, gamma/normal RE, independent-row NB2, survey inference or a joint count
likelihood. No GEE LR, pseudo/McFadden R², PMF/CDF, nuisance-correlation SE,
posterior effects, offsets/weights, alternative links/correlations, unbalanced
moment generalization, automatic bootstrap or named snapshots in this slice.

## Estimating equations and inference

For each group, eta=X*beta, mu=exp(eta), Pearson e=(y−mu)/sqrt(mu).
Legacy balanced normalized-Pearson moment:

```
S2 = sum_i sum_t e_it²
SG = sum_i (sum_t e_it)²
rho = (SG−S2) / ((T−1)*S2)
```

Working variance scale is **fixed1**, not estimated S2/N. S2/N is a separate
correlation normalizer; label the distinction explicitly. With
R=(1−rho)I+rho*11', A=diag(mu), V=sqrt(A) R sqrt(A), D=A X:
U_i=D_i'V_i^-1(y_i−mu_i), U=sum_i U_i,
B=sum_i D_i'V_i^-1D_i. Use pooled Poisson only as an internal starting point,
then full Fisher-scoring updates B^-1U with rho recomputed from current beta.
B is **expected GEE sensitivity**, not an observed joint likelihood Hessian or
an arbitrary observed derivative of U. Do not add rho-derivative information.

Return both full raw-parameter covariances and normal95/z/two-sided p/slope-Wald
chi-square inference with explicit references:

- Working: B^-1, fixed Poisson variance scale1.
- Group sandwich: G/(G−1) * B^-1(sum_i U_i U_i')B^-1. No extra (N−1)/(N−K),
  G−1 t reference or alpha=0 LR. Existing Gaussian sandwich stays multiplier1.

The robust interpretation needs independent groups and correct mean, not merely
integer counts. Working inference additionally needs its variance/correlation
assumptions; neither supplies few-group reliability or causal interpretation.

Numerics: RMS-scaled design/established positive-information solves; centered
within/between group equations or symmetric whitening avoid subtraction of
near-equal rank-one terms. Compensate sums where material. Evaluate rho boundary
gaps using sums of squares (lower SG/S2, upper within-group Pearson variation),
requiring gaps >128*EPSILON on dimensionless propagation scale; reject zero or
roundoff-dominated Pearson scale and sandwich variance. No clipping, ridge,
pseudoinverse, dropped groups or failed iterate as success. Bound eta to[-40,40]
and100 full steps; reject escaped/nonfinite updates rather than silently change
algorithm. Require final scaled coefficient step<=1e-12 and score<=1e-10;
record both. No global-root/optimality claim. Safeguards may reject numerically
marginal but mathematically admissible inputs; document them.

## Typed boundaries and compatibility

Syntax adds exactly:
`xtgee y xvars, family(poisson) link(log) corr(exchangeable) id(group)`.
Keep the existing Gaussian `Command::XtGee { XtGeeCommand }` wire/fields intact.
Prefer an additive `Command::XtGeePoisson { command: XtGeeCommand }` for the new
family rather than a mandatory field breaking old Gaussian serialization.
Both canonical command names remain xtgee. Add separate PoissonGee result/state/
family variants and normal report metadata; freeze old Gaussian round trips.

Use `PoissonGeeOptions/Result` in tabdat-stats with pure owned inputs/results,
no I/O. Reuse established matrix/count/normal inference helpers when semantically
correct. A Poisson inference type alias to existing covariance-only normal GEE
inference is acceptable; do not rename Gaussian public types or change its
moments/correction. All metadata must identify Poisson/log, legacy balanced
moment, fixed working scale1 versus Pearson normalizer, G/(G−1) and actual sample.

Execution uses DuckDB design/group preparation and existing typed errors.
Native failed **new fits clear stale estimation** but preserve data/labels/stored
reports. New command classification must participate in the same policy.
Predictions use owned coefficients: `n`=marginal mean, `xb`=eta,
`residuals`=current y−mu, without requiring current group IDs. Atomic destination
creation, no-clobber columns/files, no current-data/model mutation on failed
prediction. Reject count probabilities/intervals/std/saving; no silently using a
pooled or Gaussian model. Explicitly reject unsupported named-store/covariance
requests without registry changes; do not extend Gaussian-only snapshot selection
implicitly. Whole source script remains ordered, not transactional.

## Five acceptance cases and independent references

1. Original1730/346/22 exact source/prepared/sample/prediction records: legacy
   R gee4.13-30 (reported version4.13.30), v4.4compat=TRUE, scale.fix=TRUE,
   scale.value=1, pooled start, tol1e-12/maxiter100. Preserve existing27 report,
   346 score and every mean/residual gate: `1e-9+1e-10*abs(ref)`; score1e-8 plus
   relative1e-10 per existing convention. New native p gates relative1e-8 for
   nonzero tails (steep normal-tail sensitivity near |z|18), predeclared before
   kernel output; all other new scalars same existing absolute/relative rule.
   Independently evaluate matrix equations in base R in addition to package fit.
2. Existing positive/negative-rho12-firm controls: full matrices/inference and
   every fitted/source row/group score; supplement a balanced multivariate and
   rescaled-predictor dense-R fixture so general K is not inferred from K2.
   Independent QR/Cholesky or direct dense blocks, not native-derived goldens.
3. Actual CLI/script text/JSON on original/control; byte-identical exports after
   canonical sorting, source input unchanged, zero firms retained and current
   state actually PoissonGee. Repeat prediction/labels/read-only summary and
   owner/export failure preserve the established state invariants.
4. Nonintegral/negative/all-zero counts, raw invalid keys/caps, joint missing
   sample/unbalanced result, no-intercept/weights/rank and exact/near rho/scale/
   sandwich degeneracy and forced nonconvergence fail explicitly. Failed refit
   clears stale model only; failed prediction/parser/store preserves prior state
   and registry as appropriate, no half destination or false fit report.
5. Freeze parser/domain wires and API/help/catalog/JSON/text contracts, including
   Gaussian legacy unchanged. Real-script source/report/sample/covariance and
   shared wrong native/R artifact forgeries rejected by independent identities.
   Historical printed slope .5595302 versus converged .5595300704 remains separate
   unresolved precision/stopping evidence; never change tolerance to claim parity.

## Ordered implementation and checks

1. Before kernel: record author plan review of family/link/wire/moment/scale,
   expected sensitivity versus observed Hessian, sample balance and failure policy.
   Reproduce existing package/base-R references; freeze supplemental dense R
   fixtures and direct numerical/failure tests. No output from native as oracle.
2. Implement `tabdat-stats/src/poisson_gee.rs`/exports, focused stats tests first;
   keep helpers private/small, reuse existing module/library, safe Rust only.
3. Add domain command/state/result variants and wire tests; language parser/tests;
   execution fit/predict dispatch and state policy; reporting catalog/help/terminal/
   JSON and exact reverse-consumer tests (especially discovery goldens). Complete
   native source script/auditor/forgery/runtime/root workflow tests before merge.
4. Run new/modified focused tests explicitly, affected crates and consumers,
   independent original/control checks, fmt/scoped Clippy/diff. Cross-cutting
   additions warrant current full-workspace regression before completion; no
   empty/filtered/timeout selection is a passing broad check. Doctests only if
   Rust examples change. Update SPEC/roadmap/ledger/source docs after evidence.
5. Review final code/PR and verification evidence, push/merge only the whole usable
   slice, verify merged tree and prune feature branches before another slice.

Implement exactly this plan. Do not broaden scope. Stop/report source/reference,
statistical or supported-contract conflicts rather than silently substituting a
method. New dependencies, unrelated generic-GEE frameworks, guessed historical
behavior and broad refactors are excluded. Report changed files, exact checks,
deviations, limitations and the next priority; required inventory remains open.

## Pre-kernel author plan review

Reviewed the contract against Gaussian stats/execution/parser/state/capabilities
and the package/base-R source equations. No independent code review is claimed.
Resolved before implementation: (1) separate variance scale1 from correlation
normalizer S2/N; (2) B is expected sensitivity, so finite-differencing observed U
must not incorrectly substitute its derivative as bread; (3) Gaussian command
wire stays unchanged via additive typed dispatch; (4) the new command must join
failed-fit invalidation/capability classification, not leave stale Poisson state;
(5) selected balance is checked after joint-finite selection, with source-all-row
membership separately enforced; (6) no PMF/LR/named snapshot is inferred from a
Poisson working family. Generic unbalanced legacy moments and nuisance inference
remain excluded. Supplemental multivariate/rescaling reference and direct failure
fixtures are the next gate before kernel implementation.

## Reference setup evidence (pre-kernel)

Current R4.6.2/gee4.13.30 reproduces the existing original source oracle under
`/tmp/tabdat-ch23-poisson-gee/original-reference`; no native kernel implemented.
Optional development-only source archive SHA256
`65f9fe9c906a200bd89a6c053e8c33cce356b626a0de639cc69d804bc3326b95`.
No production R/Fortran dependency. On this host R's FLIBS referenced Darwin27,
but installed gfortran16.2 resolves its private libraries under Darwin25; default
install failed with missing emutls_w. A per-install R_MAKEVARS_USER FLIBS override
using gfortran `-print-file-name` paths installs successfully without editing
global R configuration or changing package code. Both logs are preserved in the
temporary evidence directory. If this exact Homebrew/macOS mismatch recurs,
compare `R CMD config FLIBS` with `gfortran -print-file-name=libemutls_w.a` and
`gfortran -print-file-name=libgfortran.dylib`; use the compiler-resolved directories
in a temporary Makevars, never edit global R Makeconf. Check both files exist
before replacing FLIBS; this is not a workaround for arbitrary linker failures.
Do not hardcode these host paths in production.

Fresh positive/negative12-firm package/base-R runs match every existing frozen
report/prediction/score field. Separate [supplemental R generator](native_poisson_gee_reference.R)
now freezes [multivariate/rescaling controls](../../crates/tabdat-stats/tests/fixtures/poisson_gee/README.md):
60 rows/12 groups, three coefficients,66 fields/both full3x3 matrices, all
predictions/three-vector group scores per case, explicitly17 digits. Dense block
V/score/bread solutions agree with R gee; natural coefficient/SE/CI/full-covariance
rescaling identities pass. These are pre-kernel oracles, not native evidence.
Direct native numerical/failure tests, kernel and the remaining integration/
workflow checks are still pending. Do not merge this reference-only checkpoint.

I-DETAIL/frequency/graphics, historical ML/URBAN/MEDEXP/LNHHEXP discrepancies,
remaining normal-RE/bootstrap/method illustrations and full book inventory remain
required. This contract claims neither implemented native Poisson GEE nor full
chapter/graduate-method/book completion.
