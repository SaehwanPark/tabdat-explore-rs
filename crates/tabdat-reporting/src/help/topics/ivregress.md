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
excluded-instrument partial F tests and partial R-squared values. These are
relevance diagnostics, **not weak-instrument-robust inference**. Shea's partial
R-squared, overidentification tests, weak-IV-robust intervals and GMM are not
implemented; `gmm` is parsed but execution reports it as unsupported.

Examples:
- `ivregress 2sls wage age, endog(educ) iv(distance)`
- `ivregress 2sls wage76 black south76, endog(grade76 exp76 expsq76) iv(col4 age76 agesq76) robust`
