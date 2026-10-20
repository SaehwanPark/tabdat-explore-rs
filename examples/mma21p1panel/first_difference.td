# Table21.2 first-difference author-program portion; retain its estimated intercept.
# Use raw rows for LAG: deleting model-missing rows first could bridge missing years.
use MOM.csv
isid id year
sql """SELECT CASE WHEN COUNT(*) = 0 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR id != floor(id) OR year != floor(year) OR id < 1 OR id > 532 OR year < 1979 OR year > 1988) > 0 THEN error('MOM requires nonempty exact integer id 1..532 and year 1979..1988') ELSE COUNT(*) END AS observations, COUNT(DISTINCT id) AS people, MIN(year) AS first_year, MAX(year) AS last_year FROM active"""
sql """WITH adjacent AS (SELECT source_row, id, year, lnhr, lnwg, LAG(source_row) OVER w AS previous_source_row, LAG(year) OVER w AS previous_year, lnhr - LAG(lnhr) OVER w AS dlnhr, lnwg - LAG(lnwg) OVER w AS dlnwg FROM active WINDOW w AS (PARTITION BY id ORDER BY year)) SELECT * FROM adjacent WHERE year = previous_year + 1 ORDER BY id, year""" into first_differences
isid id year
regress dlnhr dlnwg
estimates store fd_classical
test dlnwg = 0
lincom dlnwg
regress dlnhr dlnwg, robust
estimates store fd_hc1
test dlnwg = 0
lincom dlnwg
regress dlnhr dlnwg, cluster(id)
estimates store fd_cluster
test dlnwg = 0
lincom dlnwg
estimates table fd_classical fd_hc1 fd_cluster
predict fd_fitted, xb
predict fd_residual, residuals
summarize dlnhr dlnwg fd_fitted fd_residual
export panel_fd_predictions.csv
