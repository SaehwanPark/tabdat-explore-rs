# Chapter 24 individual shared-gamma Poisson RE — implementation contract

**Status: planned/in progress, not installed.** Existing native conditional FE
and pooled workflows are unchanged. This is the next bounded slice, not whole
Table24.6, chapter or survey-method completion.

## Exact outcome and semantics

Add native `xtpoisson y xvars, re id(group)` plus an executable §24.7/Table24.6
individual gamma-RE fit/report/marginal-prediction export workflow. The pinned
source, preparation and ten numeric predictors are in [README](README.md).
Joint-finite selection retains 27,765 people in all194 communes, including
commune64's 94 all-zero records; source row27734 is entirely missing and removed
by preparation/workflow before structural-key audit. People are cross-sectional;
source-row is not time. Fit an intercept plus the original ten numeric columns;
no factors, weights or offsets. Preserve original source/identity ordering.

Given independent group effects `u_g ~ Gamma(shape=1/alpha,rate=1/alpha)`, counts
are conditionally independent `Poisson(u_g*exp(X_i*beta))`. Thus E(u)=1 and
Var(u)=alpha: shared-group mixing, not independent-row NB2. With r=1/alpha,
S=sum(y) and M=sum(exp(X*beta)) within group, the integrated contribution is
`lgamma(r+S)-lgamma(r)+r*log(r)-(r+S)*log(r+M)+sum(y*X*beta-lgamma(y+1))`.
All-zero groups and singletons remain. Exact raw group keys are structural even
on rows omitted for model missingness.

Estimate coefficients and ln(alpha) jointly. Use inverse full joint observed
information, including coefficient/dispersion cross terms, normal95 coefficient
and ln(alpha) inference, delta alpha SE and exponentiated log-scale alpha CI.
Slope-only Wald uses the slope covariance block and chi-square reference.
Compare with re-estimated same-sample pooled Poisson at alpha=0 using the
conventional 50:50 chi-square(0)/chi-square(1) boundary LR reference. No LR
reference is to be inferred for robust/cluster inference. No slope-null LR or
unit-specific posterior effect inference is promised.

`predict ..., n` is the marginal population mean exp(X*beta), xb the marginal
log index, residuals current y minus marginal mean. These do not require a
current group key or estimate posterior effects. Retain owned fitted state,
explicit method/sample/covariance labels and convergence information.

## Bounded acceptance cases

1. Independent R direct integrated likelihood/analytic score/full information,
   central score-derivative check and NB-total/conditional-multinomial likelihood
   decomposition validate coefficients, all covariance/inference/dispersion/LR
   fields and every source/marginal-prediction row. Predeclare absolute1e-9,
   likelihood1e-8, nonzero-p relative1e-8, exact underflow zero; investigate
   discrepancies rather than widen tolerances. Separately audit pinned DTA and
   printed author facts, preserving precision/optimizer distinctions.
2. A small multivariate unbalanced/interleaved reference includes positive and
   all-zero groups, singleton, missing case and independent-row-NB counterexample;
   ordering/scaling controls preserve the declared fit and marginal means.
3. CLI text/JSON and actual script exercise load/prepare → specify → fit →
   interpret/report → predict/export. Discovery/help describe the supported RE
   distribution and limits without changing native FE behavior.
4. Invalid keys/counts, rank deficiency, nonpositive final joint information,
   boundary/no-interior dispersion and iteration failure are explicit; no silently
   clipped alpha, dropped groups or Poisson fit mislabeled as gamma RE. Failed new
   fits clear stale models without changing data; syntax errors preserve state.
5. Prediction failure is atomic; successful/failed export preserves source and
   existing owner files and retains fit. Script effects before export failure
   remain intentionally nontransactional.

Initial resource limits to enforce/test: candidate rows<=40000, slopes1..32,
rows*(slopes+2)^2<=20million, raw groups<=10000, retained groups>joint parameter
count, exact individual counts and selected group totals<=100000. The group-total
limit bounds stable integer-recurrence work/cache, rather than looping up to an
unbounded total; this original sample's maximum is310. Require an interior
ln(alpha) strictly in(-12,12), full-rank design and finite positive final observed
information, with strict scaled score/step convergence. Boundary fits fail
explicitly in this bounded slice; automatic boundary fallback is unsupported.
Reuse existing stats/domain/execution/reporting crates and numerical helpers;
no production Python/R dependency or general optimizer framework.

Exclusions: normal RE, robust/cluster/survey covariance, offsets/weights,
factors, posterior effects, generic margins/post-estimation and automatic
bootstrap. These and the remaining required textbook outcomes are not waived.

## Preliminary independent feasibility evidence (not native validation)

A development-only R4.6.2 base-R BFGS fit, refined by full observed information,
retains all27,765 people/all194 communes and gives ASCII log likelihood
-23419.13188988957, ln(alpha) -1.0398391652175825, alpha0.35351153433744487,
LNHHEXP -0.10137463292705126 and intercept -1.2769738417366050. Full information
is positive; central score derivatives reproduce covariance within1e-8;
NB-total/conditional likelihood decomposition differs by5.821e-11. This is a
feasibility probe, not a checked-in full oracle or a printed-author/native audit.
The initial unscaled BFGS objective could overshoot into numerically unstable
near-zero dispersion; optimizing likelihood/score per group avoids that probe
failure without changing the objective's optimum. Stable recurrence evaluation
and bounded native convergence still need implementation and validation.
