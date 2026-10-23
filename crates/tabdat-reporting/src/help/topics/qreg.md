# qreg

Installed syntax:
`qreg y x1 x2 [, quantile(value) noconstant]`

Fit an unweighted linear conditional quantile, minimizing the check loss
`sum(r * (tau - I[r < 0]))`. The default is the median (`tau=0.5`); supported
finite quantiles range from 0.05 through 0.95. Coefficients describe conditional
quantiles, not means. An intercept is included unless `noconstant` is requested.

Examples:
- `qreg cost age bmi`
- `qreg cost age bmi, quantile(0.25)`
- `predict fitted_quantile`
- `predict residual, residuals`
- `export predictions.csv`

The joint complete-finite-case sample follows current row order. Variables must
be numeric; no weights, factors, interactions or clustered fit are installed.
The parsed `robust` option fails explicitly. A new fit attempt clears the previous
fit even on failure; data, labels and named reports remain unchanged.

SEs/covariance use IID location errors: Hall–Sheather bandwidth, local residual
spacing median regression, and `tau*(1-tau)*sparsity^2*inv(X'X)`, matching
R quantreg `summary.rq(se="iid", hs=TRUE)` covariance. Reported z tests and 95%
intervals use the asymptotic standard normal, with no finite-sample t correction.
This inference assumes a common error density independent of the design; it is
not heteroskedastic-robust, clustered, weighted or survey inference. Separate
quantiles do not supply cross-quantile covariance or simultaneous tests.

The sparse dual LP uses Clarabel 0.11.1 with QR polishing and explicit numerical
optimality checks. Limits: N>K, N<=10,000, K<=32 including the intercept,
N*K²<=10,000,000; 200 LP iterations and 10 seconds per solve. Unresolved or
nonisolated solutions, rank deficiency, insufficient residual spacing and
nonpositive/overflowing sparsity or uncertainty fail rather than returning an
approximate successful fit. Runtime is not guaranteed at the numerical limits.

`predict` supports xb (the fitted conditional quantile, default) and residuals
on current rows. It uses retained coefficients without refitting. Prediction
column errors preserve data/model state. NULL/NaN/infinity arithmetic follows the
existing linear prediction contract. Quantile `lincom`, `test`, `estat margins`,
named estimate storage and prediction intervals are not installed.

Pseudo R² is `1 - check_loss/null_check_loss`; the null model uses an empirical
unconditional quantile and is absent without an intercept. Check loss at the
median is half the absolute-deviation sum; it is not a likelihood.

See `examples/mma04p2qreg/README.md` for the original-source household workflow,
independent references and historical Stata input/convention differences.

For heteroskedastic IID pairs uncertainty and cross-quantile covariance, see
`help bootstrap`: `bootstrap y x, quantiles(.25 .5 .75) seed(10101)`.
This separate read-only report preserves qreg's retained fit and does not change
its IID-location covariance, prediction behavior or post-estimation support.
