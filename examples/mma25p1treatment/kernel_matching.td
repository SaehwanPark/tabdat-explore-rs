# Fixed-bandwidth Gaussian kernel comparison with estimated-score sandwich.
use nswpsid.csv
isid source_row
sql """
SELECT CASE WHEN COUNT(*)<=14 OR COUNT(*)>10000
  OR MIN(source_row)<>1 OR MAX(source_row)<>COUNT(*)
  OR COUNT(*) FILTER (WHERE source_row<>input_position)>0
  OR COUNT(*) FILTER (WHERE NOT coalesce(isfinite(source_row) AND source_row=floor(source_row)
    AND treat IN (0,1) AND isfinite(age) AND isfinite((age/10)*(age/10))
    AND isfinite(educ) AND isfinite((educ/10)*(educ/10))
    AND black IN (0,1) AND hisp IN (0,1) AND marr IN (0,1)
    AND isfinite(re74) AND re74>=0 AND isfinite(re75) AND re75>=0 AND isfinite(re78)
    AND isfinite((re74/10000)*(re74/10000)) AND isfinite((re75/10000)*(re75/10000)),FALSE))>0
  OR COUNT(*) FILTER (WHERE treat=0)<2 OR COUNT(*) FILTER (WHERE treat=1)<2
  OR NOT coalesce(isfinite(var_samp(re78) FILTER (WHERE treat=0))
    AND isfinite(var_samp(re78) FILTER (WHERE treat=1))
    AND (var_samp(re78) FILTER (WHERE treat=0)+var_samp(re78) FILTER (WHERE treat=1))>0,FALSE)
THEN error('propensity diagnostics require ordered dense unique IDs, finite data, binary groups, nonnegative pretreatment earnings, both groups and nonzero within-group outcome variation')
ELSE COUNT(*) END AS observations FROM (SELECT *,row_number() OVER () AS input_position FROM active)
"""
sql """
SELECT *, age/10 AS age_s,(age/10)*(age/10) AS age_s2,
  educ/10 AS educ_s,(educ/10)*(educ/10) AS educ_s2,
  CASE WHEN educ<12 THEN 1 ELSE 0 END AS nodegree,
  re74/10000 AS re74_s,re75/10000 AS re75_s,
  (re74/10000)*(re74/10000) AS re74_s2,(re75/10000)*(re75/10000) AS re75_s2,
  CASE WHEN re74=0 THEN black ELSE 0 END AS u74black
FROM active ORDER BY source_row
""" into propensity_design
logit treat age_s age_s2 educ_s educ_s2 marr nodegree black hisp re74_s re75_s re74_s2 re75_s2 u74black
estimates store propensity
predict pscore, pr
sql """
SELECT CASE WHEN COUNT(*) FILTER (WHERE NOT coalesce(isfinite(pscore) AND pscore>0 AND pscore<1,FALSE))>0
  OR COUNT(*) FILTER (WHERE treat=1)*COUNT(*) FILTER (WHERE treat=0)>2000000
THEN error('kernel comparison requires finite open-unit scores and at most 2000000 treated-control pairs')
ELSE COUNT(*) END AS valid_scores FROM active
"""
sql """SELECT * FROM active ORDER BY source_row""" into kernel_scored
sql """
WITH treated AS (SELECT source_row AS ti,pscore AS tp,re78 AS ty FROM active WHERE treat=1),
controls AS (SELECT source_row AS ci,pscore AS cp,re78 AS cy FROM active WHERE treat=0),
raw AS (SELECT ti,tp,ty,ci,cp,cy,tp-cp AS delta,
  exp(-0.5*((tp-cp)/0.06)*((tp-cp)/0.06)) AS kernel FROM treated CROSS JOIN controls),
weighted AS (SELECT *,sum(kernel) OVER (PARTITION BY ti) AS denominator,
  sum(kernel*cy) OVER (PARTITION BY ti) AS numerator,
  min(abs(delta)) OVER (PARTITION BY ti) AS nearest_distance FROM raw)
SELECT ti,tp,ty,ci,cp,cy,delta,kernel/denominator AS pair_weight,
  numerator/denominator AS matched_mean,nearest_distance FROM weighted ORDER BY ti,ci
""" into kernel_pairs
sql """
SELECT CASE WHEN COUNT(*) FILTER (WHERE NOT coalesce(isfinite(pair_weight) AND pair_weight>0
  AND isfinite(matched_mean) AND isfinite(nearest_distance),FALSE))>0
THEN error('kernel comparison requires finite positive pair weights and finite matched means') ELSE COUNT(*) END AS valid_pairs FROM active
"""
sql """
WITH treated AS (SELECT ti,first(ty-matched_mean) AS contrast,first(matched_mean) AS matched_mean,
  first(nearest_distance) AS nearest_distance,1/sum(pair_weight*pair_weight) AS match_ess,
  sum(pair_weight*(cy-matched_mean)*delta)*first(tp*(1-tp))/(0.06*0.06) AS score_sum
  FROM active GROUP BY ti),
controls AS (SELECT ci,sum(pair_weight*(cy-matched_mean)) AS residual_sum,
  -sum(pair_weight*(cy-matched_mean)*delta)*first(cp*(1-cp))/(0.06*0.06) AS score_sum,
  sum(pair_weight) AS reuse_weight FROM active GROUP BY ci),
summary AS (SELECT count(*) AS treated_n,avg(contrast) AS estimate FROM treated)
SELECT ti AS source_row,(contrast-estimate)/treated_n AS direct_influence,score_sum/treated_n AS score_numerator,
  matched_mean,nearest_distance,match_ess,CAST(NULL AS DOUBLE) AS reuse_weight FROM treated CROSS JOIN summary
UNION ALL
SELECT ci AS source_row,-residual_sum/treated_n AS direct_influence,score_sum/treated_n AS score_numerator,
  CAST(NULL AS DOUBLE) AS matched_mean,CAST(NULL AS DOUBLE) AS nearest_distance,
  CAST(NULL AS DOUBLE) AS match_ess,reuse_weight FROM controls CROSS JOIN summary
ORDER BY source_row
""" into kernel_components
isid source_row
use kernel_scored
join kernel_components on source_row
sql """
SELECT *,pscore*(1-pscore) AS information_weight,score_numerator/(pscore*(1-pscore)) AS nuisance_response
FROM active ORDER BY source_row
""" into kernel_design
sql """
SELECT CASE WHEN COUNT(*) FILTER (WHERE NOT coalesce(isfinite(direct_influence) AND isfinite(score_numerator)
  AND isfinite(information_weight) AND information_weight>0 AND isfinite(nuisance_response)
  AND ((treat=1 AND isfinite(matched_mean) AND isfinite(nearest_distance) AND isfinite(match_ess) AND match_ess>0)
    OR (treat=0 AND isfinite(reuse_weight) AND reuse_weight>0)),FALSE))>0
THEN error('kernel comparison requires finite matched values, influence components and nuisance response') ELSE COUNT(*) END AS valid_components FROM active
"""
sql """SELECT 'The following weighted regression is an internal information-matrix solve; its coefficients and inference are not matching effects' AS interpretation FROM active LIMIT 1"""
regress nuisance_response age_s age_s2 educ_s educ_s2 marr nodegree black hisp re74_s re75_s re74_s2 re75_s2 u74black, wls(information_weight)
predict nuisance_multiplier, xb
sql """
SELECT *,direct_influence+nuisance_multiplier*(treat-pscore) AS influence
FROM active ORDER BY source_row
""" into kernel_influences
sql """
WITH totals AS (SELECT avg(re78-matched_mean) FILTER (WHERE treat=1) AS estimate,
  sum(influence*influence) AS variance,
  count(*) FILTER (WHERE NOT coalesce(isfinite(nuisance_multiplier) AND isfinite(influence),FALSE)) AS invalid FROM active)
SELECT CASE WHEN invalid>0 OR NOT coalesce(isfinite(variance) AND variance>0 AND isfinite(estimate)
  AND isfinite(estimate-1.959963984540054*sqrt(variance))
  AND isfinite(estimate+1.959963984540054*sqrt(variance)),FALSE)
THEN error('kernel comparison requires finite influence and positive finite variance') ELSE 1 END AS valid_inference FROM totals
"""
sql """
SELECT count(*) AS observations,count(*) FILTER (WHERE treat=1) AS treated_people,
  avg(re78-matched_mean) FILTER (WHERE treat=1) AS kernel_contrast,sqrt(sum(influence*influence)) AS sandwich_se,
  avg(re78-matched_mean) FILTER (WHERE treat=1)-1.959963984540054*sqrt(sum(influence*influence)) AS normal_95_lower,
  avg(re78-matched_mean) FILTER (WHERE treat=1)+1.959963984540054*sqrt(sum(influence*influence)) AS normal_95_upper,
  min(match_ess) FILTER (WHERE treat=1) AS min_match_ess,avg(match_ess) FILTER (WHERE treat=1) AS mean_match_ess,
  max(nearest_distance) FILTER (WHERE treat=1) AS max_nearest_distance,
  max(reuse_weight) FILTER (WHERE treat=0) AS max_control_reuse,
  'Fixed h=0.06 Gaussian kernel contrast; includes estimated logit; finite bandwidth is not automatically a causal ATT' AS interpretation
FROM active
"""
sql """SELECT source_row,treat,re78,pscore,matched_mean,nearest_distance,match_ess,reuse_weight,direct_influence,score_numerator,nuisance_multiplier,influence FROM active ORDER BY source_row""" into kernel_output
export kernel_matching_rows.csv
