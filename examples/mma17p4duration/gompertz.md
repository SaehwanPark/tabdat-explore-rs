# WIP: original Table17.8 Gompertz full-time re-employment column

**Status: independent R original/control references and native kernel/API
controls pass on this WIP branch. Native command parsing/execution/owned state/
reporting is NOT installed. Keep this slice WIP until its complete native
workflow and verification pass; the original native audit remains pending.**

## Exact illustration and independent reference

Cameron–Trivedi (2005), §17.11 pp.603–608, Table17.8 Gompertz coefficient column;
March2005 `mma17p4duration.do`, original `ema1996.dta`. Same3343 spells/1073
full-time events/20887 recorded two-week intervals and40 author covariates plus
intercept as the validated [Weibull](weibull.md)/[exponential](README.md) columns.
Keep all102 all-zero status rows as non-events and stateur from original DTA;
ASCII omits stateur and is not a full-design substitute. Recorded durations are
exact continuous times here, not an interval-censored likelihood.

Author command is `streg $xlist, nohr robust dist(gompertz)`. Model is
h(t|X)=exp(X beta+gamma*t), H(t|X)=exp(X beta)*t*exprel(gamma*t),
S(t|X)=exp(-H), exprel(z)=expm1(z)/z with exprel(0)=1. Gamma is unrestricted,
including zero and negative values: not log gamma or a positive scale.
[Established distribution documentation](https://chjackson.github.io/flexsurv/reference/Gompertz.html)
uses the same shape/rate parameterization and Stata-compatible unrestricted sign.
For gamma<0, H(infinity)=exp(X beta)/(-gamma) and S(infinity)>0. Do not silently
force a proper zero-survival tail or interpret the mass as identified causal cure.

R4.6.1/foreign0.8.91 feasibility uses a different estimation path: fixed-gamma
Poisson GLM solves conditional beta with offset log(integral exp(gamma*s) ds),
then a bracketed scalar profile-score root estimates gamma. Numerical integration
supplies independent zeroth/first/second time moments; full joint scores and
observed information, N/(N-1) sandwich and `stats::optimHess` finite-difference
certificates are reconstructed without native initial values or a Gompertz package.
Original gamma=.0026580345401342183, robust SE=.0067759139606453, density
LL=-3946.4953351777945, Stata-normalized LL=-2700.6049751544342, slope
Wald529.75029546002418. ALL252 printed original beta/gamma coefficient/SE/z/p/CI
quantities match exact printed rounding. Max raw score2.3e-10; independent numeric
information relative discrepancy3.0e-8. Existing400-row full-design control is
also regular: gamma=.08247225857748143. These are R-only, not native evidence.

Reproduce with `Rscript gompertz_reference.R duration.csv NEW_DIR
[original_DTA_or_-] [noconstant]` (no-intercept uses both optional arguments).
The [committed generator](gompertz_reference.R) now passes original/default400/
no-intercept400 runs. Original detailed/scalar/compact author gates total268,
all matching printed rounding. [Frozen synthetic references](../../tests/fixtures/gompertz/README.md)
contain2030/2029 model/robust gates (all1764 joint covariance entries each),
1941/1940 no-intercept gates, observed/UI0/UI1 points and independently integrated
zero/both-sign/tail moment fixtures. Original source/reference outputs remain
outside the repository. Native kernel/API passes all11 focused controls and225
unique affected-stats tests (one pre-existing ignored original REML probe),
including every reference covariance entry/current point. All-target stats
Clippy `-D warnings` and formatting pass. Broad stats execution used current Cargo
inventory/46 SHA256-identical fresh-copy standard-harness artifacts under the
previously diagnosed host dyld-start workaround, not a broad nextest pass. Focused
nextest itself passed; focused replays are not additional distinct tests. An
independent read-only statistical kernel/oracle source review found no issues;
it did not rerun tests or audit the original native/runtime/CLI, which remain
pending. Eight Clippy literal spellings were shortened to identical f64 bits;
no numerical reference or tolerance changed, and that one test was replayed.

## Proposed native contract

`streg time xvars, failure(event) dist(gompertz) [robust noconstant]`.
Preserve the current streg requirement for at least one explicit predictor;
intercept-only is a kernel/API control, not a parser expansion. Unweighted,
entry-zero single-spell independent observations; no cluster/weights/survey,
delayed/left/interval entry/censoring/truncation or frailty. Joint finite
eligibility precedes positive-time/original-value exact-binary event checks;
require a failure. Reuse the original-cell check fixed for Weibull/Cox, never
round invalid DECIMAL near-one codes into an event. Full-rank numeric design,
N<=10000, P=location coefficients+gamma<=64, N>P, N*P*P<=20000000. Preserve
other streg distributions/errors; no native exponential implementation is implied.

Reuse shared strict concave Newton on RMS X and duration-scaled natural gamma,
not an artificially positive final coordinate. Joint concavity follows because
H is an integral of exp(affine(beta,gamma)); event log hazard is affine. Require
per-N mean-score AND relative-step bounds1e-10, regular undamped full information,
finite represented estimates/covariance and bounded iterations. No ridge,
pseudoinverse, clipping, forced gamma sign or nonconverged fit publication.
Compute log exprel and time-moment ratios continuously at gamma0, stably for both
signs; avoid exp(z) overflow and cancellation near zero. Native moment formulas
must be checked against independent numerical quadrature, not a copied formula.

Density LL=sum[d*(X beta+gamma*t)-H]; expose the separate Stata normalization
adding fixed sum(d*log t). Null refits intercept+gamma on the same sample (or
only gamma with index fixed0 for noconstant). LR tests slopes, not a cross-family
Weibull comparison; robust LR p/reference is withheld. Selected-covariance,
rank-aware slope Wald is separate. Model-based FULL beta/gamma inverse observed
information or independent-spell FULL row-score sandwich times N/(N-1), normal95.
Include all beta/gamma cross terms; no conditional-beta covariance substitution.
Gamma SE/z/p/natural normal95 interval test gamma0 (exponential within this family),
not a positive-scale boundary or overall adequacy. Zero/unrepresentable or
rank-limited inference is explicitly unavailable. Negative gamma and CIs crossing
zero are valid. No finite-sample/cluster guarantee.

Owned result/sample/model supports current `predict ..., xb/hazard/cumhaz/survival`.
xb is raw X beta, not X beta+gamma*time, and needs only current X. Timed modes
require current X/time, never event. Time0 is meaningful: h0=exp(X beta), H0/S1;
negative finite time errors. Relevant missing/nonfinite values give NULL.
No auxiliaries/uncertainty; at most10000 current rows, typed DOUBLE/NULL, atomic
publication. Schema/support/existing-target/numerical errors preserve data,
labels/order and owned fit. Genuine survival0 is allowed if H overflows but its
log remains represented; unrepresentable required intermediate/log values error
rather than inventing a result. Gamma<0 finite-time survival approaches a positive
limit; do not force it to0. Predictions are model-based extrapolations, not
validated infinite-tail claims, competing-risk incidence or causal UI effects.
Executed fit attempts clear stale inference only; parse failures do not execute.

## Acceptance / important failures

1. Full original sample/design, all41 beta+gamma/full42x42 covariance/normal
   inference/null/slope Wald and all author printed coefficient/shape/compact
   column facts against independently integrated/profiled R. Retain the exact
   original source/ordinal cells and the all-zero statuses/stateur.
2. Complete source script/JSON/text reporting and observed/UI0/UI1 xb/h/H/S at
   SAME current times, coupled UI/interactions/restoration and export identity;
   model-based/robust controls and noconstant reference exercise non-author options.
3. Quadrature moment controls across both gamma signs/zero, derivative/Hessian
   checks, gamma0 exponential likelihood/hazard/information equivalence,
   regular negative-gamma fit and positive survival tail, time-unit/affine-X
   covariance transforms. Strict convergence/final full curvature are observable.
4. Joint eligibility/original DECIMAL codes, no-event/rank/convergence/support/
   resource/unsupported-option failures, stale-state/labels/order preservation,
   current edits/drop event/time0, schema/target/overflow and no partial columns.
5. Parser→runtime→owned state→text/CLI-v1/help/source execution complete, focused
   then affected/reverse-consumer regressions and risk-proportionate source review,
   honest SPEC/ledger updates, ready PR merge/sync and branch cleanup before any
   new analysis. Ordinary reference1e-9 absolute+1e-8 relative; positive tails
   1e-300+1e-8 relative; source hex exact/printed half-last-unit. Do not widen
   tolerances for unexplained differences. Quadrature/finite-difference numerical
   certificates have separately justified approximation bounds.

## Scope and delivery order

Existing Rust crates/shared guarded solver/typed transactional data adapter;
no new production dependency/native/unsafe code or general estimator framework.
No margins/general restrictions/named-model storage, new graphics, automatic
panel/survival declarations, formal model adequacy, prediction uncertainty, full
Table17.9 hazard-ratio uncertainty or whole chapter/family coverage. This closes
one bounded column; heterogeneity/multiple hazards and other required work remain.

Order: early WIP contract → reproducible independent R/control artifacts → safe
kernel/moment and inference tests → runtime/current points/reporting → original
native script/source audit → affected/reverse checks and bounded statistical
source review → capability docs → ready/merge/sync/safe branch cleanup. Stop for
substantive source/statistical/scope conflicts rather than changing the target
or merging a parser/kernel-only milestone.
