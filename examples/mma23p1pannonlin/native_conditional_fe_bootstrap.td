# Native conditional Poisson FE whole-firm pairs bootstrap; retain original fit.
use patents.csv
isid id year
sql """SELECT CASE WHEN COUNT(*)<10 OR COUNT(*)>1730 OR COUNT(*) FILTER (WHERE typeof(id)!='BIGINT' OR id IS NULL OR id<1 OR id>346 OR typeof(year)!='BIGINT' OR year IS NULL OR year<1975 OR year>1979 OR pat IS NULL OR NOT isfinite(pat) OR pat<0 OR pat>1000 OR pat!=floor(pat) OR logr IS NULL OR NOT isfinite(logr) OR abs(logr)>20 OR cusip IS NULL OR NOT isfinite(cusip) OR scisect IS NULL OR NOT isfinite(scisect) OR logk IS NULL OR NOT isfinite(logk) OR sumpat IS NULL OR NOT isfinite(sumpat) OR (ardssic IS NOT NULL AND (TRY_CAST(ardssic AS DOUBLE) IS NULL OR NOT isfinite(TRY_CAST(ardssic AS DOUBLE)))))>0 THEN error('requires canonical numeric patents source, exact positive keys, finite integer counts0..1000 and finite LOGR within20') ELSE COUNT(*) END AS source_rows,COUNT(DISTINCT id) AS source_firms,COUNT(*) FILTER (WHERE ardssic IS NULL) AS missing_nonmodel_ardssic FROM active"""
sql """SELECT CASE WHEN EXISTS(SELECT id FROM active GROUP BY id HAVING COUNT(*)!=5) THEN error('every loaded firm must retain five unique years1975..1979') ELSE COUNT(DISTINCT id) END AS five_year_firms FROM active"""
sql """WITH cohort AS (SELECT *,SUM(pat ORDER BY year) OVER (PARTITION BY id) AS selected_firm_total FROM active) SELECT *,selected_firm_total>0 AS included_in_native_fe FROM cohort ORDER BY id,year""" into native_conditional_fe_bootstrap_cohort
xtpoisson pat logr, fe id(id)
bootstrap pat logr, model(conditional_poisson) cluster(id) reps(500) seed(10001)
predict native_fe_mean, n
predict native_fe_xb, xb
predict native_fe_residual, residuals
summarize native_fe_mean native_fe_residual
sql """SELECT FIRST('Native conditional-Poisson FE whole-positive-firm pairs bootstrap, B500, seed10001, continuous non-Stata stream; repeated selections are distinct copies, failed refits abort without redraw. B-1 covariance and type7 percentile95 intervals, not classical SEs or historical Stata bootstrap parity. Original owned FE predictions remain. Native conditional Poisson firm FE given selected positive-firm count totals; one LOGR slope, no common intercept or year effects. Zero-total firms remain in source/export with NULL mean/residual and finite slope-only xb. Model-based inverse conditional information, normal95/z and chi-square LR/Wald; conditional multinomial likelihood includes factorial constants. Not cluster-robust, survey-design or causal inference. Native predictions retain fitted positive-group normalization, not a new normalization of current subsets; no population/new-firm forecast or unit-effect uncertainty. Script is ordered, not whole-transactional.') AS scope FROM active"""
export native_conditional_fe_bootstrap_predictions.csv
