# Annual FD series/residual correlations; descriptive, not a formal test.
use MOM.csv
isid id year
isid source_row
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(*)>5320 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR source_row IS NULL OR lnhr IS NULL OR lnwg IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR NOT isfinite(source_row) OR NOT isfinite(lnhr) OR NOT isfinite(lnwg) OR id!=floor(id) OR id<1 OR id>532 OR year!=floor(year) OR year<1979 OR year>1988 OR source_row!=floor(source_row) OR source_row<1 OR source_row>5320 OR ABS(lnhr)>20 OR ABS(lnwg)>20)>0 THEN error('FD correlations require finite bounded model fields and exact MOM keys') ELSE COUNT(*) END AS observations FROM active"""
sql """SELECT CASE WHEN COUNT(*)<3 OR COUNT(*)>532 OR MIN(t)!=10 OR MAX(t)!=10 THEN error('FD correlations require 3..532 complete ten-year people') ELSE COUNT(*) END AS people FROM (SELECT id,COUNT(*) AS t FROM active GROUP BY id)"""
sql """SELECT 'Annual FD series and OLS residual correlations compare people across year pairs; they are descriptive, not a test or covariance correction. All pairs use the same complete ten-year people. Every differenced year requires positive finite cross-person variance in each series.' AS correlation_scope"""
sql """WITH adjacent AS (SELECT source_row,lnhr,lnwg,kids,ageh,agesq,disab,id,year,LAG(source_row) OVER w AS previous_source_row,LAG(year) OVER w AS previous_year,LAG(lnhr) OVER w AS previous_lnhr,LAG(lnwg) OVER w AS previous_lnwg,lnhr-LAG(lnhr) OVER w AS dlnhr,lnwg-LAG(lnwg) OVER w AS dlnwg FROM active WINDOW w AS (PARTITION BY id ORDER BY year)) SELECT * FROM adjacent WHERE year=previous_year+1 ORDER BY id,year""" into fd_pairs
isid id year
regress dlnhr dlnwg
predict fd_fitted, xb
predict fd_residual, residuals
sql """WITH years AS (SELECT year,COUNT(*) AS n,VAR_POP(dlnhr) AS h,VAR_POP(dlnwg) AS w,VAR_POP(fd_residual) AS e FROM active GROUP BY year) SELECT CASE WHEN COUNT(*)!=9 OR COUNT(*) FILTER (WHERE n!=(SELECT COUNT(DISTINCT id) FROM active) OR h IS NULL OR w IS NULL OR e IS NULL OR NOT isfinite(h) OR NOT isfinite(w) OR NOT isfinite(e) OR h<=0 OR w<=0 OR e<=0)>0 THEN error('FD correlations require positive finite cross-person variance in each series and year') ELSE COUNT(*) END AS validated_years FROM years"""
sql """WITH pairs AS (SELECT a.year AS year_a,b.year AS year_b,COUNT(*) AS n,CORR(a.dlnhr,b.dlnhr) AS h,CORR(a.dlnwg,b.dlnwg) AS w,CORR(a.fd_residual,b.fd_residual) AS e FROM active a JOIN active b ON a.id=b.id GROUP BY a.year,b.year) SELECT CASE WHEN COUNT(*)!=81 OR COUNT(*) FILTER (WHERE n!=(SELECT COUNT(DISTINCT id) FROM active) OR h IS NULL OR w IS NULL OR e IS NULL OR NOT isfinite(h) OR NOT isfinite(w) OR NOT isfinite(e) OR ABS(h)>1.000000000001 OR ABS(w)>1.000000000001 OR ABS(e)>1.000000000001)>0 THEN error('FD correlations require 243 finite, identified year-pair correlations') ELSE COUNT(*) END AS validated_pairs FROM pairs"""
sql """SELECT * FROM active ORDER BY id,year""" into fd_residual_rows
export panel_fd_residual_rows.csv
sql """WITH pairs AS (SELECT a.year AS year_a,b.year AS year_b,COUNT(*) AS people,CORR(a.dlnhr,b.dlnhr) AS h,CORR(a.dlnwg,b.dlnwg) AS w,CORR(a.fd_residual,b.fd_residual) AS e FROM active a JOIN active b ON a.id=b.id GROUP BY a.year,b.year) SELECT 'dlnhr' AS variable,year_a,year_b,people,h AS correlation FROM pairs UNION ALL SELECT 'dlnwg' AS variable,year_a,year_b,people,w AS correlation FROM pairs UNION ALL SELECT 'fd_residual' AS variable,year_a,year_b,people,e AS correlation FROM pairs ORDER BY variable,year_a,year_b""" into fd_residual_correlations
sql """SELECT variable,year_a,year_b,people,correlation FROM active ORDER BY variable,year_a,year_b"""
# The fitted annual-FD model remains owned after matrix materialization.
lincom dlnwg
export panel_fd_residual_correlations.csv
