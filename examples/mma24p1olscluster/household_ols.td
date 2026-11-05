# MMA §24.7 Table 24.4 OLS columns 1–4; not a complex-survey analysis.
use vietnam.csv
isid source_row
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(*) FILTER (WHERE typeof(source_row)!='BIGINT' OR source_row IS NULL OR source_row<1 OR typeof(commune)!='BIGINT' OR commune IS NULL OR commune<1 OR age IS NULL OR NOT isfinite(age) OR sex IS NULL OR NOT isfinite(sex) OR hhsize IS NULL OR NOT isfinite(hhsize) OR farm IS NULL OR NOT isfinite(farm) OR educ IS NULL OR NOT isfinite(educ) OR lnhhexp IS NULL OR NOT isfinite(lnhhexp) OR (lnexp12m IS NOT NULL AND NOT isfinite(lnexp12m)))>0 THEN error('invalid household source or commune key') ELSE COUNT(*) END AS source_rows, COUNT(*) FILTER (WHERE lnexp12m IS NULL) AS missing_outcomes, COUNT(DISTINCT commune) AS source_communes FROM active"""
sql """SELECT * FROM active WHERE lnexp12m IS NOT NULL ORDER BY source_row""" into household_cohort
regress lnexp12m lnhhexp age sex hhsize farm educ
estimates store household_iid
regress lnexp12m lnhhexp age sex hhsize farm educ, robust
estimates store household_hc1
regress lnexp12m lnhhexp age sex hhsize farm educ, cluster(commune)
estimates store household_cluster
estimates table household_iid household_hc1 household_cluster
test lnhhexp = 0
lincom lnhhexp
predict household_fitted, xb
predict household_residual, residuals
summarize household_fitted household_residual
sql """SELECT 'IID, HC1, and commune CR1 are alternative covariance assumptions for the same OLS fit, not weighted survey design or causal estimates. The retained model is commune-clustered OLS.' AS scope"""
export household_ols_predictions.csv
