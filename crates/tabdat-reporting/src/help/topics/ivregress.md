# ivregress

How to invoke:
`ivregress 2sls y [exog...], endog(x1 x2) iv(z1 z2) [robust|cluster(id)|noconstant]`

`ivregress gmm y, endog(x1 x2) iv(z1 z2 z3) noconstant cluster(id)`

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
`estat ar, null(b1 b2 ...)` supplies a separate weak-IV-robust joint test of all
endogenous coefficients: classical F after a nonrobust fit, auxiliary HC1/CR1
Wald asymptotic chi-square after a robust/cluster fit. See
`help estat` for null ordering, nuisance parameters and assumptions. Ordinary
coefficient p-values remain conventional. With a nonrobust fit and exactly one endogenous regressor,
`estat ar, ci [level(95)]` provides homoskedastic confidence sets,
including empty or unbounded sets; see `help estat` for numerical limits.
Overidentified nonrobust fits additionally support `estat overid`: homoskedastic
Sargan N*R² with asymptotic chi-square inference. Independent-row robust fits
instead return a Wooldridge residualized-instrument overidentification score,
with uncorrected score OPG (not the parent's HC1 correction), chi-square(L-K)
reference and retained structural residuals. Neither is Hansen J or proof of
instrument validity. See `help estat` for strong-identification assumptions,
DF, retained-residual and failure semantics. `estat endogenous` additionally
provides the joint homoskedastic regression-based Wu–Hausman F test of all declared
endogenous regressors, even for just-identified models. It assumes valid
instruments and uses N-K-m augmented-regression residual DF; it is not a Durbin LM,
robust/cluster or endogenous-subset test. Robust/cluster parent fits instead
support a distinct augmented-OLS joint innovation Wald test with asymptotic
chi-square(m) inference and auxiliary HC1/CR1 corrections (Kaux=K+m), not an
exact F or weak-IV-robust test. See `help estat` for assumptions/numerical limits.
Subset/robust-score endogeneity, clustered 2SLS overid, multi-endogenous
confidence regions and robust/cluster confidence sets are not implemented.

Clustered two-step GMM:
The bounded `gmm` form requires `cluster(id) noconstant`, with no included
exogenous variables and no weights. It jointly excludes missing/non-finite
outcome, endogenous, instrument and cluster values. Up to 10,000 retained rows,
8 endogenous regressors and 16 instruments are supported; the design and both
cluster moment covariance matrices must have full numerical rank and there must
be more clusters than instruments. Perfect/near-perfect fit is rejected. No
pseudoinverse, cluster finite-sample correction or iterative GMM is used.

The main standard errors, z tests and normal 95% intervals use the full
fixed-pilot-weight cluster sandwich. The fixed-weight J uses the same pilot
weight and is the conventional objective. Separately, the result reports the
source author's inverse-information standard errors and J computed using a
cluster moment covariance updated at the second-step residuals. These are not
the fixed-weight sandwich or its objective. Both J references are asymptotic
chi-square with instruments minus regressors degrees of freedom and are absent
when exactly identified. Results assume independent clusters and valid moments;
none of these tests verifies instrument validity or provides weak-IV-robust
inference. Use `predict ..., xb` or `predict ..., residuals` on current data;
2SLS post-estimation commands and stored estimates do not support GMM.

Examples:
- `ivregress gmm y, endog(x1 x2) iv(z1 z2 z3) noconstant cluster(id)`
- `ivregress 2sls wage age, endog(educ) iv(distance)`
- `ivregress 2sls wage76 black south76, endog(grade76 exp76 expsq76) iv(col4 age76 agesq76) robust`
