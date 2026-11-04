# Manual program centered-OLS covariance comparison; not canonical FE uncertainty.
use MOM.csv
isid id year
isid source_row
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(*)>5320 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR source_row IS NULL OR lnhr IS NULL OR lnwg IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR NOT isfinite(source_row) OR NOT isfinite(lnhr) OR NOT isfinite(lnwg) OR id!=floor(id) OR id<1 OR id>532 OR year!=floor(year) OR year<1979 OR year>1988 OR source_row!=floor(source_row) OR source_row<1 OR source_row>5320 OR ABS(lnhr)>20 OR ABS(lnwg)>20)>0 THEN error('manual FE comparison requires finite bounded MOM model fields and exact keys') ELSE COUNT(*) END AS observations FROM active"""
sql """SELECT CASE WHEN COUNT(*)<3 OR COUNT(*)>532 OR MIN(t)!=10 OR MAX(t)!=10 THEN error('manual FE comparison requires complete ten-year people') ELSE COUNT(*) END AS people FROM (SELECT id,COUNT(*) AS t FROM active GROUP BY id)"""
sql """SELECT 'Canonical FE below uses absorbed N-G-1 DF. Subsequent auxiliary IID/row-HC1 fits are working comparisons, not canonical FE covariance. Only auxiliary slope inference is validated; centered-intercept t/p are floating-point artifacts and must not be interpreted.' AS inference_scope"""
xtreg lnhr lnwg, fe id(id) time(year)
sql """WITH means AS (SELECT *,AVG(lnhr ORDER BY year) OVER (PARTITION BY id) AS mean_lnhr,AVG(lnwg ORDER BY year) OVER (PARTITION BY id) AS mean_lnwg FROM active) SELECT source_row,id,year,lnhr,lnwg,kids,ageh,agesq,disab,mean_lnhr,mean_lnwg,lnhr-mean_lnhr AS within_lnhr,lnwg-mean_lnwg AS within_lnwg FROM means ORDER BY source_row""" into manual_fe_covariance_frame
regress within_lnhr within_lnwg
estimates store p2_fe_iid_working
lincom within_lnwg
regress within_lnhr within_lnwg, robust
estimates store p2_fe_hc1_working
lincom within_lnwg
regress within_lnhr within_lnwg, cluster(id)
estimates store p2_fe_person_cluster
estimates table p2_fe_iid_working p2_fe_hc1_working p2_fe_person_cluster
sql """SELECT 'Named columns are auxiliary OLS only; the centered intercept is not the canonical normalized level intercept. Person CR1 assumes independent people/adequate cluster asymptotics. Zero or nearly-zero score variance does not establish nominal interval coverage; no joint full-covariance test is attempted.' AS comparison_limits"""
predict aux_fitted, xb
predict aux_residual, residuals
summarize aux_fitted aux_residual
# Verify owned cluster inference still works after reports and prediction columns.
lincom within_lnwg
export panel_manual_fe_covariance_predictions.csv
