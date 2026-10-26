# Table5.7 WNLS and FGNLS: bounded native-composition contract

Status: **planning/reference feasibility only; not implemented or validated as a
CLI workflow**. Starts from merged unweighted NLS (`1aa5060`, PR340); do not repeat
that slice or claim the remaining table/method family is complete.

## Exact outcome and source

Implement the two weighted NLS columns of Cameron–Trivedi (2005), §5.9.2–3,
pp159–163/Table5.7 using original generated10000-row `mma05data.asc` and
`mma05p2nls.do/.txt`. Reuse [pinned preparation/source identity](README.md), native
`nl`, typed prediction, SQL and export. No general weights API, new estimator
framework, production dependency, wire/result change or kernel change expected.

The source freezes preliminary means **once** from its unweighted fit:

- Original mean `mu(theta,x)=exp(-a-b*x)`, parameter order a,b; starts1,0.
- Preliminary `mu0=mu(theta0,x)` from the initial unweighted NLS, starts1,0.
- WNLS: fixed `w=1/mu0`; FGNLS: fixed `w=1/mu0^2`.
- Each final fit independently starts1,0. **No iterative weight updates.**
- Sample is the same ordered, unique source_row1..10000, complete finite x,y,
  positive generated y. The offline script control supports3..10000 such rows,
  subject to existing regularity/convergence/representability restrictions.
  No row may silently disappear in a later stage.

## Native composition and statistical meaning

Freeze the preliminary prediction column. Set `s=1/sqrt(w)` and fit the
transformed response `z=y/s` to `m(theta)=exp(-a-b*x)/s` with existing unweighted
`nl`. Thus sWNLS=sqrt(mu0), sFGNLS=mu0 and
`sum((z-m)^2)=sum(w*(y-mu)^2)` exactly in real arithmetic. Compute square roots
and derived design columns using existing SQL; do not expand Generate syntax.
The positive finite scales, weights and transformed values must pass an explicit
preflight before weighted fitting. Preserve source fields/order.

