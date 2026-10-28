# Table 21.2 pooled OLS: whole-person pairs bootstrap, not Stata RNG parity.
use MOM.csv
isid id year
isid source_row
sql """SELECT CASE WHEN COUNT(*) = 0 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR id != floor(id) OR year != floor(year) OR id < 1 OR id > 532 OR year < 1979 OR year > 1988) > 0 THEN error('MOM requires nonempty exact integer id 1..532 and year 1979..1988') ELSE COUNT(*) END AS observations, COUNT(DISTINCT id) AS people, MIN(year) AS first_year, MAX(year) AS last_year FROM active"""
sql """SELECT COUNT(*) AS raw_rows, COUNT(*) FILTER (WHERE coalesce(isfinite(lnhr) AND isfinite(lnwg), FALSE)) AS retained_rows, COUNT(*) FILTER (WHERE NOT coalesce(isfinite(lnhr) AND isfinite(lnwg), FALSE)) AS excluded_rows FROM active"""
sql """SELECT source_row,id,year,lnhr,lnwg FROM active WHERE isfinite(lnhr) AND isfinite(lnwg) ORDER BY source_row""" into pooled_bootstrap_frame
regress lnhr lnwg
bootstrap lnhr lnwg, cluster(id) reps(500) seed(10001)
sql """SELECT 'Whole-person pairs; independent people assumed; 500 draws have Monte Carlo error; RNG differs from Stata' AS bootstrap_interpretation"""
predict pooled_fitted, xb
predict pooled_residual, residuals
summarize pooled_fitted pooled_residual
export panel_pooled_bootstrap_predictions.csv
