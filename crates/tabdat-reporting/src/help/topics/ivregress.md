# ivregress

How to invoke:
`ivregress 2sls y [exog...], endog(x1 x2) iv(z1 z2) [robust|cluster(id)|noconstant]`

The options belong after one comma, for example:
`ivregress 2sls y exog..., endog(x1 x2) iv(z1 z2) robust`

What it does:
Fits linear two-stage least squares (2SLS). Included exogenous regressors are
also included instruments; `iv()` lists excluded instruments. Multiple endogenous
regressors are supported when the instrument matrix identifies the full design.

Sample and inference:
Rows with missing or non-finite values in the outcome, regressors, excluded
instruments, or requested cluster variable are excluded together. Default
covariance is classical; `robust` requests HC1 and `cluster(id)` requests
one-way cluster-robust covariance. Results include conventional first-stage
excluded-instrument partial F tests, ordinary partial R-squared and Shea's partial
R-squared. Shea's statistic accounts for the other endogenous regressors; it is a
design diagnostic independent of the requested covariance on the same sample.
These are relevance diagnostics, **not weak-instrument-robust inference**.
After a nonrobust fit, `estat ar, null(b1 b2 ...)` supplies a separate
homoskedastic weak-IV-robust joint test of all endogenous coefficients; see
`help estat` for null ordering, nuisance parameters and assumptions. Ordinary
coefficient p-values remain conventional. With exactly one endogenous regressor,
`estat ar, ci [level(95)]` provides homoskedastic confidence sets,
including empty or unbounded sets; see `help estat` for numerical limits.
Overidentification tests, multi-endogenous confidence regions, robust/cluster AR
and GMM are not implemented;
`gmm` is parsed but execution reports it as unsupported.

Examples:
- `ivregress 2sls wage age, endog(educ) iv(distance)`
- `ivregress 2sls wage76 black south76, endog(grade76 exp76 expsq76) iv(col4 age76 agesq76) robust`