For fixed s, let r=(y-mu)/s, G_i=(-mu,-x*mu)/s and B=sum(G_i G_i'). At the
converged, full-rank, positive observed-loss-curvature minimum:

- Classical **working** covariance: `[sum(r^2)/(N-2)] * inverse(B)`.
- Independent-row HC1: `N/(N-2) * inverse(B) * sum(G_i G_i' r_i^2) * inverse(B)`.
- Existing approximate Student-t(N-2),95% intervals, no exact nonlinear t claim.
- Both modes must produce identical coefficients/means/residuals; the working
  covariance correction is1 and HC1 is N/(N-2).
- Multiplying all fixed weights by a positive constant leaves the solution and
  both covariance matrices unchanged (objective and reported residual variance
  scale). This explains Stata aweight normalization; do not compare its normalized
  root-MSE directly to native unnormalized transformed-response root-MSE.

Generated-weight inference requires a correct conditional mean, regular
root-N-consistent preliminary fit and weights that are smooth/positive in that
neighborhood. The expected derivative of the weighted estimating equation with
respect to preliminary parameters is zero because E[y-mu|x]=0. Consequently
preliminary-weight estimation adds no first-order covariance term under these
assumptions; this is not finite-sample conditional independence or inference
robust to a misspecified mean. Keep this justification explicit and independently
review it before implementation. No bootstrap/first-stage finite-sample correction
is promised.

The author's WNLS comment that classical and robust covariance coincide
asymptotically is **not adopted**: w=1/mu0 does not whiten Var(y|x)=mu^2. Its
classical covariance remains working; the author's own printed SEs differ.
FGNLS uses the correct variance weight under the stated DGP, asymptotically.

### Reporting and retained state

The installed result remains `NlsResult` on **transformed response/design**:
`sample.weights` remains null, because weighting is explicitly represented in the
formula/data, not a new weight field. Outcome and predictor bindings must identify
the actual transformed response and fixed scale; no forged original-y metadata.
Native RSS/RMSE/residual variance/centered R-squared concern that response and
unnormalized objective. Do not relabel them as original-y diagnostics.

Reconstruct original-level means `s*fitted_m`, residuals `y-s*fitted_m`, reciprocal
source-law rates and original-level RSS/centered R-squared separately. Label scaled
and level diagnostics explicitly. Report/export preliminary means and frozen
weights/scales with both covariance-mode reconstructions. Audit every row against
independent R, including transformed native JSON projections and level CSVs.

Existing convergence/work/sample limits and failure/state rules apply. Each fit
attempt clears stale estimation; successful prediction retains its owned fit and
typed publication rules. A subsequent SQL design replacement follows existing
SQL state semantics; retaining the preliminary fitted model across it is NOT
promised. Capture its predictions/results before replacement. Scripts are ordered
commands, not whole-script transactions; export is no-clobber, no crash-durability
claim.

## Independent reference and feasibility (before implementation)

Development-only base-R profile amplitude at each slope d:
`c(d)=sum(w*y*exp(d*x))/sum(w*exp(2*d*x))`; solve the profiled slope score by
Brent/uniroot, then a=-log(c), b=-d. Independently certify Jacobian rank, observed
loss curvature, score and full undamped GN step; form full covariance/inference
and every projection. No failed R nls iterate/author rounded coefficient is the
primary oracle. Use the existing1e-11 absolute+1e-10 relative primary tolerance;
never widen it to conceal a discrepancy.

Initial independent profile feasibility (R4.6.1, original double source values):

| Fit | a | b | Classical SE a,b | HC1 SE a,b |
| --- | --- | --- | --- | --- |
| WNLS |1.9906226832569718 |-.9960671601693482 |.0224902662300443,.00977703464844766 |.0358551069833449,.0224175505534487 |
| FGNLS |1.984034854561487 |-.9906910367889625 |.0147736843728497,.0100099993655808 |.0145802622867085,.0101319394406074 |

Both have rank2, positive observed minimum eigenvalues1025.89/3125.10,
normalized scores8.40e-15/2.38e-15, full steps1.54e-14/2.54e-15. Independently
normalizing weights to sum N changes neither fitted parameters nor covariance
beyond1e-12. Source four-decimal coefficient/SE columns agree; independently
computed level R-squared .3901799592060249/.3905660528871318 matches source printed
.39017996/.39056605. Logs/scripts are development feasibility, not frozen reference
artifacts or native execution evidence yet.

Supplemental source-log exponential-density-at-point is **not an NLS likelihood**
and is outside these two coefficient/SE columns. The precise WNLS root gives
-208.93382391129 versus historical printed -208.93381 (difference1.39e-5); do not
hide this by widening any numerical/source tolerance or claim exact log parity.
Printed seven-digit historical coefficients and float32-derived predictions/rates/
log densities differ from the certified double root. Initial corner probes with
those declared decimal intervals/float32 operations reach -208.9338101305, which
rounds to the historical scalar, but this is NOT reconstruction of the hidden
legacy coefficients/optimizer or rigorous bounds on the quantized pipeline.
Preserve this limitation. No exponential ML, LR or supplemental density table is
added in this slice; the native NLS likelihood fields stay null as before.

## Acceptance and implementation order

Risk: **high**, specifically generated-weight inference and confusion between
scaled/original-level covariance/diagnostics. Bounded independent statistical/plan
review is required before implementation, not a repeated review of the completed
kernel.

1. Freeze independent reference generation first: same preliminary fit/fixed
   weights, both final fits/both complete covariance and inference matrices,
   all10000 sample indices/transformed and level projections, diagnostics and
   compact author Table5.7 gates. Reproduction must preserve frozen references.
2. Add `weighted.td` on existing `nls.csv`: preflight, preliminary fit/prediction,
   frozen SQL design, four final fits/projections, explicitly labeled diagnostics,
   no-clobber `weighted_predictions.csv`. Keep unweighted script/outputs intact.
3. Add `check_weighted.py` plus persistent failure replays: pin source/ordinal/
   bindings/sample/weights and actual native JSON/CSV; reject coefficient,
   covariance, refreshed-final-weight, swapped-scale/response, projection and
   diagnostic forgeries; do not emit a success receipt on failure. Receipt
   staging/link cleanup/no-clobber must retain existing invariants.
4. Add `tests/mma05_weighted_nls_workflow.rs` with the following observable cases:
   - Actual original-source JSON and text exports byte-identical; both complete
     final covariances/inference, fixed weights and scaled/level diagnostics agree
     with R at unchanged tolerance. Runtime source is developer-supplied, not
     redistributed. Offline root tests use small inspectable controls instead.
   - Closed-form constant mean2: x[-1,0,1], y[2.2,1.6,2.2]. Preliminary/final
     a=-log(2),b=0; weights1/2 and1/4; scaled RSS.12/.06 versus level RSS.24;
     covariance diag(.02,.03) classical, diag(.02,.015) HC1, offdiag0; df1
     Cauchy reference for approximate intervals/statistics/p-values. Exercise
     both text/JSON and actual exported columns, not a kernel-only control.
   - Reject duplicate/invalid ordinals, missing/nonpositive/nonfinite source
     values, rank/zero-variance/row-limit failures before new artifact publication.
     Existing output sentinel and input CSV survive failures; no owned staging.
5. Run focused new root tests, affected root package tests/relevant source/SQL/
   export regressions and scoped test Clippy/fmt/diff. No kernel/API change means
   no automatic repeat of PR340's unchanged whole-workspace baseline. Expand
   scope if any shared implementation must change. Review complete source/audit
   diff and original-source replay before ready/merge/verified-tree cleanup.
6. Update README, SPEC, roadmap and ledger with only actually implemented/tested
   scope and remaining work, plus PR evidence and all unverified areas.

Likely files: this contract, `weighted_reference.R`, `weighted.td`,
`check_weighted.py`, `check_weighted_failures.py`, the new root test, README/SPEC/
roadmap/ledger. Existing source preparation and unweighted reference/scripts
should remain unchanged. No Rust production files are planned.

Stop and report if rescaling is not equivalent to the frozen source objective,
reference certificates fail, a primary tolerance fails, or a public API/kernel/
parser/dependency change is needed. Do not improvise broader weight support.
Known-variance covariance, OLS/exponential ML, marginal effects, constraints,
iterative FGNLS, uncertainty for predictions and the full illustration inventory
remain required separate work; no whole-Table5.7/family or platform claim.

Implement exactly this plan. Do not broaden scope. If the plan conflicts with the
codebase, stop and report the conflict instead of improvising. Report files
changed, tests run, deviations and unresolved risks.
