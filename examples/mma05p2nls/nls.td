# Unweighted Table5.7 NLS column only; generated data, not an empirical cohort.
# Mean exp(-a-b*x), rate reciprocal(mean); no exponential-ML/FGNLS claim.
use nls.csv
isid source_row
sql """
SELECT CASE WHEN COUNT(*)<=2 OR COUNT(*)>10000
  OR MIN(source_row)<>1 OR MAX(source_row)<>COUNT(*)
  OR COUNT(*) FILTER (WHERE NOT coalesce(
    isfinite(source_row) AND source_row=floor(source_row) AND source_row>0
    AND isfinite(x) AND isfinite(y) AND y>0,FALSE))>0
THEN error('NLS source workflow requires 3..10000 complete finite rows, positive generated y and unique dense source_row1..N')
ELSE COUNT(*) END AS observations,
  'generated exponential-mean data; unweighted NLS, not exponential ML' AS analysis
FROM active
"""
nl y = exp(-a-b*x), params(a b) start(1 0)
predict mean_classical, mean
predict residual_classical, residuals
nl y = exp(-a-b*x), params(a b) start(1 0) robust
predict mean_hc1
predict residual_hc1, residuals
# Plug-in rate under the SOURCE exponential law; no uncertainty/adequacy claim.
generate fitted_rate = 1/mean_hc1
generate squared_residual = residual_hc1*residual_hc1
sql """
SELECT COUNT(*) AS observations, AVG(y) AS observed_mean,
  AVG(mean_hc1) AS mean_fitted_response, SUM(squared_residual) AS residual_sum_of_squares,
  AVG(residual_hc1) AS mean_residual,
  MAX(abs(mean_classical-mean_hc1)) AS covariance_mode_mean_difference,
  MAX(abs(residual_classical-residual_hc1)) AS covariance_mode_residual_difference,
  'descriptive residual summaries, not an adequacy test; covariance assumes correct mean' AS scope
FROM active
"""
export nls_predictions.csv
