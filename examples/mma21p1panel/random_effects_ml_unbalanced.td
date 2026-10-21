# Unbalanced MOM sensitivity analysis; changed sample, not a published table row.
use MOM.csv
isid id year
sql """SELECT CASE WHEN COUNT(*) = 0 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR id != floor(id) OR year != floor(year) OR id < 1 OR id > 532 OR year < 1979 OR year > 1988) > 0 THEN error('MOM requires nonempty exact integer id 1..532 and year 1979..1988') ELSE COUNT(*) END AS observations, COUNT(DISTINCT id) AS people, MIN(year) AS first_year, MAX(year) AS last_year FROM active"""
sql """SELECT * FROM active WHERE NOT ((id % 3 = 0 AND year IN (1980,1985)) OR (id % 7 = 0 AND year BETWEEN 1981 AND 1983)) ORDER BY source_row""" into sensitivity_ml
xtreg lnhr lnwg, mle id(id) time(year)
predict ub_xb, xb
predict ub_residual, residuals
summarize ub_xb ub_residual
export panel_re_ml_unbalanced_predictions.csv
