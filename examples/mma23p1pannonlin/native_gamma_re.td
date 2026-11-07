# Patents1975–79: native shared mean-one Gamma firm effects, not row-level NB.
use patents.csv
isid id year
sql """SELECT CASE WHEN COUNT(*)<10 OR COUNT(*)>1730 OR COUNT(*) FILTER (WHERE typeof(id)!='BIGINT' OR id IS NULL OR id<1 OR id>346 OR typeof(year)!='BIGINT' OR year IS NULL OR year<1975 OR year>1979 OR pat IS NULL OR NOT isfinite(pat) OR pat<0 OR pat>1000 OR pat!=floor(pat) OR logr IS NULL OR NOT isfinite(logr) OR abs(logr)>20 OR cusip IS NULL OR NOT isfinite(cusip) OR scisect IS NULL OR NOT isfinite(scisect) OR logk IS NULL OR NOT isfinite(logk) OR sumpat IS NULL OR NOT isfinite(sumpat) OR (ardssic IS NOT NULL AND NOT isfinite(ardssic)))>0 THEN error('requires canonical numeric patents source, exact positive keys, finite integer counts0..1000 and finite LOGR within20') ELSE COUNT(*) END AS source_rows,COUNT(DISTINCT id) AS source_firms,COUNT(*) FILTER (WHERE ardssic IS NULL) AS missing_nonmodel_ardssic FROM active"""
sql """SELECT CASE WHEN EXISTS(SELECT id FROM active GROUP BY id HAVING COUNT(*)!=5) THEN error('every loaded firm must retain five unique years1975..1979') ELSE COUNT(DISTINCT id) END AS five_year_firms FROM active"""
sql """SELECT *,SUM(pat ORDER BY year) OVER (PARTITION BY id) AS selected_firm_total,TRUE AS included_in_native_re FROM active ORDER BY id,year""" into native_gamma_re_cohort
xtpoisson pat logr, re id(id)
predict native_re_mean, n
predict native_re_xb, xb
predict native_re_residual, residuals
summarize native_re_mean native_re_residual
sql """SELECT FIRST('Native shared mean-one Gamma firm effects; all-zero firms retained, no metadata missingness filter. Full joint observed model covariance and normal inference; lnalpha=0 means alpha=1, not no heterogeneity. Alpha=0 uses the separate boundary-mixture LR. Population marginal means, not posterior firm effects. Not clustered/survey/causal inference; later native predict still uses this owned RE fit. Script is ordered, not whole-transactional.') AS scope FROM active"""
export native_gamma_re_predictions.csv
