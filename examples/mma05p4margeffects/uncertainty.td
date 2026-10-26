# Conditional-X delta-method effect uncertainty for the exponential density.
# The native Poisson output is a score/information primitive, not a count DGP.
use nls.csv
isid source_row
sql """
SELECT CASE WHEN COUNT(*)<=2 OR COUNT(*)>10000
  OR MIN(source_row)<>1 OR MAX(source_row)<>COUNT(*)
  OR COUNT(*) FILTER (WHERE NOT coalesce(isfinite(source_row)
    AND source_row=floor(source_row) AND source_row>0
    AND isfinite(x) AND isfinite(y) AND y>0,FALSE))>0
  OR NOT coalesce(isfinite(stddev_samp(x)) AND stddev_samp(x)>0,FALSE)
THEN error('Exponential effects require 3..10000 unique dense rows, positive finite y and finite nonconstant x')
ELSE COUNT(*) END AS observations FROM active
"""
sql """SELECT *, 1 AS event, ln(y) AS log_exposure, x AS original_x,
  avg(x) OVER () AS mean_x FROM active ORDER BY source_row""" into effects_design
poisson event x, offset(log_exposure)
predict integrated_rate, n
replace x = original_x+1
predict integrated_rate_plus_one, n
replace x = mean_x
predict integrated_rate_at_mean, n
replace x = original_x
sql """
SELECT CASE WHEN COUNT(*) FILTER (WHERE NOT coalesce(
  isfinite(integrated_rate) AND integrated_rate>0
  AND isfinite(integrated_rate_plus_one) AND integrated_rate_plus_one>0
  AND isfinite(integrated_rate_at_mean) AND integrated_rate_at_mean>0
  AND isfinite(y/integrated_rate) AND y/integrated_rate>0
  AND isfinite(ln(integrated_rate_plus_one/integrated_rate)),FALSE))>0
THEN error('Exponential effect predictions are not finite and positive')
ELSE COUNT(*) END AS valid_predictions FROM active
"""
# Information is sum(y*lambda_i*[1,x_i]'[1,x_i]); the same converged fit is used.
# All gradients condition on the observed x; uncertainty excludes sampling x itself.
sql """
WITH moments AS (
  SELECT count(*) AS n, avg(original_x) AS xbar,
    avg(ln(integrated_rate_plus_one/integrated_rate)) AS b,
    avg(y/integrated_rate) AS mu_bar,
    avg(original_x*y/integrated_rate) AS x_mu_bar,
    avg(y/integrated_rate_at_mean) AS mu_at_mean,
    sum(integrated_rate) AS i00,
    sum(integrated_rate*original_x) AS i01,
    sum(integrated_rate*original_x*original_x) AS i11
  FROM active
), inv AS (
  SELECT *, i00*i11-i01*i01 AS det FROM moments
), gradients AS (
  SELECT *,
    -b*mu_bar AS ame, b*mu_bar AS ame_a, -mu_bar+b*x_mu_bar AS ame_b,
    (exp(-b)-1)*mu_bar AS fd, -(exp(-b)-1)*mu_bar AS fd_a,
    -exp(-b)*mu_bar-(exp(-b)-1)*x_mu_bar AS fd_b,
    -b*mu_at_mean AS mem, b*mu_at_mean AS mem_a,
    (-1+b*xbar)*mu_at_mean AS mem_b
  FROM inv
), variances AS (
  SELECT *,
    (i11*ame_a*ame_a-2*i01*ame_a*ame_b+i00*ame_b*ame_b)/det AS ame_v,
    (i11*fd_a*fd_a-2*i01*fd_a*fd_b+i00*fd_b*fd_b)/det AS fd_v,
    (i11*mem_a*mem_a-2*i01*mem_a*mem_b+i00*mem_b*mem_b)/det AS mem_v
  FROM gradients
)
SELECT n, b, det, i00, i01, i11, ame, ame_v, fd, fd_v, mem, mem_v
FROM variances
""" into effects_variances
sql """
SELECT CASE WHEN COUNT(*)<>1 OR COUNT(*) FILTER (WHERE NOT coalesce(
  isfinite(det) AND isfinite(i00) AND isfinite(i01) AND isfinite(i11)
  AND i00>0 AND i11>0 AND det>1e-12*i00*i11
  AND isfinite(ame) AND isfinite(fd) AND isfinite(mem)
  AND isfinite(ame_v) AND ame_v>0 AND isfinite(fd_v) AND fd_v>0
  AND isfinite(mem_v) AND mem_v>0,FALSE))>0
THEN error('Exponential effect information or delta variance is ill-conditioned or nonrepresentable')
ELSE COUNT(*) END AS valid_inference FROM active
"""
sql """
SELECT n, 'average analytical derivative' AS effect, ame AS estimate, sqrt(ame_v) AS standard_error,
  ame-1.959963984540054*sqrt(ame_v) AS ci_lower, ame+1.959963984540054*sqrt(ame_v) AS ci_upper,
  'normal95, conditional x, exponential model-based' AS inference FROM active
UNION ALL SELECT n, 'average one-unit difference', fd, sqrt(fd_v),
  fd-1.959963984540054*sqrt(fd_v), fd+1.959963984540054*sqrt(fd_v),
  'normal95, conditional x, exponential model-based' FROM active
UNION ALL SELECT n, 'derivative at mean x', mem, sqrt(mem_v),
  mem-1.959963984540054*sqrt(mem_v), mem+1.959963984540054*sqrt(mem_v),
  'normal95, conditional x, exponential model-based' FROM active
""" into effects_uncertainty
sql """
SELECT CASE WHEN COUNT(*)<>3 OR COUNT(*) FILTER (WHERE NOT coalesce(
  isfinite(estimate) AND isfinite(standard_error) AND standard_error>0
  AND isfinite(ci_lower) AND isfinite(ci_upper),FALSE))>0
THEN error('Exponential effect intervals are not finite and representable')
ELSE COUNT(*) END AS valid_intervals FROM active
"""
sql """SELECT * FROM active"""
export effects_uncertainty.csv
