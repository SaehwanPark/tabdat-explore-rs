# Fixed weights use the first unweighted fit; transformed diagnostics are not level diagnostics.
use nls.csv
isid source_row
sql """
SELECT CASE WHEN COUNT(*)<=2 OR COUNT(*)>10000
  OR MIN(source_row)<>1 OR MAX(source_row)<>COUNT(*)
  OR COUNT(*) FILTER (WHERE NOT coalesce(
    isfinite(source_row) AND source_row=floor(source_row) AND source_row>0
    AND isfinite(x) AND isfinite(y) AND y>0,FALSE))>0
THEN error('weighted NLS requires 3..10000 complete finite rows, positive y and unique dense source_row1..N')
ELSE COUNT(*) END AS observations FROM active
"""
nl y = exp(-a-b*x), params(a b) start(1 0)
predict preliminary_mean, mean
sql """
SELECT CASE WHEN COUNT(*) FILTER (WHERE NOT coalesce(
  isfinite(preliminary_mean) AND preliminary_mean>0,FALSE))>0
THEN error('weighted NLS requires positive finite preliminary means')
ELSE COUNT(*) END AS observations FROM active
"""
sql """
SELECT *,sqrt(preliminary_mean) AS wnls_scale,preliminary_mean AS fgnls_scale,
  1/preliminary_mean AS wnls_weight,1/(preliminary_mean*preliminary_mean) AS fgnls_weight,
  y/sqrt(preliminary_mean) AS wnls_y,y/preliminary_mean AS fgnls_y
FROM active ORDER BY source_row
""" into weighted_design
sql """
SELECT CASE WHEN COUNT(*) FILTER (WHERE NOT coalesce(
  isfinite(wnls_scale) AND wnls_scale>0 AND isfinite(fgnls_scale) AND fgnls_scale>0
  AND isfinite(wnls_weight) AND wnls_weight>0 AND isfinite(fgnls_weight) AND fgnls_weight>0
  AND isfinite(wnls_y) AND isfinite(fgnls_y)
  AND isfinite(exp(-1)/wnls_scale) AND exp(-1)/wnls_scale>0
  AND isfinite(exp(-1)/fgnls_scale) AND exp(-1)/fgnls_scale>0,FALSE))>0
THEN error('weighted NLS requires positive finite weights, scales and transformed starting means and finite responses')
ELSE COUNT(*) END AS observations FROM active
"""
nl wnls_y = exp(-a-b*x)/wnls_scale, params(a b) start(1 0)
predict wnls_mean_classical, mean
predict wnls_residual_classical, residuals
nl wnls_y = exp(-a-b*x)/wnls_scale, params(a b) start(1 0) robust
predict wnls_mean_hc1, mean
predict wnls_residual_hc1, residuals
nl fgnls_y = exp(-a-b*x)/fgnls_scale, params(a b) start(1 0)
predict fgnls_mean_classical, mean
predict fgnls_residual_classical, residuals
nl fgnls_y = exp(-a-b*x)/fgnls_scale, params(a b) start(1 0) robust
predict fgnls_mean_hc1, mean
predict fgnls_residual_hc1, residuals
sql """
SELECT *,wnls_scale*wnls_mean_hc1 AS wnls_level_mean,
  y-wnls_scale*wnls_mean_hc1 AS wnls_level_residual,
  1/(wnls_scale*wnls_mean_hc1) AS wnls_rate,
  fgnls_scale*fgnls_mean_hc1 AS fgnls_level_mean,
  y-fgnls_scale*fgnls_mean_hc1 AS fgnls_level_residual,
  1/(fgnls_scale*fgnls_mean_hc1) AS fgnls_rate
FROM active ORDER BY source_row
""" into weighted_predictions
sql """
SELECT CASE WHEN COUNT(*) FILTER (WHERE NOT coalesce(
  isfinite(wnls_level_mean) AND wnls_level_mean>0 AND isfinite(wnls_level_residual) AND isfinite(wnls_rate) AND wnls_rate>0
  AND isfinite(fgnls_level_mean) AND fgnls_level_mean>0 AND isfinite(fgnls_level_residual) AND isfinite(fgnls_rate) AND fgnls_rate>0,FALSE))>0
THEN error('weighted NLS requires representable original-level predictions and rates')
ELSE COUNT(*) END AS observations FROM active
"""
sql """
SELECT COUNT(*) AS observations,
  SUM(wnls_residual_hc1*wnls_residual_hc1) AS wnls_scaled_rss,
  SUM(wnls_level_residual*wnls_level_residual) AS wnls_level_rss,
  1-SUM(wnls_level_residual*wnls_level_residual)/NULLIF(SUM((y-mean_y)*(y-mean_y)),0) AS wnls_level_r_squared,
  SUM(fgnls_residual_hc1*fgnls_residual_hc1) AS fgnls_scaled_rss,
  SUM(fgnls_level_residual*fgnls_level_residual) AS fgnls_level_rss,
  1-SUM(fgnls_level_residual*fgnls_level_residual)/NULLIF(SUM((y-mean_y)*(y-mean_y)),0) AS fgnls_level_r_squared,
  'fixed first-fit weights; scaled and original-level diagnostics are distinct' AS scope
FROM (SELECT *,AVG(y) OVER () AS mean_y FROM active)
"""
export weighted_predictions.csv
