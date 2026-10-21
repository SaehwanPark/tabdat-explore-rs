# Manual-program block4: within-transformed OLS, resampling whole people.
use MOM.csv
isid id year
sql """SELECT CASE WHEN COUNT(*) = 0 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR id != floor(id) OR year != floor(year) OR id < 1 OR id > 532 OR year < 1979 OR year > 1988) > 0 THEN error('MOM requires nonempty exact integer id 1..532 and year 1979..1988') ELSE COUNT(*) END AS observations, COUNT(DISTINCT id) AS people, MIN(year) AS first_year, MAX(year) AS last_year FROM active"""
isid source_row
sql """SELECT source_row, id, year, lnhr, lnwg, lnhr - AVG(lnhr ORDER BY year) OVER (PARTITION BY id) AS within_lnhr, lnwg - AVG(lnwg ORDER BY year) OVER (PARTITION BY id) AS within_lnwg FROM active WHERE isfinite(lnhr) AND isfinite(lnwg) ORDER BY source_row""" into within_bootstrap_frame
regress within_lnhr within_lnwg
bootstrap within_lnhr within_lnwg, cluster(id) reps(500) seed(10001)
predict within_ols_fitted, xb
predict within_ols_residual, residuals
summarize within_ols_fitted within_ols_residual
export panel_within_bootstrap_predictions.csv
