# Exponential-mean NLS: contract and WIP evidence

**Status: bounded native unweighted NLS workflow implemented and validated for the
selected Table5.7 column. Risk: high; no whole-table/method-family claim.**
Deliver one complete native `nl` → predict/diagnose → report workflow for the
unweighted NLS column of Cameron–Trivedi(2005) Table5.7, §5.9.2–3 pp.159–163.
This is the author's **generated**10,000-row example, not an empirical sample or
whole-table/chapter coverage. A separate [fixed-first-fit WNLS/FGNLS workflow](weighted.md)
uses transformed responses and fixed preliminary weights. Exponential MLE,
known-variance sandwich and§5.9.4 marginal effects remain required separate work.

## Exact source, model and evidence selected before implementation

Author index: https://cameron.econ.ucdavis.edu/mmabook/mmaprograms.html.
Original assets stay external, explicitly acquired:

| Asset | SHA256 |
| --- | --- |
| mma05p2nls.do | 61a47826604f21594c2c2c96e44ef7233caa7b82df2b63566f49eecf0586554d |
| mma05p2nls.txt | ca79b44886ee861f16a5e617f9e5f88c79fa5ad0fdb3276261010e9708a1d557 |
| mma05data.asc | 460f5d731f1848c8695d78a1fbf0db348d0daced4b8b5b0c2734bffa20b3e31c |

Original two-column ASCII y,x,10,000 rows; preserve source ordinal/tokens, no RNG
replay or float32 conversion in production. Model mean **µ=exp(-a-b*x)**,
parameters ordered a,b, explicit start1,0. True source rate is λ=exp(a+b*x),
mean1/λ, variance1/λ²=µ². The author's comment equating exp(-a-b*x) with µ² is
incorrect; executable covariance code uses µ². NLS is NOT exponential MLE.

Strict R `nls` (even analytic derivatives,500 iterations,tol1e-12) stalls near the
minimum: reported convergence FALSE, a differs from a certified optimum by3e-8.
Do not freeze that failed iterate as an exact oracle or widen result tolerances.
Independent primary source oracle profiles amplitude c at fixed slope d in
µ=c*exp(d*x): c=sum(y*exp(d*x))/sum(exp(2*d*x)); base-R Brent `uniroot` solves the
profile score on[0,2], then a=-log(c),b=-d. R4.6.1 root residual1.78e-15 gives
(a,b)=(1.8875622977112052,-0.9574683112983651),
RSS10149.86322727484, RMSE1.007566057685353,
centredR².391344620619342. Independent guarded Gauss–Newton/QR reaches this in
12 iterations (normalized score2.05e-13, relative full step6.38e-13).
Initial observed half-RSS Hessian is indefinite (eigenvalues921.4,-6645.2), so the
existing concave Newton solver cannot be used unchanged. Final eigenvalues are
positive113012.1,1065.3. These are feasibility evidence, NOT native execution.

The original compact NLS table gives a1.8876,b-.9575, classical SE.0307/.0097,
robust SE.1421/.0612. Native optimum will be compared at those printed units;
verbose coefficient rounding beyond them differs slightly from the old optimizer.
Input float32 alone shifts a/b only about1e-9; do not blame all optimizer differences
on storage. Source LL at NLS is an exponential-density evaluation, not a Gaussian
NLS likelihood/LR test. Author derivative-OLS coefficients are zero updates, NOT
the fitted a,b; only its covariance/SEs represent the NLS comparison.

## Supported interface and statistical meaning

Execute existing AST, without grammar/starting-value defaults changing:
`nl y = exp(-a-b*x), params(a b) start(1 0) [robust]`.
General bounded arithmetic expressions may use finite numeric literals, data and
parameter identifiers, unary minus,+,-,*,/, and one-argument exp only. Unsupported
functions/comparisons/strings/NULL expressions error, not reinterpretation. Every
declared parameter must occur, params/start are finite/unique/matching and outcome
cannot be a parameter. Parameter identifiers take their explicit params role;
remaining identifiers are numeric data variables in first-use order. No automatic
intercept: formula controls all parameters; legacy noconstant/default flag does
not add/remove an explicit parameter (same substantive Python behavior).

