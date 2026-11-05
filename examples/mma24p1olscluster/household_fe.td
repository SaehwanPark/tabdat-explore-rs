# Commune FE for cross-sectional households: source_row is a unique key, not time.
use vietnam.csv
isid source_row
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(*) FILTER (WHERE typeof(source_row)!='BIGINT' OR source_row IS NULL OR source_row<1 OR typeof(commune)!='BIGINT' OR commune IS NULL OR commune<1 OR age IS NULL OR NOT isfinite(age) OR sex IS NULL OR NOT isfinite(sex) OR hhsize IS NULL OR NOT isfinite(hhsize) OR farm IS NULL OR NOT isfinite(farm) OR educ IS NULL OR NOT isfinite(educ) OR lnhhexp IS NULL OR NOT isfinite(lnhhexp) OR (lnexp12m IS NOT NULL AND NOT isfinite(lnexp12m)))>0 THEN error('invalid household source or commune key') ELSE COUNT(*) END AS source_rows, COUNT(*) FILTER (WHERE lnexp12m IS NULL) AS missing_outcomes, COUNT(DISTINCT commune) AS source_communes FROM active"""
sql """SELECT * FROM active WHERE lnexp12m IS NOT NULL ORDER BY source_row""" into household_fe_cohort
sql """SELECT CASE WHEN COUNT(DISTINCT commune)<2 THEN error('commune FE workflow requires at least two complete-case communes') ELSE COUNT(DISTINCT commune) END AS complete_case_communes FROM active"""
xtreg lnexp12m lnhhexp age sex hhsize farm educ, fe id(commune) time(source_row)
lincom _cons
lincom lnhhexp
test _cons lnhhexp
predict fe_xb, xb
predict fe_xbu, xbu
predict fe_u, u
predict fe_residual, residuals
summarize fe_xb fe_xbu fe_u fe_residual
sql """SELECT 'Commune FE on cross-sectional households; source_row is a unique key, not time. Common intercept uses observation-weighted zero-sum unit effects. Not survey-design or causal inference.' AS scope"""
export household_fe_predictions.csv
