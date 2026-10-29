# WIP: log-patents pooled, within-FE and RE-GLS linear panels.
use patents.csv
isid id year
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(DISTINCT id)<2 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR pat IS NULL OR logr IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR NOT isfinite(pat) OR NOT isfinite(logr) OR id!=floor(id) OR id<1 OR id>346 OR year!=floor(year) OR year<1975 OR year>1979 OR pat<0 OR pat!=floor(pat) OR pat>1000 OR ABS(logr)>20)>0 THEN error('invalid log-patents panel data') ELSE COUNT(*) END AS observations,COUNT(DISTINCT id) AS firms FROM active"""
sql """SELECT CASE WHEN COUNT(*)=0 OR MIN(n)!=5 OR MAX(n)!=5 THEN error('each firm requires all five years 1975–79') ELSE COUNT(*) END AS firms FROM (SELECT id,COUNT(*) AS n FROM active GROUP BY id)"""
# Reserve the name before SQL so an existing column cannot silently replace the outcome.
generate lpat = 0
sql """SELECT * EXCLUDE(lpat),LN(CASE WHEN pat=0 THEN 0.5 ELSE pat END) AS lpat FROM active""" into linear_panel_data
summarize pat logr lpat
regress lpat logr, cluster(id)
lincom logr
test logr = 0
predict pooled_xb, xb
predict pooled_residual, residuals
xtreg lpat logr, fe id(id) time(year)
lincom logr
test logr = 0
predict fe_xb, xb
predict fe_xbu, xbu
predict fe_u, u
predict fe_residual, residuals
xtreg lpat logr, re id(id) time(year)
predict re_xb, xb
predict re_residual, residuals
summarize pooled_xb pooled_residual fe_xbu fe_residual re_xb re_residual
sort id year
export linear_panel_predictions.csv
