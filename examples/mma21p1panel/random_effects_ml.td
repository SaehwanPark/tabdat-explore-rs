# Table21.2 RE-ML coefficient/likelihood portion: balanced Gaussian interior ML.
use MOM.csv
isid id year
sql """SELECT CASE WHEN COUNT(*) = 0 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR id != floor(id) OR year != floor(year) OR id < 1 OR id > 532 OR year < 1979 OR year > 1988) > 0 THEN error('MOM requires nonempty exact integer id 1..532 and year 1979..1988') ELSE COUNT(*) END AS observations, COUNT(DISTINCT id) AS people, MIN(year) AS first_year, MAX(year) AS last_year FROM active"""
xtreg lnhr lnwg, mle id(id) time(year)
predict ml_xb, xb
predict ml_residual, residuals
summarize ml_xb ml_residual
export panel_re_ml_predictions.csv
