# Table21.2 POLS author-program portion, not a within-FE or RE fit.
use MOM.csv
isid id year
sql """SELECT CASE WHEN COUNT(*) = 0 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR id != floor(id) OR year != floor(year) OR id < 1 OR id > 532 OR year < 1979 OR year > 1988) > 0 THEN error('MOM requires nonempty exact integer id 1..532 and year 1979..1988') ELSE COUNT(*) END AS observations, COUNT(DISTINCT id) AS people, MIN(year) AS first_year, MAX(year) AS last_year FROM active"""
regress lnhr lnwg
estimates store pols_classical
test lnwg = 0
lincom lnwg
regress lnhr lnwg, robust
estimates store pols_hc1
test lnwg = 0
lincom lnwg
regress lnhr lnwg, cluster(id)
estimates store pols_cluster
test lnwg = 0
lincom lnwg
estimates table pols_classical pols_hc1 pols_cluster
predict pols_fitted, xb
predict pols_residual, residuals
summarize pols_fitted pols_residual
export panel_pooled_predictions.csv
