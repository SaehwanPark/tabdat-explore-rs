# WIP: original Table17.8 Weibull duration regression

**Status: pure stats kernel/reference controls pass. Native `streg` execution is NOT installed.**
This is one ongoing vertical slice; do not merge parser/kernel-only work or start
another slice before the complete workflow passes its checks.

## Exact source and supported outcome

March2005 `mma17p4duration.do` §17.11 pp.603–608, Table17.8 Weibull
coefficient column: same ORIGINAL3343 one-record spells/1073 full-time events/
20887 two-week intervals, all40 author regressors plus intercept, as the
[merged exponential analysis](README.md). Use its audited original DTA conversion,
not the42-field ASCII (stateur absent). Retain102 all-zero status rows as censor1=0.
Original assets/fingerprints and source acquisition are already documented there.

Implement the existing parsed form:

```text
streg spell xvars, failure(censor1) dist(weibull) robust
predict eta, xb
predict h, hazard
predict H, cumhaz
predict S, survival
```

IID model-based default and explicit independent-spell robust covariance,
`noconstant` when identified; cluster/exponential variants remain explicitly
unsupported in this bounded native slice. Existing composed exponential workflow
remains available and unchanged. No new survival declaration, left truncation/
delayed entry/other censoring, weights/survey, multiple spells, named fit storage,
effects/prediction uncertainty, mean/residual/count-probability predictions or
whole Table17.9/family/chapter claims. Reuse existing crates and safe Rust.

## Statistical contract

PH parameterization: shape a>0, eta=X beta (intercept first when present),
H(t|X)=exp(eta)*t^a, h(t|X)=a*exp(eta)*t^(a-1), net S=exp(-H).
Time is recorded as exact positive continuous two-week intervals. Competing/non-
event statuses are treated exactly as in the original full-time cause-specific
analysis; net survival is not all-cause joblessness survival/cumulative incidence,
and UI projections are not causal. Do not assert latent-cause independence.

Fit joint-complete-finite time/event/X cases, then validate positive retained time,
exact0/1 event, at least one observed event and regular full-rank joint information.
Owned sample records total/dropped/retained indices and retained failures/censors.
The source script must separately reject incomplete/invalid original designs so
all3343 original rows are used. Bounds: N<=10000, P=K+shape<=64,
N*P²<=20000000, N>P; no arbitrary shape clipping/ridge/pseudoinverse fallback.
CLI requires predictors as already parsed; internal null/intercept-only fits need
no fake regressor. No silently fixed shape or count-only conditional covariance.

Density likelihood per record:

```text
l_i = d_i*(log(a)+X_i beta+(a-1)*log(t_i)) - exp(X_i beta+a*log(t_i))
```

Report density LL and explicitly separate Stata-normalized LL=density+sum(d log t).
Full/null same-sample slope LR refits intercept+shape (shape-only when noconstant);
model-based reference chi-square(number of slopes), robust p/reference withheld.
Full joint observed covariance in parameter order beta then lnshape. Model-based
correction1; robust full joint score sandwich N/(N-1), no residual-DF multiplier.
Normal beta CIs, lnshape SE/normal inference, positive exp(logshape±z*SE) shape CI;
shape=1 is regular interior testing, not an invented shape=0 test. Robust slope
omnibus is rank-aware Wald, with honest unavailability if not identified.

Prediction owns fitted beta/shape and uses CURRENT X. xb=PH eta and needs no time
or event; hazard/cumhaz/survival use current recorded time-variable plus X, not
current event flags. Missing/nonfinite required values yield NULL; finite nonpositive
prediction time or unrepresentable finite arithmetic fails before atomic DOUBLE
publication. Preserve model/data/labels/targets on prediction failure; failed new
fits clear stale inference only, not data/labels. Native new prediction kinds must
not change legacy count/binary/choice/Tobit/truncreg/Heckman semantics.

## Independent reference / analytic implementation seam

R4.6.1/survival3.8.6 `survreg(...,dist="weibull",robust=TRUE)` fits AFT
parameters b,ell=ln(sigma). PH mapping is beta=-b/sigma, q=ln(a)=-ell.
The FULL Jacobian has beta/b=-a I, beta/ell=-beta and q/ell=-1. Transform the
entire42x42 covariance, then multiply robust covariance by3343/3342. Merely scaling
the beta block omits shape uncertainty/cross terms and is WRONG.

Independent natural (beta,a) score: s_beta=X(d-H), s_a=d/a+(d-H)log t.
Observed information is sum H*[X,log t]*[X,log t]' plus sum(d)/a² in the final
shape diagonal. It is jointly concave for a>0; use the existing guarded positive-
coordinate Newton utility if suitable, with explicit RMS-unit-column scaling and
canonical score/step diagnostics. Preserve its existing Gaussian consumers.
Transform natural shape to q after fitting for covariance/reporting. Full robust
sandwich must agree with the separate R AFT Jacobian route.

Observed original-source audit probe (no native fit claim):

- a=1.1292248659100332, q=.12153143797482154, robust SE(q)=.019437422302810994.
- PH intercept=-4.357885544054; RR=.448115601822, DR=-.426918743841,
  UI=-1.496065954585, RRUI=1.015226418175, DRUI=-.298841657038,
  LOGWAGE=.365525270152.
