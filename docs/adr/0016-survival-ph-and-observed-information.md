# ADR 0016: Normalize survival PH parameters and observed information

- Status: Accepted for bounded feasibility, not product streg support
- Date: 2026-10-12

## Evidence and decision

Pinned streg fits proportional-hazard log-rate gamma with exponential or Weibull
hazard `exp(X gamma) * shape * time^(shape-1)`; failures are 1 and right-censored
observations 0. Native duration uses AFT beta/raw sigma and the opposite censor
dummy. Normalize exponential gamma=-beta. For Weibull gamma=-beta/sigma and
log(shape)=-log(sigma), using full joint `J V J'` with cross terms. Report correct
parameter names and retain exact sample/failure/censor row identities.

Native `duration.c::duration_loglik` evaluates event density on log(time). An
initial comparison failed by 163.860533595 for exponential and the corresponding
Weibull Jacobian is 52.787478957. Correct owned time-density likelihood by subtracting
sum(log(time)) over failures only; censored survivor terms do not change. This
constant normalization does not change the optimizer/coefficients/covariance.
Time-unit metamorphic checks verify the likelihood shifts by failure-count*log(unit),
not sample-count*log(unit), and the PH intercept shifts correctly.

Original pinned Python coefficients pass unchanged MLE tolerance, but scipy
BFGS/L-BFGS-B inverse-Hessian approximation SEs disagree by ~7.24e-4 exponential
and ~9.78e-4 Weibull. Preserve originals and, as in ADR 0014, accept observed
information validated against independent R survival and a separately labeled
external Python observed-Hessian probe using the same objective/start/optimizer.
Do not describe it as original SE parity, modify the checkout or widen tolerance.

## Native scope and safety consequences

Independent `survival-plugin` selects only same-source duration.so under existing
shared-core/lifetime/source controls. Native requires a constant: the bounded
facade rejects no-intercept requests. All-censored samples are unidentified and
rejected. C shim also checks positive finite times/binary censoring and events,
protecting the public low-level wrapper, not only the facade.

Native precheck blindly decrements initial OLS rank for a supplied censor dummy.
An all-zero dummy was already dropped by OLS, so strip it on an owned copied list
for all-event input; never mutate the Rust borrowed slice or silently lose a real
predictor. The facade omits the separator too. Mixed censor-dummy collinearity can
still cause native rank loss: exact coefficient/sample checks reject it, with no
claim to support that valid-but-backend-limited design. Partial owned-list/model
allocation failures clean up using existing RAII/native ownership.

No runtime R/Python, Cargo dependency, plugin unload/path setter, robust/cluster,
Cox, left truncation, competing risks, model state or distribution acceptance.
Full native safety/sanitizer/leak, relocation and licensing gates remain open.

[Survival report](../feasibility/survival.md),
[original output](../../spikes/gretl-prototype/fixtures/survival-original-python.csv),
[external observed probe](../../spikes/gretl-prototype/fixtures/survival_observed.py).
