# Table 21.2 annual first-difference OLS with whole-person pairs bootstrap.
# Construct lags on raw levels; filtering earlier could bridge missing years.
use MOM.csv
isid id year
isid source_row
sql """SELECT CASE WHEN COUNT(*) = 0 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR id != floor(id) OR year != floor(year) OR id < 1 OR id > 532 OR year < 1979 OR year > 1988) > 0 THEN error('MOM requires nonempty exact integer id 1..532 and year 1979..1988') ELSE COUNT(*) END AS observations, COUNT(DISTINCT id) AS people, MIN(year) AS first_year, MAX(year) AS last_year FROM active"""
sql """WITH adjacent AS (SELECT source_row,id,year,lnhr,lnwg,LAG(source_row) OVER w AS previous_source_row,LAG(year) OVER w AS previous_year,lnhr-LAG(lnhr) OVER w AS dlnhr,lnwg-LAG(lnwg) OVER w AS dlnwg FROM active WINDOW w AS (PARTITION BY id ORDER BY year)) SELECT * FROM adjacent WHERE year = previous_year + 1 ORDER BY id,year""" into fd_bootstrap_pairs
sql """SELECT COUNT(*) AS annual_pairs, COUNT(*) FILTER (WHERE coalesce(isfinite(dlnhr) AND isfinite(dlnwg), FALSE)) AS retained_pairs, COUNT(*) FILTER (WHERE NOT coalesce(isfinite(dlnhr) AND isfinite(dlnwg), FALSE)) AS excluded_pairs FROM active"""
sql """SELECT * FROM active WHERE isfinite(dlnhr) AND isfinite(dlnwg) ORDER BY id,year""" into fd_bootstrap_frame
regress dlnhr dlnwg
bootstrap dlnhr dlnwg, cluster(id) reps(500) seed(10001)
sql """SELECT 'Whole-person annual-pair blocks; independent people assumed; 500 draws have Monte Carlo error; RNG differs from Stata' AS bootstrap_interpretation"""
predict fd_fitted, xb
predict fd_residual, residuals
summarize fd_fitted fd_residual
export panel_fd_bootstrap_predictions.csv
