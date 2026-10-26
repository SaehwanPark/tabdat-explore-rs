# Table 5.7: two OLS and two exponential rate fits on the same original sample.
# Poisson output is the event/exposure likelihood primitive, not a count model.
use nls.csv
isid source_row
sql """
SELECT CASE WHEN COUNT(*)<=2 OR COUNT(*)>10000
  OR MIN(source_row)<>1 OR MAX(source_row)<>COUNT(*)
  OR COUNT(*) FILTER (WHERE NOT coalesce(isfinite(source_row)
    AND source_row=floor(source_row) AND source_row>0
    AND isfinite(x) AND isfinite(y) AND y>0,FALSE))>0
  OR NOT coalesce(isfinite(stddev_samp(x)) AND stddev_samp(x)>0,FALSE)
  OR NOT coalesce(isfinite(stddev_samp(y)) AND stddev_samp(y)>0,FALSE)
THEN error('Table 5.7 workflow requires 3..10000 unique dense rows, positive finite nonconstant y and finite nonconstant x')
ELSE COUNT(*) END AS observations FROM active
"""
sql """SELECT 'Poisson results represent exponential rate fits; their reported log likelihood includes the log(y) offset' AS analysis_note FROM active LIMIT 1"""
regress y x
predict ols_classical, xb
regress y x, robust
predict ols_hc1, xb
sql """SELECT *, 1 AS event, ln(y) AS log_exposure FROM active ORDER BY source_row""" into exponential_design
poisson event x, offset(log_exposure)
predict integrated_classical, n
poisson event x, offset(log_exposure) robust
predict integrated_robust, n
sql """
SELECT source_row, y, x, ols_classical, ols_hc1,
  integrated_classical, integrated_robust,
  y/integrated_classical AS exponential_mean,
  ln(integrated_classical/y)-integrated_classical AS density_log_likelihood,
  y-y/integrated_classical AS exponential_residual,
  y-ols_classical AS ols_residual
FROM active ORDER BY source_row
""" into comparison_predictions
sql """
SELECT CASE WHEN COUNT(*) FILTER (WHERE NOT coalesce(
  isfinite(ols_classical) AND isfinite(ols_hc1)
  AND isfinite(integrated_classical) AND integrated_classical>0
  AND isfinite(integrated_robust) AND integrated_robust>0
  AND isfinite(exponential_mean) AND exponential_mean>0
  AND isfinite(density_log_likelihood) AND isfinite(exponential_residual)
  AND isfinite(ols_residual),FALSE))>0
THEN error('Table 5.7 projections require finite representable values')
ELSE COUNT(*) END AS valid_predictions FROM active
"""
sql """
SELECT CASE WHEN NOT coalesce(
  isfinite(sum(density_log_likelihood)) AND isfinite(var_pop(y)) AND var_pop(y)>0
  AND isfinite(sum(exponential_residual*exponential_residual))
  AND isfinite(sum(ols_residual*ols_residual))
  AND isfinite(1-sum(exponential_residual*exponential_residual)/(var_pop(y)*count(*)))
  AND isfinite(1-sum(ols_residual*ols_residual)/(var_pop(y)*count(*))),FALSE)
THEN error('Table 5.7 summary requires finite representable sums and positive outcome variance')
ELSE COUNT(*) END AS valid_summary FROM active
"""
sql """
SELECT COUNT(*) AS observations, sum(density_log_likelihood) AS exponential_density_log_likelihood,
  1-sum(exponential_residual*exponential_residual)/(var_pop(y)*count(*)) AS exponential_level_r_squared,
  1-sum(ols_residual*ols_residual)/(var_pop(y)*count(*)) AS ols_r_squared,
  max(abs(ols_classical-ols_hc1)) AS ols_covariance_mode_prediction_difference,
  max(abs(integrated_classical-integrated_robust)) AS exponential_covariance_mode_prediction_difference,
  'level R-squared is descriptive; OLS and exponential likelihoods differ' AS interpretation
FROM active
"""
export comparison_predictions.csv
