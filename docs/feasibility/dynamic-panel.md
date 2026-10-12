# Dynamic-panel difference-GMM contract (in progress)

Base f78be4a. Independent dpanel-plugin, source-matched dpanel.so sharing verified
core/lazy serialized lifecycle. One-step first-difference Arellano–Bond with one
lagged outcome, collapsed lag-2-only level outcome instrument, strictly exogenous
predictors instrumented by their differences. Individual effects removed by
transformation, no time effects/intercept/system/two-step/Windmeijer route.

Declare finite complete balanced entity-major/time-minor zero-based keys; seed
29183, 64 entities × 8 periods, two predictors, usable t=2..7 (384 rows).
Native model residual/sample mask must agree with expected keys, not merely N.
Own beta `L1.y`, x1/x2, full 3×3 covariance/SE and exact keys/instrument/group counts.
Covariance is entity-summed one-step sandwich without finite-N correction, not
independent-row heteroskedasticity. Reference distribution/CI/AR/Hansen/Sargan,
prediction/session are not exposed by this bounded result.

Predeclare abs 1e-9 + rel 1e-8*abs(reference) for all beta/SE/covariance, exact
names/shape/sample and finite values. Compare pinned executor points and preserve
its original SEs/VCV where obtainable. It uses pooled single-level-lag IVGMM with
independent-row covariance (and defaults to two GMM steps), not general block AB.
This collapsed just-identified fixture makes points weight-independent; it does
not validate overidentified weighting, multi-lag instruments or whole xtabond.
Independent R plm pgmm(one-step, individual, difference, collapse) vcovHC and
separate external Python entity-sandwich algebra define intended inference.
Explain original covariance disagreement explicitly, never widen tolerance.

Check metadata changes/restoration, list/lag/index/dimension arithmetic before C,
owned MODEL/sample copies and failures/recovery/concurrency/missing module. Default
static and other feature-only builds remain unchanged. No new Cargo dependency,
waiver, runtime Python/R or product/final-backend claim. Broader native integer/
allocation/sanitizer/leak/license/relocation gates remain separate.
