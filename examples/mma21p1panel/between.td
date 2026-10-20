# Table21.2 Between classical portion, plus independent-person HC1 comparison.
# Select joint-finite model cases before computing either mean; equal person weights.
use MOM.csv
isid id year
sql """SELECT CASE WHEN COUNT(*) = 0 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR id != floor(id) OR year != floor(year) OR id < 1 OR id > 532 OR year < 1979 OR year > 1988) > 0 THEN error('MOM requires nonempty exact integer id 1..532 and year 1979..1988') ELSE COUNT(*) END AS observations, COUNT(DISTINCT id) AS people, MIN(year) AS first_year, MAX(year) AS last_year FROM active"""
sql """SELECT id, COUNT(*) AS model_rows, MIN(year) AS first_year, MAX(year) AS last_year, AVG(lnhr) AS mean_lnhr, AVG(lnwg) AS mean_lnwg FROM active WHERE isfinite(lnhr) AND isfinite(lnwg) GROUP BY id ORDER BY id""" into between_means
isid id
regress mean_lnhr mean_lnwg
estimates store between_classical
test mean_lnwg = 0
lincom mean_lnwg
regress mean_lnhr mean_lnwg, robust
estimates store between_hc1
test mean_lnwg = 0
lincom mean_lnwg
estimates table between_classical between_hc1
predict between_fitted, xb
predict between_residual, residuals
summarize mean_lnhr mean_lnwg between_fitted between_residual
export panel_between_predictions.csv
