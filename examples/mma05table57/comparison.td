# Five Table 5.7 specifications with separately labeled working/robust reports.
# Poisson output is an exponential-rate algebraic primitive, not a count DGP.
use nls.csv
isid source_row
sql """
SELECT CASE WHEN COUNT(*)<=2 OR COUNT(*)>10000
  OR MIN(source_row)<>1 OR MAX(source_row)<>COUNT(*)
  OR COUNT(*) FILTER (WHERE source_row<>input_position)>0
  OR COUNT(*) FILTER (WHERE NOT coalesce(isfinite(source_row)
    AND source_row=floor(source_row) AND source_row>0
    AND isfinite(x) AND isfinite(y) AND y>0,FALSE))>0
  OR NOT coalesce(isfinite(stddev_samp(x)) AND stddev_samp(x)>0,FALSE)
  OR NOT coalesce(isfinite(stddev_samp(y)) AND stddev_samp(y)>0,FALSE)
THEN error('Table 5.7 requires 3..10000 ordered unique dense rows, finite nonconstant x and positive finite nonconstant y')
ELSE COUNT(*) END AS observations FROM (SELECT *,row_number() OVER () AS input_position FROM active)
"""
sql """SELECT 'Poisson columns use event/exposure algebra; NLS a,b and OLS/Poisson intercept,x are distinct names. Weighted NLS report statistics use transformed responses.' AS interpretation FROM active LIMIT 1"""
regress y x
estimates store ols_working
predict ols_mean, xb
regress y x, robust
estimates store ols_robust
sql """SELECT *,1 AS event,ln(y) AS log_exposure FROM active ORDER BY source_row""" into exponential_design
poisson event x, offset(log_exposure)
estimates store exp_working
predict exp_integrated, n
poisson event x, offset(log_exposure) robust
estimates store exp_robust
nl y = exp(-a-b*x), params(a b) start(1 0)
estimates store nls_working
predict nls_mean, mean
nl y = exp(-a-b*x), params(a b) start(1 0) robust
estimates store nls_robust
sql """
SELECT CASE WHEN COUNT(*) FILTER (WHERE NOT coalesce(
  isfinite(nls_mean) AND nls_mean>0 AND isfinite(exp_integrated) AND exp_integrated>0
  AND isfinite(y/exp_integrated) AND y/exp_integrated>0,FALSE))>0
THEN error('Table 5.7 requires finite positive NLS and exponential predictions')
ELSE COUNT(*) END AS valid_means FROM active
"""
# Weights are fixed once from the first unweighted fit; diagnostics use scaled responses.
sql """
SELECT *,sqrt(nls_mean) AS wnls_scale,nls_mean AS fgnls_scale,
  1/nls_mean AS wnls_weight,1/(nls_mean*nls_mean) AS fgnls_weight,
  y/sqrt(nls_mean) AS wnls_y,y/nls_mean AS fgnls_y
FROM active ORDER BY source_row
""" into weighted_design
sql """
SELECT CASE WHEN COUNT(*) FILTER (WHERE NOT coalesce(
  isfinite(wnls_scale) AND wnls_scale>0 AND isfinite(fgnls_scale) AND fgnls_scale>0
  AND isfinite(wnls_weight) AND wnls_weight>0 AND isfinite(fgnls_weight) AND fgnls_weight>0
  AND isfinite(wnls_y) AND wnls_y>0 AND isfinite(fgnls_y) AND fgnls_y>0
  AND isfinite(exp(-1)/wnls_scale) AND exp(-1)/wnls_scale>0
  AND isfinite(exp(-1)/fgnls_scale) AND exp(-1)/fgnls_scale>0,FALSE))>0
THEN error('Table 5.7 requires finite positive fixed weights, scales and transformed starts/responses')
ELSE COUNT(*) END AS valid_weights FROM active
"""
nl wnls_y = exp(-a-b*x)/wnls_scale, params(a b) start(1 0)
estimates store wnls_working
nl wnls_y = exp(-a-b*x)/wnls_scale, params(a b) start(1 0) robust
estimates store wnls_robust
predict wnls_scaled_mean, mean
nl fgnls_y = exp(-a-b*x)/fgnls_scale, params(a b) start(1 0)
estimates store fgnls_working
nl fgnls_y = exp(-a-b*x)/fgnls_scale, params(a b) start(1 0) robust
estimates store fgnls_robust
predict fgnls_scaled_mean, mean
sql """
SELECT source_row,y,x,ols_mean,exp_integrated,y/exp_integrated AS exp_mean,nls_mean,
  wnls_scale,fgnls_scale,wnls_weight,fgnls_weight,
  wnls_scale*wnls_scaled_mean AS wnls_mean,fgnls_scale*fgnls_scaled_mean AS fgnls_mean
FROM active ORDER BY source_row
""" into table57_predictions
sql """
SELECT CASE WHEN COUNT(*) FILTER (WHERE NOT coalesce(
  isfinite(ols_mean) AND isfinite(exp_mean) AND exp_mean>0 AND isfinite(nls_mean) AND nls_mean>0
  AND isfinite(wnls_mean) AND wnls_mean>0 AND isfinite(fgnls_mean) AND fgnls_mean>0,FALSE))>0
THEN error('Table 5.7 requires finite representable original-scale projections')
ELSE COUNT(*) END AS valid_projections FROM active
"""
sql """
WITH diagnostics AS (
  SELECT var_pop(y)*count(*) AS tss,
    sum(-ln(exp_mean)-y/exp_mean) AS exp_ll,
    sum(-ln(nls_mean)-y/nls_mean) AS nls_ll,
    sum(-ln(wnls_mean)-y/wnls_mean) AS wnls_ll,
    sum(-ln(fgnls_mean)-y/fgnls_mean) AS fgnls_ll,
    sum((y-ols_mean)*(y-ols_mean)) AS ols_rss,
    sum((y-exp_mean)*(y-exp_mean)) AS exp_rss,
    sum((y-nls_mean)*(y-nls_mean)) AS nls_rss,
    sum((y-wnls_mean)*(y-wnls_mean)) AS wnls_rss,
    sum((y-fgnls_mean)*(y-fgnls_mean)) AS fgnls_rss
  FROM active
)
SELECT CASE WHEN NOT coalesce(isfinite(tss) AND tss>0
  AND isfinite(exp_ll) AND isfinite(nls_ll) AND isfinite(wnls_ll) AND isfinite(fgnls_ll)
  AND isfinite(ols_rss) AND isfinite(exp_rss) AND isfinite(nls_rss)
  AND isfinite(wnls_rss) AND isfinite(fgnls_rss)
  AND isfinite(ols_rss/tss) AND isfinite(exp_rss/tss) AND isfinite(nls_rss/tss)
  AND isfinite(wnls_rss/tss) AND isfinite(fgnls_rss/tss),FALSE)
THEN error('Table 5.7 requires finite original-scale diagnostics') ELSE 1 END AS valid_diagnostics
FROM diagnostics
"""
sql """
SELECT COUNT(*) AS observations,
  sum(-ln(exp_mean)-y/exp_mean) AS exp_density_log_likelihood,
  sum(-ln(nls_mean)-y/nls_mean) AS nls_density_log_likelihood,
  sum(-ln(wnls_mean)-y/wnls_mean) AS wnls_density_log_likelihood,
  sum(-ln(fgnls_mean)-y/fgnls_mean) AS fgnls_density_log_likelihood,
  1-sum((y-ols_mean)*(y-ols_mean))/(var_pop(y)*count(*)) AS ols_level_r_squared,
  1-sum((y-exp_mean)*(y-exp_mean))/(var_pop(y)*count(*)) AS exp_level_r_squared,
  1-sum((y-nls_mean)*(y-nls_mean))/(var_pop(y)*count(*)) AS nls_level_r_squared,
  1-sum((y-wnls_mean)*(y-wnls_mean))/(var_pop(y)*count(*)) AS wnls_level_r_squared,
  1-sum((y-fgnls_mean)*(y-fgnls_mean))/(var_pop(y)*count(*)) AS fgnls_level_r_squared,
  'Density evaluations assume the exponential law; level R-squared is descriptive, not a likelihood or cross-family test' AS interpretation
FROM active
"""
estimates table ols_working exp_working nls_working wnls_working fgnls_working
estimates table ols_robust exp_robust nls_robust wnls_robust fgnls_robust
export table57_predictions.csv
