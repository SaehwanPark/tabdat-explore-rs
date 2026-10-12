# Tobit feasibility contract (in progress)

Base 720902b. Classical Gaussian Tobit, intercept and two predictors, finite
complete ordered sample. Probe both left-only censoring at zero and two-sided
censoring at zero/two. Owned beta coefficients/SEs, joint beta/log(sigma) covariance,
log(sigma), likelihood and exact sample. Predeclared MLE tolerance abs 1e-7 +
rel 1e-6; exact names/shape/sample and positive scale. Compare pinned executor
(route explicitly recorded) and independent R survival Gaussian interval2 fits.
No Python/R runtime dependency; these are external reference tools only.

Add opt-in tobit-plugin building only same-source interval.so against the verified
shared core, without changing default static or count-only capabilities. Reuse
serialized initialization/plugin lifetime, private bounded path and RAII model.
No robust/cluster, general missing samples, model/session state or product parity
claim. Check invalid limits/design, all-censored failure/recovery and threading.
Preserve any fallback Python covariance discrepancy rather than widen tolerances.
