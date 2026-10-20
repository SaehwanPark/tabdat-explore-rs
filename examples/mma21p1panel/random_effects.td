# Table21.2 RE-GLS classical portion: balanced one-way Swamy-Arora.
use MOM.csv
isid id year
sql """SELECT CASE WHEN COUNT(*) = 0 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR id != floor(id) OR year != floor(year) OR id < 1 OR id > 532 OR year < 1979 OR year > 1988) > 0 THEN error('MOM requires nonempty exact integer id 1..532 and year 1979..1988') ELSE COUNT(*) END AS observations, COUNT(DISTINCT id) AS people, MIN(year) AS first_year, MAX(year) AS last_year FROM active"""
xtreg lnhr lnwg, re id(id) time(year)
predict re_xb, xb
predict re_residual, residuals
summarize re_xb re_residual
export panel_re_gls_predictions.csv
