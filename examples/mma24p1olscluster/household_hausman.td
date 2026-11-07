# Classical joint FE/RE comparison; source_row orders households, not time.
use vietnam.csv
isid source_row
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(*)>40000 OR COUNT(*) FILTER (WHERE typeof(source_row)!='BIGINT' OR source_row IS NULL OR source_row<1 OR typeof(commune)!='BIGINT' OR commune IS NULL OR commune<1 OR age IS NULL OR NOT isfinite(age) OR sex IS NULL OR NOT isfinite(sex) OR hhsize IS NULL OR NOT isfinite(hhsize) OR farm IS NULL OR NOT isfinite(farm) OR educ IS NULL OR NOT isfinite(educ) OR lnhhexp IS NULL OR NOT isfinite(lnhhexp) OR (lnexp12m IS NOT NULL AND NOT isfinite(lnexp12m)))>0 THEN error('invalid or oversized household source or commune key') ELSE COUNT(*) END AS source_rows, COUNT(*) FILTER (WHERE lnexp12m IS NULL) AS missing_outcomes, COUNT(DISTINCT commune) AS source_communes FROM active"""
sql """SELECT * FROM active WHERE lnexp12m IS NOT NULL ORDER BY source_row""" into household_hausman_cohort
xtreg lnexp12m lnhhexp age sex hhsize farm educ, re id(commune) time(source_row) vcmethod(equal_group_between)
estat hausman
predict re_xb, xb
predict re_residual, residuals
summarize re_xb re_residual
sql """SELECT 'Classical joint slope covariance-difference Hausman test with separate FE/RE residual scales. Requires RE efficiency under H0 and IID homoskedastic idiosyncratic errors. Not cluster-robust, survey or causal inference; non-rejection does not establish exogeneity. Retains original equal-group-between RE-GLS.' AS scope"""
export household_hausman_predictions.csv
