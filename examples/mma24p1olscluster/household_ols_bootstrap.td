# Chapter24 household pooled OLS: 500 whole-commune draws, not the Stata RNG.
use vietnam.csv
isid source_row
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(*) FILTER (WHERE typeof(source_row)!='BIGINT' OR source_row IS NULL OR source_row<1 OR typeof(commune)!='BIGINT' OR commune IS NULL OR commune<1)>0 THEN error('invalid household source or commune key') ELSE COUNT(*) END AS source_rows, COUNT(DISTINCT commune) AS source_communes FROM active"""
sql """SELECT COUNT(*) AS raw_rows, COUNT(*) FILTER (WHERE coalesce(isfinite(lnexp12m) AND isfinite(lnhhexp) AND isfinite(age) AND isfinite(sex) AND isfinite(hhsize) AND isfinite(farm) AND isfinite(educ), FALSE)) AS retained_rows, COUNT(*) FILTER (WHERE NOT coalesce(isfinite(lnexp12m) AND isfinite(lnhhexp) AND isfinite(age) AND isfinite(sex) AND isfinite(hhsize) AND isfinite(farm) AND isfinite(educ), FALSE)) AS excluded_rows FROM active"""
sql """SELECT * FROM active WHERE coalesce(isfinite(lnexp12m) AND isfinite(lnhhexp) AND isfinite(age) AND isfinite(sex) AND isfinite(hhsize) AND isfinite(farm) AND isfinite(educ), FALSE) ORDER BY source_row""" into household_bootstrap_cohort
regress lnexp12m lnhhexp age sex hhsize farm educ
bootstrap lnexp12m lnhhexp age sex hhsize farm educ, cluster(commune) reps(500) seed(10001)
predict pooled_fitted, xb
predict pooled_residual, residuals
summarize pooled_fitted pooled_residual
sql """SELECT 'Whole-commune pairs; independent communes assumed; B=500 has Monte Carlo error; native RNG differs from Stata. Unweighted, not survey-design or causal inference.' AS bootstrap_scope"""
export household_ols_bootstrap_predictions.csv
