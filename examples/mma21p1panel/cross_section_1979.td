# P3-1979 cross-sectional intercept OLS; descriptive, not panel FE or causal inference.
use MOM.csv
isid id year
isid source_row
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(*)>5320 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR source_row IS NULL OR lnhr IS NULL OR lnwg IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR NOT isfinite(source_row) OR NOT isfinite(lnhr) OR NOT isfinite(lnwg) OR id!=floor(id) OR id<1 OR id>532 OR year!=floor(year) OR year<1979 OR year>1988 OR source_row!=floor(source_row) OR source_row<1 OR source_row>5320 OR ABS(lnhr)>20 OR ABS(lnwg)>20)>0 THEN error('1979 cross-section requires finite bounded model fields and exact MOM keys') ELSE COUNT(*) END AS observations FROM active"""
sql """SELECT CASE WHEN COUNT(*)<3 OR COUNT(*)>532 OR MIN(t)!=10 OR MAX(t)!=10 THEN error('1979 cross-section requires 3..532 complete ten-year people') ELSE COUNT(*) END AS people FROM (SELECT id,COUNT(*) AS t FROM active GROUP BY id)"""
sql """SELECT 'Cross-sectional intercept OLS uses 1979 observations only, with classical homoskedastic covariance and t/F inference; it is not panel FE or a causal claim.' AS regression_scope"""
sql """SELECT * FROM active WHERE year=1979 ORDER BY source_row""" into cross_section_1979
regress lnhr lnwg
predict cross_fitted, xb
predict cross_residual, residuals
sql """SELECT * FROM active ORDER BY source_row""" into cross_section_1979_predictions
# Preserve owned OLS model across SQL materialization.
test lnwg = 0
lincom lnwg
export panel_1979_cross_section_predictions.csv
