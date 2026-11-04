# Classical common FE intercept under observation-weighted zero-effect normalization.
use MOM.csv
isid id year
isid source_row
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(*)>5320 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR source_row IS NULL OR lnhr IS NULL OR lnwg IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR NOT isfinite(source_row) OR NOT isfinite(lnhr) OR NOT isfinite(lnwg) OR id!=floor(id) OR id<1 OR id>532 OR year!=floor(year) OR year<1979 OR year>1988 OR source_row!=floor(source_row) OR source_row<1 OR source_row>5320 OR ABS(lnhr)>20 OR ABS(lnwg)>20)>0 THEN error('classical FE source workflow requires finite bounded MOM model fields and exact keys') ELSE COUNT(*) END AS observations FROM active"""
sql """SELECT CASE WHEN COUNT(*)<3 OR COUNT(*)>532 OR MIN(t)!=10 OR MAX(t)!=10 THEN error('classical FE source workflow requires complete ten-year people') ELSE COUNT(*) END AS people FROM (SELECT id,COUNT(*) AS t FROM active GROUP BY id)"""
xtreg lnhr lnwg, fe id(id) time(year)
sql """SELECT * FROM active ORDER BY source_row""" into classical_fe_frame
lincom _cons
lincom _cons + 2.5*lnwg
test _cons lnwg
predict fe_xb, xb
predict fe_xbu, xbu
predict fe_u, u
predict fe_residual, residuals
summarize fe_xb fe_xbu fe_u fe_residual
export panel_classical_fe_intercept_predictions.csv
