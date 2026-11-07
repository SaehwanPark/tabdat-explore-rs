# Household Table24.4 native comparison; source_row is a key, not calendar time.
use vietnam.csv
isid source_row
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(*)>40000 OR COUNT(*) FILTER (WHERE typeof(source_row)!='BIGINT' OR source_row IS NULL OR source_row<1 OR typeof(commune)!='BIGINT' OR commune IS NULL OR commune<1 OR age IS NULL OR NOT isfinite(age) OR sex IS NULL OR NOT isfinite(sex) OR hhsize IS NULL OR NOT isfinite(hhsize) OR farm IS NULL OR NOT isfinite(farm) OR educ IS NULL OR NOT isfinite(educ) OR lnhhexp IS NULL OR NOT isfinite(lnhhexp) OR (lnexp12m IS NOT NULL AND NOT isfinite(lnexp12m)))>0 THEN error('invalid or oversized household source or commune key') ELSE COUNT(*) END AS source_rows, COUNT(*) FILTER (WHERE lnexp12m IS NULL) AS missing_outcomes, COUNT(DISTINCT commune) AS source_communes FROM active"""
sql """SELECT * FROM active WHERE lnexp12m IS NOT NULL ORDER BY source_row""" into household_mixed_cohort
sql """SELECT CASE WHEN COUNT(DISTINCT commune)<8 THEN error('mixed household workflow requires at least eight complete-case communes') ELSE COUNT(DISTINCT commune) END AS complete_case_communes FROM active"""
xtgee lnexp12m lnhhexp age sex hhsize farm educ, family(gaussian) link(identity) corr(exchangeable) id(commune)
estimates store pa, covariance(working)
xtreg lnexp12m lnhhexp age sex hhsize farm educ, fe id(commune) time(source_row)
estimates store fe
xtreg lnexp12m lnhhexp age sex hhsize farm educ, re id(commune) time(source_row) vcmethod(equal_group_between)
estimates store refgls
xtreg lnexp12m lnhhexp age sex hhsize farm educ, mle id(commune) time(source_row)
estimates store remle
estimates table pa fe refgls remle
predict ml_xb, xb
predict ml_residual, residuals
summarize ml_xb ml_residual
sql """SELECT 'Native PA/FE/GLS/ML comparison: explicit GEE working covariance, classical normalized FE t, RE normal z, separate sample/covariance conventions. R-squared is unavailable in the common OLS-labelled row; only ML supplies marginal likelihood. No silent refits, arbitrary LR comparison, survey or causal inference. Retains RE-ML; source_row is a key, not time. Printed historical ML SE discrepancies remain unresolved.' AS scope"""
export household_mixed_report_predictions.csv