- Density LL=-3933.4898796520902; normalized LL=-2687.5995196287299
  (author−2687.5995). Max raw score≈4.46e-11; independent full sandwich vs
  transformed R covariance max absolute discrepancy≈1.2e-13.

## Acceptance and failure cases before implementation

1. Source native script all3343/1073 rows, all41 beta values/full42x42 joint
   covariance/shape inference, LL conventions/null and robust Wald agree with R
   and author printed Weibull rows. Exact original source hex/sample/design audit.
2. Independent synthetic controls cover model-based/robust, no-intercept, joint
   missingness, units/permutation, shape near1 and finite nonconstant hazards;
   analytic score/Hessian checked separately, including beta/shape cross terms.
3. Actual script/root CLI JSON and text expose full fit/inference and current
   xb/hazard/H/S projections with source/UI-interaction restoration; native
   plots/reporting only as needed for a usable source analysis, not layout parity.
4. Invalid supports/nonbinary events, zero events, rank/nonregular numerical fits,
   bounds and current-prediction schema/support/target/overflow errors protect
   data/labels/files/model state as declared. Test prior-fit replacement/failure
   and legacy estimator/prediction consumers; reject unsupported options explicitly.

Current kernel milestone: seven `weibull` library tests pass. The400-row full
41-beta control checks2032 model-based and2031 robust R gates, all1764 covariance
cells per mode and4800 unique current/UI0/UI1 point predictions (both modes).
Independent score/Hessian finite differences, Weibull time-unit joint covariance
Jacobian (intercept/shape covariance is NOT invariant), invalid supports/rank/
sample/weights/cluster/options and exponential/probability limits pass. Stats
all-target Clippy `-D warnings` passes. No-intercept full joint R controls add1943
model-based/1942 robust gates, including an independently bracketed scalar shape-
only null (not a fake intercept or fixed shape), and reversed-row covariance/
owned-sample identity passes. Original intercept-reference artifacts remain byte-
identical after extending the reference generator. The original3343-spell R
reference passes its separate full observed/sandwich reconstruction, but original
native execution is NOT yet checked. Shared runtime/reporting/prediction and
whole affected-crate checks remain pending. With noconstant the PH scale is fixed;
unlike a free intercept, this specification is not generally time-unit invariant.

Reference reproduction (development only):

```sh
Rscript examples/mma17p4duration/weibull_reference.R /path/duration.csv /tmp/new-weibull-original /path/ema1996.dta
Rscript examples/mma17p4duration/weibull_reference.R tests/fixtures/duration_exponential/input.csv /tmp/new-weibull-control
Rscript examples/mma17p4duration/weibull_reference.R tests/fixtures/duration_exponential/input.csv /tmp/new-weibull-no-intercept - noconstant
cargo nextest run --locked -p tabdat-stats --lib weibull --no-tests fail
```

Stats optimization uses the Stata-normalized objective to avoid time-unit constants
in Newton line-search precision/LR, then explicitly restores density LL. Reported
beta/logshape curvature includes the actual finite-score chain-rule term, not
merely a zero-score Jacobian approximation. Positive shape CIs/inference are NULL
when unrepresentable, never spuriously exact or zero-tailed. Prediction survival
can round to0 for huge integrated hazard even when H cannot itself be represented;
hazard/H overflow fails, while a genuine probability endpoint is preserved.

Finish gates: focused numerical/runtime/new script tests first, then stats and
reverse-dependent packages/root consumers proportional to new shared result/state/
predict boundaries; relevant doctests/Clippy/format/diff. Early draft PR,
incremental commits/pushes, PR evidence, merge only after complete workflow and
safe branch cleanup. Model/source/R evidence and capability docs/ledger must
clearly distinguish source audit, parser acceptance, execution and tested support.

## Ordered implementation / handoff

Risk: **high** (new estimator, joint shape inference and shared model/predict DTOs).
First three actions: commit/open this draft; implement independent source/control
reference; implement/validate the pure stats kernel before wiring it. Then add
owned execution state/data adapter, result/text/JSON reporting and three explicit
prediction kinds; only then add/run the actual source workflow.

Bounded targets: stats `weibull.rs`/lib exports and existing guarded Newton utility;
execution `streg.rs`, estimation/predict dispatch and error/result enums; domain
command/prediction kinds, language parser, reporting JSON/text and current schema/
contract consumers. Reuse Tobit/truncreg adapter patterns. Public additions are a
new executable Weibull result and named prediction modes; existing result schemas/
modes stay unchanged. Use CONTRIBUTING impact-scoped checks, starting with
`cargo nextest run --locked -p tabdat-stats weibull --no-tests fail`, then the new
execution/root targets, followed by affected/reverse crates.

Stop for inaccessible original data, reference discrepancies, demonstrated need
for new dependencies/unsupported censoring or conflicting unrelated branch work.
Implement exactly this plan. Do not broaden scope. If the plan conflicts with the
codebase, stop and report the conflict instead of improvising. Report changed
files, commands/scopes tested, actual deviations/unresolved risks; never describe
unrun checks as passing. Keep WIP status until end-to-end evidence exists.
