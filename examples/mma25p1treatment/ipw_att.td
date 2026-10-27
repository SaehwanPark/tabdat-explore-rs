# Normalized IPW ATT with an estimated-logit estimating-equation sandwich.
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
THEN error('propensity diagnostics require ordered dense unique IDs, finite data, binary groups, nonnegative pretreatment earnings and both groups')
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
THEN error('IPW ATT requires finite propensity scores strictly between zero and one') ELSE COUNT(*) END AS valid_scores FROM active
"""
sql """
WITH totals AS (SELECT sum(treat) AS treated_n,
  sum(CASE WHEN treat=1 THEN re78 ELSE 0 END)/sum(treat) AS treated_mean,
  sum(CASE WHEN treat=0 THEN pscore/(1-pscore) ELSE 0 END) AS control_weight_sum,
  sum(CASE WHEN treat=0 THEN pscore/(1-pscore)*re78 ELSE 0 END)/sum(CASE WHEN treat=0 THEN pscore/(1-pscore) ELSE 0 END) AS weighted_control_mean
FROM active)
SELECT CASE WHEN NOT coalesce(isfinite(treated_mean) AND isfinite(weighted_control_mean)
  AND isfinite(control_weight_sum) AND control_weight_sum>0 AND treated_n>1,FALSE)
THEN error('IPW ATT requires finite weighted outcome totals and positive group weights') ELSE treated_n END AS valid_treated FROM totals
"""
sql """
WITH totals AS (SELECT sum(treat) AS treated_n,
  sum(CASE WHEN treat=1 THEN re78 ELSE 0 END)/sum(treat) AS treated_mean,
  sum(CASE WHEN treat=0 THEN pscore/(1-pscore) ELSE 0 END) AS control_weight_sum,
  sum(CASE WHEN treat=0 THEN pscore/(1-pscore)*re78 ELSE 0 END)/sum(CASE WHEN treat=0 THEN pscore/(1-pscore) ELSE 0 END) AS weighted_control_mean
FROM active)
SELECT active.*,treated_n,treated_mean,control_weight_sum,weighted_control_mean,
  CASE WHEN treat=0 THEN pscore/(1-pscore) ELSE 0 END AS control_weight,
  pscore*(1-pscore) AS information_weight,
  CASE WHEN treat=0 THEN (re78-weighted_control_mean)/(control_weight_sum*(1-pscore)*(1-pscore)) ELSE 0 END AS nuisance_response
FROM active CROSS JOIN totals ORDER BY source_row
""" into ipw_design
sql """
SELECT CASE WHEN COUNT(*) FILTER (WHERE NOT coalesce(isfinite(control_weight) AND isfinite(information_weight)
  AND information_weight>0 AND isfinite(nuisance_response),FALSE))>0
THEN error('IPW ATT requires finite weights and nuisance response') ELSE COUNT(*) END AS valid_weights FROM active
"""
sql """SELECT 'The following weighted regression is an internal information-matrix solve; its coefficients and inference are not treatment effects' AS interpretation FROM active LIMIT 1"""
regress nuisance_response age_s age_s2 educ_s educ_s2 marr nodegree black hisp re74_s re75_s re74_s2 re75_s2 u74black, wls(information_weight)
predict nuisance_multiplier, xb
sql """
SELECT *,CASE WHEN treat=1 THEN (re78-treated_mean)/treated_n
  ELSE -control_weight*(re78-weighted_control_mean)/control_weight_sum END
  -nuisance_multiplier*(treat-pscore) AS influence
FROM active ORDER BY source_row
""" into ipw_influences
sql """
WITH totals AS (SELECT sum(influence*influence) AS variance,sum(control_weight*control_weight) AS weight_square_sum,
  max(control_weight_sum) AS weight_sum,max(treated_mean-weighted_control_mean) AS estimate FROM active)
SELECT CASE WHEN NOT coalesce(isfinite(variance) AND variance>0 AND isfinite(weight_square_sum) AND weight_square_sum>0
  AND isfinite(weight_sum*weight_sum) AND isfinite(estimate) AND isfinite(estimate-1.959963984540054*sqrt(variance))
  AND isfinite(estimate+1.959963984540054*sqrt(variance)),FALSE)
THEN error('IPW ATT requires finite positive variance and effective sample size') ELSE 1 END AS valid_inference FROM totals
"""
sql """
SELECT count(*) AS observations,max(treated_n) AS treated_people,max(treated_mean) AS treated_mean,
  max(weighted_control_mean) AS weighted_control_mean,max(treated_mean-weighted_control_mean) AS att,
  sqrt(sum(influence*influence)) AS sandwich_se,
  max(treated_mean-weighted_control_mean)-1.959963984540054*sqrt(sum(influence*influence)) AS normal_95_lower,
  max(treated_mean-weighted_control_mean)+1.959963984540054*sqrt(sum(influence*influence)) AS normal_95_upper,
  max(control_weight_sum)*max(control_weight_sum)/sum(control_weight*control_weight) AS control_weight_ess,
  max(control_weight) AS max_control_weight,min(pscore) AS min_score,max(pscore) AS max_score,
  'Untrimmed normalized ATT; independent-person sandwich includes estimated logit; causal interpretation requires overlap and exchangeability' AS interpretation
FROM active
"""
sql """SELECT source_row,treat,re78,pscore,control_weight,treated_mean,weighted_control_mean,nuisance_multiplier,influence FROM active ORDER BY source_row""" into ipw_output
export ipw_att_rows.csv
