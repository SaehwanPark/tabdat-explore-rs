# Quantile regression workflow (WIP)

Target §4.6.4 pp88–90, mma04p2qreg.do block1: conditional median of log
household medical expenditure on log total expenditure with an intercept.
The original ASCII qreg0902.asc has5,999 rows; exclude993 missing medical-log
observations to obtain5,006 jointly finite cases, preserving source identity.

Implement bounded unweighted qreg, current-row xb/residual predictions and
explicit IID-location-error covariance. Point estimates minimize check loss;
validate against R quantreg and certify LP residual/duality/identification.
Use the R quantreg Hall–Sheather residual-spacing sparsity estimator for IID
covariance, with explicitly asymptotic normal tails/95% intervals. No heteroskedastic
or cluster covariance, weights, cross-quantile tests or simultaneous bootstrap.

Clarabel0.11.1 is a candidate pure-Rust LP backend with no optional native/Python
features. Numerical feasibility, source/sample integrity, dependency policy,
state/failure contracts and independent numerical evidence are pending.
This is not installed capability or complete replication of Figures4.1–4.2.
