# Patents1975–79: native balanced Poisson/log exchangeable population-averaged GEE.
use patents.csv
isid id year
sql """SELECT CASE WHEN COUNT(*)<10 OR COUNT(*)>1730 OR COUNT(*) FILTER (WHERE typeof(id)!='BIGINT' OR id IS NULL OR id<1 OR id>346 OR typeof(year)!='BIGINT' OR year IS NULL OR year<1975 OR year>1979 OR pat IS NULL OR NOT isfinite(pat) OR pat<0 OR pat>1000 OR pat!=floor(pat) OR logr IS NULL OR NOT isfinite(logr) OR abs(logr)>20 OR cusip IS NULL OR NOT isfinite(cusip) OR scisect IS NULL OR NOT isfinite(scisect) OR logk IS NULL OR NOT isfinite(logk) OR sumpat IS NULL OR NOT isfinite(sumpat) OR (ardssic IS NOT NULL AND (TRY_CAST(ardssic AS DOUBLE) IS NULL OR NOT isfinite(TRY_CAST(ardssic AS DOUBLE)))))>0 THEN error('requires canonical numeric patents source, exact positive keys, finite integer counts0..1000 and finite LOGR within20') ELSE COUNT(*) END AS source_rows,COUNT(DISTINCT id) AS source_firms,COUNT(*) FILTER (WHERE ardssic IS NULL) AS missing_nonmodel_ardssic FROM active"""
sql """SELECT CASE WHEN EXISTS(SELECT id FROM active GROUP BY id HAVING COUNT(*)!=5) THEN error('every loaded firm must retain five unique years1975..1979') ELSE COUNT(DISTINCT id) END AS five_year_firms FROM active"""
sql """SELECT *,SUM(pat ORDER BY year) OVER (PARTITION BY id) AS selected_firm_total,TRUE AS included_in_native_gee FROM active ORDER BY id,year""" into native_poisson_gee_cohort
xtgee pat logr, family(poisson) link(log) corr(exchangeable) id(id)
predict native_gee_mean, n
predict native_gee_xb, xb
predict native_gee_residual, residuals
summarize native_gee_mean native_gee_residual
sql """SELECT FIRST('Native balanced Poisson/log exchangeable population-averaged GEE; every source firm/year and zero-total firm retained, no metadata missingness filter. Legacy normalized Pearson correlation uses RSS/N only as correlation normalizer; working variance scale is fixed1. Working and G/(G-1) group-score sandwich covariance, normal95/z and chi-square slope-Wald inference. No N-K factor, G-1 t, joint likelihood/LR, posterior group effects, count probabilities, survey or causal claim. Later native predict uses this owned GEE fit, not the internal pooled start. Script is ordered, not whole-transactional.') AS scope FROM active"""
export native_poisson_gee_predictions.csv
