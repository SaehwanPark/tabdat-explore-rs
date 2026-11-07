# Classical pooled-residual RE LM; commune is a group, source_row is not time.
use vietnam.csv
isid source_row
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(*)>40000 OR COUNT(*) FILTER (WHERE typeof(source_row)!='BIGINT' OR source_row IS NULL OR source_row<1 OR typeof(commune)!='BIGINT' OR commune IS NULL OR commune<1 OR age IS NULL OR NOT isfinite(age) OR sex IS NULL OR NOT isfinite(sex) OR hhsize IS NULL OR NOT isfinite(hhsize) OR farm IS NULL OR NOT isfinite(farm) OR educ IS NULL OR NOT isfinite(educ) OR lnhhexp IS NULL OR NOT isfinite(lnhhexp) OR (lnexp12m IS NOT NULL AND NOT isfinite(lnexp12m)))>0 THEN error('invalid or oversized household source or commune key') ELSE COUNT(*) END AS source_rows, COUNT(*) FILTER (WHERE lnexp12m IS NULL) AS missing_outcomes, COUNT(DISTINCT commune) AS source_communes FROM active"""
sql """SELECT * FROM active WHERE lnexp12m IS NOT NULL ORDER BY source_row""" into household_bp_cohort
sql """SELECT CASE WHEN COUNT(DISTINCT commune)<2 OR COUNT(DISTINCT commune)>10000 OR COUNT(*)<=COUNT(DISTINCT commune) THEN error('BP LM requires 2..10000 selected communes and at least one within-group pair') ELSE COUNT(DISTINCT commune) END AS selected_communes FROM active"""
regress lnexp12m lnhhexp age sex hhsize farm educ
predict bp_xb, xb
predict bp_residual, residuals
sql """WITH groups AS (
 SELECT commune,COUNT(*) AS n,SUM(bp_residual ORDER BY source_row) AS residual_sum,
 SUM(bp_residual*bp_residual ORDER BY source_row) AS rss FROM active GROUP BY commune
), moments AS (
 SELECT SUM(n)::DOUBLE AS observations,COUNT(*) AS communes,MIN(n) AS minimum_group_size,
 MAX(n) AS maximum_group_size,COUNT(*) FILTER(WHERE n=1) AS singleton_groups,
 SUM(n::DOUBLE*n ORDER BY commune) AS sum_group_size_squared,
 SUM(residual_sum*residual_sum ORDER BY commune) AS group_residual_sum_squares,
 SUM(rss ORDER BY commune) AS residual_sum_of_squares,
 (SELECT VAR_POP(lnexp12m)*COUNT(*) FROM active) AS centered_outcome_sum_squares FROM groups
), guarded AS (
 SELECT *,CASE WHEN residual_sum_of_squares IS NULL OR NOT isfinite(residual_sum_of_squares)
 OR centered_outcome_sum_squares IS NULL OR NOT isfinite(centered_outcome_sum_squares)
 OR centered_outcome_sum_squares<=0 OR residual_sum_of_squares<=1e-20*centered_outcome_sum_squares
 OR group_residual_sum_squares IS NULL OR NOT isfinite(group_residual_sum_squares)
 OR sum_group_size_squared<=observations THEN error('BP LM requires finite residual moments and a non-nearly-perfect pooled fit')
 ELSE observations*(group_residual_sum_squares/residual_sum_of_squares-1)/sqrt(2*(sum_group_size_squared-observations)) END AS signed_score FROM moments
), result AS (
 SELECT *,signed_score*signed_score AS lm_statistic,2*tabdat_gauss_phi(-ABS(signed_score)) AS p_value FROM guarded
)
SELECT observations,communes,minimum_group_size,maximum_group_size,singleton_groups,
 sum_group_size_squared,group_residual_sum_squares,residual_sum_of_squares,centered_outcome_sum_squares,
 CASE WHEN signed_score IS NULL OR NOT isfinite(signed_score) OR lm_statistic IS NULL OR NOT isfinite(lm_statistic)
 OR p_value IS NULL OR NOT isfinite(p_value) THEN error('BP LM nonfinite statistic') ELSE signed_score END AS signed_score,
 lm_statistic,1 AS df,p_value,'pooled_residual_breusch_pagan_unbalanced' AS method,
 'legacy_squared_score_chi_square_1' AS reference_distribution,
 'Classical IID homoskedastic errors; not robust, one-sided variance inference, ML LR, survey or causal inference' AS scope
FROM result
""" into household_bp_diagnostic
sql """SELECT * FROM active"""
export household_bp_diagnostic.csv
use household_bp_cohort
summarize bp_xb bp_residual
export household_bp_predictions.csv
