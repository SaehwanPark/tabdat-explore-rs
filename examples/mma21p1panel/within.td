# Table21.2 Within slope/conditional-prediction portion, not the full column.
use MOM.csv
isid id year
sql """SELECT CASE WHEN COUNT(*) = 0 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR id != floor(id) OR year != floor(year) OR id < 1 OR id > 532 OR year < 1979 OR year > 1988) > 0 THEN error('MOM requires nonempty exact integer id 1..532 and year 1979..1988') ELSE COUNT(*) END AS observations, COUNT(DISTINCT id) AS people, MIN(year) AS first_year, MAX(year) AS last_year FROM active"""
xtreg lnhr lnwg, fe id(id) time(year)
lincom lnwg
test lnwg = 0
predict fe_classical_xb, xb
predict fe_classical_xbu, xbu
predict fe_classical_u, u
predict fe_classical_residual, residuals
summarize fe_classical_xbu fe_classical_u fe_classical_residual
xtreg lnhr lnwg, fe id(id) time(year) cluster(id)
lincom lnwg
test lnwg = 0
predict fe_cluster_xb, xb
predict fe_cluster_xbu, xbu
predict fe_cluster_u, u
predict fe_cluster_residual, residuals
summarize fe_cluster_xbu fe_cluster_u fe_cluster_residual
export panel_within_fe_predictions.csv