Joint-finite outcome/referenced-data sample in original order; unrelated missing
columns do not exclude rows. Formula domain/overflow/derivative failures on a
complete row error rather than silently dropping it. Nonrepresentable exp (including
underflow to zero) fails: a clipped zero can erase nonzero tail derivatives and
falsely pass score/curvature checks; no log-tail/extended-precision evaluator is
claimed. Candidate rows<=10000,
1<=P<=32,N>P, expression<=128 nodes/depth32,
N*P²*nodes<=20million; up to200 iterations/60 step halvings, cumulative evaluation
budget500million units (value evaluation N*nodes; derivatives N*nodes*(1+P+P²)).
Check budget before each evaluation; line search needs values only. Initial invalid domain,
rank failure, zero/nonfinite residual variance, nonregular final minimum or failure
to meet BOTH normalized score<=1e-13 and relative **full undamped** GN step<=1e-13
errors. Do not declare convergence from objective change or a shrunken line step.

Use scaled Householder QR Gauss–Newton, objective-decreasing backtracking with a
32eps*(1+scaledRSS) arithmetic allowance (not comparison-tolerance widening).
Global outcome scaling protects the SAME full-sample loss; restore raw units and
reject nonrepresentable summaries. Final Jacobian full rank and observed half-loss
Hessian positive definite are mandatory; no ridge/pseudoinverse/returned damping.
Only a regular local minimum is established, not global uniqueness.

