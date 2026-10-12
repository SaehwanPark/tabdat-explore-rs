# Heckman two-step feasibility contract (in progress)

Base 8606026. Complete finite two-equation sample, binary selection with both
classes, explicit intercepts, two outcome predictors and an excluded selection
predictor. Compare pinned Python two-step coefficients (probit then selected OLS
with inverse Mills), independent R probit/QR and corrected covariance algebra,
and separately labeled Python two-step uncertainty correction.

Predeclare abs 1e-7 + rel 1e-6 for coefficients/SEs/all equation covariance,
sigma/rho; exact total selection rows and selected outcome IDs, names/shape.
Use native OPT_T, not unrequested MLE. Proper two-step outcome covariance includes
first-stage uncertainty; preserve original ordinary-OLS SEs and explicitly record
any discrepancy rather than pretend parity. Selection covariance is observed
probit information. Native joint matrix fills cross-equation block with zeros:
expose separate equation covariances, not a valid full joint cross covariance.

Independent heckman-plugin builds only same-source heckit.so under shared-core/
serialized owned lifetime controls. No robust/cluster, MLE, missing-row deletion,
no-intercept, correlation-boundary, prediction/session or product support. Ignore
unselected outcome values only after requiring the bounded complete-data contract;
exact native total/selected counts must match expected selection masks.
