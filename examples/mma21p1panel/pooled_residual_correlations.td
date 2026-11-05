# Table21.3 pooled-OLS residual correlations; descriptive, not a formal test.
use MOM.csv
isid id year
isid source_row
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(*)>5320 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR source_row IS NULL OR lnhr IS NULL OR lnwg IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR NOT isfinite(source_row) OR NOT isfinite(lnhr) OR NOT isfinite(lnwg) OR id!=floor(id) OR id<1 OR id>532 OR year!=floor(year) OR year<1979 OR year>1988 OR source_row!=floor(source_row) OR source_row<1 OR source_row>5320 OR ABS(lnhr)>20 OR ABS(lnwg)>20)>0 THEN error('pooled residual correlations require finite bounded model fields and exact MOM keys') ELSE COUNT(*) END AS observations FROM active"""
sql """SELECT CASE WHEN COUNT(*)<3 OR COUNT(*)>532 OR MIN(t)!=10 OR MAX(t)!=10 THEN error('pooled residual correlations require 3..532 complete ten-year people') ELSE COUNT(*) END AS people FROM (SELECT id,COUNT(*) AS t FROM active GROUP BY id)"""
sql """SELECT 'Pooled OLS residual correlations compare people across year pairs; they are descriptive, not a test or covariance correction. All reported pairs use the same complete ten-year people. Every year requires positive finite cross-person residual variance.' AS correlation_scope"""
regress lnhr lnwg
predict pooled_fitted, xb
predict pooled_residual, residuals
sql """WITH years AS (SELECT year,COUNT(*) AS n,VAR_POP(pooled_residual) AS v FROM active GROUP BY year) SELECT CASE WHEN COUNT(*)!=10 OR COUNT(*) FILTER (WHERE n!=(SELECT COUNT(DISTINCT id) FROM active) OR v IS NULL OR NOT isfinite(v) OR v<=0)>0 THEN error('pooled residual correlations require positive finite cross-person residual variance in every year') ELSE COUNT(*) END AS validated_years FROM years"""
sql """WITH pairs AS (SELECT a.year AS year_a,b.year AS year_b,COUNT(*) AS n,CORR(a.pooled_residual,b.pooled_residual) AS r FROM active a JOIN active b ON a.id=b.id GROUP BY a.year,b.year) SELECT CASE WHEN COUNT(*)!=100 OR COUNT(*) FILTER (WHERE n!=(SELECT COUNT(DISTINCT id) FROM active) OR r IS NULL OR NOT isfinite(r) OR ABS(r)>1.000000000001)>0 THEN error('pooled residual correlations require 100 finite, identified year-pair correlations') ELSE COUNT(*) END AS validated_pairs FROM pairs"""
sql """SELECT * FROM active ORDER BY source_row""" into pooled_residual_rows
export panel_pooled_residual_rows.csv
sql """SELECT a.year AS year_a,b.year AS year_b,COUNT(*) AS people,CORR(a.pooled_residual,b.pooled_residual) AS correlation FROM active a JOIN active b ON a.id=b.id GROUP BY a.year,b.year ORDER BY a.year,b.year""" into pooled_residual_correlations
sql """SELECT year_a,year_b,people,correlation FROM active ORDER BY year_a,year_b"""
# OLS state and inference remain owned after prediction, export and SQL materialization.
lincom lnwg
export panel_pooled_residual_correlations.csv
