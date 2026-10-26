# Chapter 5.9.4: exponential mean effects via Poisson event/exposure algebra.
# Poisson diagnostics describe the computational primitive, not the source law.
use nls.csv
isid source_row
sql """
SELECT CASE WHEN COUNT(*)<=2 OR COUNT(*)>10000
  OR MIN(source_row)<>1 OR MAX(source_row)<>COUNT(*)
  OR COUNT(*) FILTER (WHERE NOT coalesce(isfinite(source_row)
    AND source_row=floor(source_row) AND source_row>0
    AND isfinite(x) AND isfinite(y) AND y>0,FALSE))>0
  OR NOT coalesce(isfinite(stddev_samp(x)) AND stddev_samp(x)>0,FALSE)
THEN error('Exponential effects require 3..10000 ordered unique dense rows, positive finite y and finite nonconstant x')
ELSE COUNT(*) END AS observations FROM active
"""
sql """SELECT *, 1 AS event, ln(y) AS log_exposure, x AS original_x,
  avg(x) OVER () AS mean_x, 0.0001*stddev_samp(x) OVER () AS step_x
FROM active ORDER BY source_row""" into exponential_design
poisson event x, offset(log_exposure)
predict integrated_rate, n
replace x = original_x+1
predict integrated_rate_plus_one, n
replace x = original_x+step_x
predict integrated_rate_plus_step, n
replace x = mean_x
predict integrated_rate_at_mean, n
replace x = original_x
sql """
SELECT source_row, y, x, mean_x, step_x,
  y/integrated_rate AS fitted_mean,
  -ln(integrated_rate_plus_one/integrated_rate)*y/integrated_rate AS analytical_derivative,
  (y/integrated_rate_plus_step-y/integrated_rate)/step_x AS numerical_derivative,
  y/integrated_rate_plus_one-y/integrated_rate AS finite_difference,
  -ln(integrated_rate_plus_one/integrated_rate)*y/integrated_rate_at_mean AS derivative_at_mean
FROM active ORDER BY source_row
""" into exponential_effects
sql """
SELECT CASE WHEN COUNT(*) FILTER (WHERE NOT coalesce(
  isfinite(fitted_mean) AND fitted_mean>0 AND isfinite(analytical_derivative)
  AND isfinite(numerical_derivative) AND isfinite(finite_difference)
  AND isfinite(derivative_at_mean),FALSE))>0
THEN error('Exponential effects require finite, representable fitted means and effects')
ELSE COUNT(*) END AS valid_effects FROM active
"""
sql """SELECT COUNT(*) AS observations, AVG(analytical_derivative) AS average_derivative,
  AVG(numerical_derivative) AS average_numerical_derivative,
  AVG(finite_difference) AS average_finite_difference,
  AVG(derivative_at_mean) AS derivative_at_mean FROM active"""
export exponential_effects.csv