Classical covariance: RSS/(N-P)*(J'J)^-1, a **homoskedastic working** covariance,
not valid for the source's heteroskedastic exponential DGP. Robust: HC1
N/(N-P)*(J'J)^-1*sum(r_i²*j_i*j_i')*(J'J)^-1. This is explicitly **Gauss–Newton
bread, conditional mean correctly specified**, not misspecified-mean observed-loss
sandwich. Full covariance/parameter order, SE, approximate Student-t(N-P) statistic/
p-value/95% CI, RSS/RMSE/centredR² (NULL if TSS0), sample and convergence metadata.
No invented Gaussian/exponential likelihood, F/LR or general nonlinear restrictions.
A migration note must explain stricter rank/convergence/variance checks and t
rather than legacy Python normal-reference inference; no pseudoinverse parity.

Retain serialized Nonlinear(ModelState)/family tag nonlinear unchanged; derived
serde is a consumer even without an installed runtime. Add a distinct owned
NonlinearFit variant/tag nonlinear_fit with the same Nonlinear family, following
PoissonFit/TobitFit precedent. Test old-placeholder and owned-fit encoding/decoding
roundtrips with distinct tags; do not silently replace the internal-v1 payload.
A new fit attempt clears stale inference
on failure like other native estimators; data/schema/order/labels remain unchanged.
Default/legacy xb prediction evaluates the nonlinear fitted mean (NOT a latent
linear index); explicit mean also accepted, residuals=y-µ. Re-evaluate current
referenced X with retained parameters; missing X -> NULL; residuals also need y.
Complete-row invalid arithmetic and target collisions fail atomically using the
existing typed DOUBLE publication; current candidate rows<=10000/work bounds.
No intervals/std/other modes or new test/lincom/margins/named-estimates support.

## First changes, acceptance and stop rules

1. Commit independent R profiled source oracle/full GN covariance and small frozen
   controls BEFORE native kernel. Check score, final curvature and all projections;
   keep failed strict-nls feasibility evidence explicit, not a success certificate.
2. Add pure stats nls module/typed mathematical expression (no Domain dependency),
   first/second derivatives and QR/backtracking/strict checks. Reuse existing matrix/
   distribution utilities, no crate/dependency/generic optimizer framework.
3. Adapt existing Domain expression to that bounded model; install owned state,
   current-row prediction, additive NlsResult/text/CLI-v1/help/catalog and direct-AST
   validation. Add development-only source converter/reference/audits and actual
   native script; update capability/status only when executable/validated.

Likely seams: `crates/tabdat-stats/src/nls.rs` (+expression/tests submodules),
`crates/tabdat-execution/src/nls.rs`/dispatch/prediction, Domain commands/state/
results/errors, reporting JSON/text/help/catalog; runtime/reporting `nls_contract`
and root `tests/mma05_exponential_nls_workflow.rs`. Development reference/converter/
audits live here; frozen controls in `tests/fixtures/nls`. Preserve the existing
parser AST/grammar, all other estimator results and meaningful errors. A bounded
independent plan review is required before native implementation (new optimizer,
derivatives, covariance and owned/public state make this high risk).

Focused commands once those targets exist:
`cargo nextest run --locked -p tabdat-stats --lib --no-tests fail -E 'test(~nls::tests)'`,
then runtime/reporting `--test nls_contract`, then root
`--test mma05_exponential_nls_workflow`. Select every new test explicitly and finish
with affected-crate/reverse-consumer checks; never count an empty selection.

Five observable acceptance groups:
- Original10,000 rows, both covariances/all matrix entries/SE/t/p/CI/diagnostics and
  every fitted mean/residual against certified R profile/Jacobian references;
  compact author gates separate from stricter native/R numeric comparisons.
- Noisy linear formula reduces to independent R lm; additional nonlinear/parameter-
  only/negative-response controls validate derivatives, parameter order and sample.
- Joint missingness, wide original storage/labels/named tables, current-only mean
  versus residual sample, model replacement and atomic failure/prediction behavior.
- Bad start/domain/unsupported AST/unused or duplicate parameters, rank deficiency,
  zero variance, infimum-at-infinity and iteration/work bounds explicitly fail.
  Freeze stationary-nonminimum control BEFORE kernel: formula a*a+a*x, x=-1/1,
  y=1/1, start a=0. J=(-1,1), RSS2, score/full GN step0 and rank1 pass, but observed
  half-RSS Hessian2-4=-2: MUST reject, not substitute positive GN curvature.
- Actual source script JSON/text outputs/projections byte-identical and source-
  preserving; independent checks reject shared-wrong native/R parameters, covariance,
  sample/design/mean/residual values; existing output no-clobber/failure paths.

Primary numerical tolerance1e-11 absolute+1e-10 relative; source/sample/order/counts
exact, printed quantities half-last-unit. Investigate differences, never widen to
hide them. Focused new tests first, then affected crates/reverse consumers and
proportional Clippy/fmt/diff; independent kernel/integration review before merge.

Implement exactly this plan. Do not broaden scope. If it conflicts with codebase
reality, stop/report rather than improvise. Report files/tests/deviations and
unresolved risk. No whole-nl, weighted-column, chapter, book or platform claim.

### Independent preimplementation review disposition

Read-only review08245ad4-426b-4485-bad8-967dbfdc5086 found no source-case scientific
blocker and two P2 plan omissions. Both accepted before implementation:
(1) preserve legacy nonlinear serialized state and add a distinct owned tag/focused
wire roundtrip, rather than replacing its payload; (2) freeze the exact stationary
local-maximum rejection above. Reviewer inspected committed diff/source and supplied
R evidence; did not execute tests/R. Parent verified existing domain-json-v1 contract,
state serde and analogous Poisson wire test before choosing additive compatibility.
Native implementation must retain these dispositions. This plan review is not
numerical validation of the subsequent implementation.

### Pure-kernel checkpoint (runtime/CLI still deferred)

Independent references were committed before the native kernel: six fitted cases,
both covariance modes, four rejected controls and522 analytic value/derivative rows;
all eight generated reference outputs reproduced byte-for-byte. The original
external-data reference is certified separately, not redistributed as a fixture.

Native pure-stats API source probe checks40044 native/R scalar/projection gates,
20000 exact R/source fields and20000 native retained-index assertions, plus9
compact author printed-unit facts. Max differences: coefficients1.69e-12,
covariance6.04e-14, SE2.13e-13, mean/residual2.08e-11; no tolerance change. Both
covariances share bit-identical fitted parameters/projections. Native11 accepted
steps (12 linearizations) give score2.05e-13/full step6.38e-13, RSS10149.863227274845,
RMSE1.007566057685353, centredR².3913446206193423, work6.72million units.
This is NOT an installed CLI/script workflow or whole-slice completion.

Seven new focused stats tests pass, covering all frozen derivatives, complete
covariance/inference/projections, rejected stationary maximum, zero variance/rank/
unattained infimum, sample/order, tiny/huge response units and resource/arithmetic
bounds. Affected stats crate:242 distinct passes/one pre-existing ignored across46
current Cargo-inventoried artifacts (the seven are INCLUDED, not added twice).
Broad nextest timed out before tests; current truncated_gaussian --list sampled in
_dyld_start. SHA256-identical fresh copies ran the standard harness with package
cwd, one test thread and two concurrent artifacts. This is a documented fallback,
not nextest success. Scoped stats Clippy/lib/tests with-Dwarnings and fmt/diff pass.
Other crates/runtime, final review, source script/atomic IO and platforms remain
unverified at this checkpoint.

### Kernel-review P1: fail closed on curvature rescaling loss

Independent code review dfa4593b-ec31-49a4-8d80-caf0d49929a8 identified a P1:
mean1e-250*a*a+a*x, starta0, x=(1e-100,0),y=(0,1e100) has full-rank J,
zero score/full GN step and finite positive RSS1e200, but raw observed half-loss
curvature1e-200-2e-150<0. Dividing representable mean Hessian2e-250 by response
scale1e100 prematurely rounded it to zero; later column divisions should restore
a negative scaled-average curvature about-1e50, not the incorrectly checked+.5.
HC1 covariance is genuinely0 here, so covariance rejection is not the solution.

Parent reproduced the false successful fit with a failing focused regression;
independent base-R arithmetic confirms raw-2e-150/correct-scaled-1e50 versus
unsafe+.5. The curvature guard now rejects any nonzero correction erased or made
nonfinite at an intermediate division/product. Exact zero residual/Hessian terms
remain0. This is a bounded representability error, not extended-precision recovery
or a claim to accept every regular extreme-scale problem. The same regression
confirms a truly regular a*x fit with x=(1,0),y=(1,2),a1 and zero HC1 covariance
still succeeds with unavailable t/p/CI. No threshold or reference tolerance changed.

The initial curvature guard passed eight focused tests/Clippy; its rebuilt
original API probe passed40044 gates with byte-identical40052-row CSV. Follow-on
fail-closed checks also protect nonzero arithmetic/derivative, residual scaling
and covariance intermediates from erasure. A ninth test covers an unrepresentable
mean Hessian and an extreme-scale positive HC1 variance incorrectly rounded to0;
these fail rather than publish false regularity/inference. Exact-zero terms still
remain allowed. All nine focused tests pass. Earlier235 non-NLS stats passes and
one ignored are unaffected; nine replace the previous seven, giving244 distinct
passing stats cases, not a new full nextest claim. The latest expanded checks pass
scoped Clippy/lib/tests-Dwarnings and rebuilt original-data API replay:40044 gates,
unchanged tolerance, max coefficient1.69e-12/covariance6.05e-14/mean2.08e-11 versus
R. Computing the raw residual before scaling changes ordinary exported numeric
values by at most8.53e-14 versus the earlier probe, not a statistical or tolerance
change. Focused independent follow-up3bb75ba1-ba61-4042-85eb-93e32f2ba80c inspected
the repair and artifacts (no independent test execution), withdrew the original
P1 BLOCK and found no new issues in this bounded repair. This is not whole-PR or
integration approval.

### Native runtime integration checkpoint

The native adapter now compiles the bounded Domain AST with explicit parameter
roles/order, fits the joint-finite original-order sample, installs additive
`NonlinearFit` and returns `Nl` without touching legacy nonlinear wire payloads.
Current default/xb/mean/residual prediction publishes typed DOUBLE/NULL atomically
through the existing backend, preserves the owned fit and original typed data,
and updates an active named table's schema normally. Failure clears stale
estimation only for fit attempts, not predictions. Six reference formulas/both
covariances cross the real parser/runtime and every coefficient/inference/full
covariance/projection comparison at the unchanged tolerance.

Nine Domain state-wire cases (including old nonlinear/new owned tag), nine
focused runtime cases, three reporting cases and one root CLI discovery case pass. The runtime cases include
explicit order/noconstant, missingness/original indices, direct-AST failures,
current-X/missing-y mean versus residual eligibility, parameter-only means,
collisions/arithmetic failure atomicity, all four frozen nonregular rejections,
AST depth/arity rejection and wide-ID/DECIMAL/label/named-table preservation. Successful named-table column append is an expected schema update,
not a requirement to freeze the active table's old metadata. Scoped four-crate
Clippy lib/tests-Dwarnings passes. Earlier failed development test runs corrected
unsupported test syntax (`==`, not `=`; direct AST for restricted prediction
flags) and the expected active-table schema, not numerical tolerances or native
algorithms. These are focused boundary checks, not final affected/reverse-consumer
validation. Original10000-row CLI/script/source export/atomic IO audits, independent
integration/source review and complete-slice closing checks remain WIP; no merge,
whole-table/method/chapter, platform or release claim.

### Original-source CLI replay checkpoint

The development-only `prepare.py` verifies the pinned original ASCII hash/all10000
rows, retains source ordinals and exact y/x tokens, and publishes a new CSV by
same-directory staging/hard-link without clobber. Validation, injected write
failure and existing-target rejection publish nothing/leave no staging file;
this is not crash-durability or platform certification.

`nls.td` loads/preflights prepared data, fits both covariance modes, predicts means
and residuals, derives the source-law reciprocal rate and squared residual,
reports descriptive residual summaries and exports source-preserving CSV. Both
actual current CLI JSON/text executions exit0/stderr empty. The1419817-byte CSVs
are byte-identical, SHA256
`38d1108e9dbe0f0278d17f743f35d4bbce1e3df068b3eeb9c0dae7612f45056a`.

`check_reference.py` verifies source hashes, model bindings, exact samples/source
identity/order, every fitted JSON/CSV mean/residual, both full covariances and all
coefficient inference, diagnostics, rates and squared residuals against independent
R. It passes120055 numerical gates at unchanged1e-11 absolute+1e-10 relative,
30000 exact exported source fields,20000 reference/source fields and9 compact
author facts. Two ADDITIONAL relative1e-9 gates prevent bogus zero substitution
for nonzero~1e-40/1e-51 Student-t tails; max relative difference1.12e-10 reflects
the accepted converged-t difference. This is stricter than the primary absolute
rule, not tolerance widening. Max beta1.69e-12/SE2.13e-13/covariance6.05e-14,
mean/residual2.08e-11, reciprocal-rate3.52e-10/squared-residual4.47e-10; all within
the fixed scale-relative contract. Diagnostic RSS error5.28e-11. Ten actual-source
coefficient/covariance/tiny-tail/projection/binding/sample/diagnostic/source-identity/
ordinal/CSV-projection forgeries fail without receipt.

Native a1.8875622977095359,b-.9574683112978039, RSS10149.863227274845;
classical SE.030681916435664428/.00974185303447341, HC1
.14207932958772906/.06119685266594117. Both use11 accepted steps, score2.046e-13,
full step6.376e-13 and work6720000. Ordinary mean residual-.0161230529764893 need
NOT equal0: the a score weights residuals by the exponential mean, not a constant
Jacobian column. Residual summaries do not establish adequacy, the full exponential
law or causality. Classical covariance is a working homoskedastic convention for
this heteroskedastic generated DGP; HC1 assumes the correct conditional mean.

Reproduce with explicitly acquired external assets; use NEW output paths/directories,
check every exit code (e.g. shell `set -e`), and never reuse an old CSV after failure:

```sh
mkdir -p /new/json-run
python3 examples/mma05p2nls/prepare.py /path/to/mma05data.asc /new/json-run/nls.csv
(cd /new/json-run && /path/to/tabdat-explore-rs --json -f /path/to/repo/examples/mma05p2nls/nls.td > results.jsonl)
Rscript examples/mma05p2nls/reference.R /new/r-reference /path/to/mma05data.asc
python3 examples/mma05p2nls/check_reference.py /new/json-run/results.jsonl /new/json-run/nls_predictions.csv /new/r-reference /path/to/source-assets /new/receipt.json
python3 examples/mma05p2nls/check_failures.py /new/json-run/results.jsonl /new/json-run/nls_predictions.csv /new/r-reference /path/to/source-assets
```

The source-assets directory contains all three pinned author assets above. Run
text in a separate new directory with its own prepared nls.csv. R/Python are
explicit development/reference tools only; the native prepared-CSV fit/report
workflow needs neither. Scripts are ordered, not whole-script transactions;
earlier in-memory steps can succeed before a later export fails.

Four offline root end-to-end cases pass: a closed-form a=b=0 control with
mu1/residuals(.1,-.2,.1), full classical/HC1 covariance and closed-form Cauchy
t(1)95 reference quantile (NLS intervals still approximate), actual JSON/text/CSV
identity, bad/nonregular/10001-row preflight failures,
export no-clobber/no-staging and unpinned preparation failure. Source-file controls
are synthetic, not redistributed author data. The persistent `check_failures.py`
passes the ten real-original forgeries plus injected prepare/audit write failure
and existing-target/receipt no-clobber/cleanup, without changing external inputs.
Root scoped Clippy passes. Final affected/reverse-consumer and independent-review
results are recorded below.

### Closing verification and reviewed limits

Independent integration/source reviewer890364ad-ab89-424f-a6f5-fdeb60ae6eae found
no issues in immutable761fe41..ff08 integration/source scope. It inspected the
supplied committed diff, code and artifacts; it did not execute commands/tests/R,
independently reconstruct Git, hash files or repeat full CSV comparison. This
complements the independent kernel/P1 follow-up reviews, not a claim of separately
executed full-workspace testing by reviewers.

Current local unit/integration scope passes **1645 distinct tests/two pre-existing
ignored**, including affected crates and reverse consumers. Cargo successfully
inventoried282 current artifacts. Initial broad nextest timed out during build
without results. After a successful inventory/build, nextest discovery stalled
before tests; sampling current cli_json_append_contract --list showed solely
`_dyld_start`. The documented fresh-copy standard Rust harness fallback verified
SHA256 identity/package manifest cwd, used one test thread/two concurrent artifacts
and recorded282 artifacts (four zero-test artifacts, not passing regression
selections). It initially yielded1642 passes/three genuine failures: two old exact
JSON-admission diagnostics omitted newly supported nl, and the old nl test still
expected deferred execution. Corrected only those contract assertions, retaining
their rejection/state checks and adding explicit unsupported-math rejection;
all three reran successfully on current artifacts. This is composite1645-case
evidence, **not one clean full nextest run** or a claim that nextest is faulty.
Ignored cases are existing random_effects_ml_original_source and reshape_contract.

Workspace all-target Clippy-Dwarnings, fmt and diff checks pass. A prior all-target
Clippy attempt timed out with no result; the final incremental invocation finished
successfully. No numerical code/tolerances changed after the reviewed source replay;
its evidence remains current. MIG-0024 records intentional admission/deferred-error
and inference differences; old result/wire formats and pinned parser corpus remain.
No doctest run was needed for the new text/shell examples; none is claimed. GitHub
Actions are disabled; other platforms, packaging/performance and release readiness
remain unverified. The full roadmap/illustration inventory, exponential ML,
known-variance covariance and marginal effects remain required.
