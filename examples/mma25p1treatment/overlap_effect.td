# Overlap-weighted effect with an estimated-logit estimating-equation sandwich.
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
THEN error('overlap weighting requires finite propensity scores strictly between zero and one') ELSE COUNT(*) END AS valid_scores FROM active
"""
sql """
WITH totals AS (SELECT sum(CASE WHEN treat=1 THEN 1-pscore ELSE 0 END) AS w1,
  sum(CASE WHEN treat=0 THEN pscore ELSE 0 END) AS w0,
  sum(CASE WHEN treat=1 THEN (1-pscore)*re78 ELSE 0 END)/sum(CASE WHEN treat=1 THEN 1-pscore ELSE 0 END) AS mu1,
  sum(CASE WHEN treat=0 THEN pscore*re78 ELSE 0 END)/sum(CASE WHEN treat=0 THEN pscore ELSE 0 END) AS mu0 FROM active)
SELECT CASE WHEN NOT coalesce(isfinite(w1) AND w1>0 AND isfinite(w0) AND w0>0
  AND isfinite(mu1) AND isfinite(mu0),FALSE)
THEN error('overlap weighting requires finite positive group weights and finite means') ELSE 1 END AS valid_totals FROM totals
"""
sql """
WITH totals AS (SELECT sum(CASE WHEN treat=1 THEN 1-pscore ELSE 0 END) AS w1,
  sum(CASE WHEN treat=0 THEN pscore ELSE 0 END) AS w0,
  sum(CASE WHEN treat=1 THEN (1-pscore)*re78 ELSE 0 END)/sum(CASE WHEN treat=1 THEN 1-pscore ELSE 0 END) AS mu1,
  sum(CASE WHEN treat=0 THEN pscore*re78 ELSE 0 END)/sum(CASE WHEN treat=0 THEN pscore ELSE 0 END) AS mu0 FROM active)
SELECT active.*,w1,w0,mu1,mu0,CASE WHEN treat=1 THEN 1-pscore ELSE pscore END AS overlap_weight,
  pscore*(1-pscore) AS information_weight,
  CASE WHEN treat=1 THEN -(re78-mu1)/w1 ELSE -(re78-mu0)/w0 END AS nuisance_response
FROM active CROSS JOIN totals ORDER BY source_row
""" into overlap_weight_design
sql """
SELECT CASE WHEN COUNT(*) FILTER (WHERE NOT coalesce(isfinite(overlap_weight) AND overlap_weight>0
  AND isfinite(information_weight) AND information_weight>0 AND isfinite(nuisance_response),FALSE))>0
THEN error('overlap weighting requires finite positive weights and nuisance response') ELSE COUNT(*) END AS valid_weights FROM active
"""
sql """SELECT 'The following weighted regression is an internal information-matrix solve; its coefficients and inference are not treatment effects' AS interpretation FROM active LIMIT 1"""
regress nuisance_response age_s age_s2 educ_s educ_s2 marr nodegree black hisp re74_s re75_s re74_s2 re75_s2 u74black, wls(information_weight)
predict nuisance_multiplier, xb
sql """
SELECT *,CASE WHEN treat=1 THEN overlap_weight*(re78-mu1)/w1 ELSE -overlap_weight*(re78-mu0)/w0 END
  +nuisance_multiplier*(treat-pscore) AS influence
FROM active ORDER BY source_row
""" into overlap_effect_influences
sql """
SELECT CASE WHEN COUNT(*) FILTER (WHERE NOT coalesce(isfinite(nuisance_multiplier) AND isfinite(influence),FALSE))>0
THEN error('overlap weighting requires finite influence values') ELSE COUNT(*) END AS valid_influences FROM active
"""
sql """
WITH totals AS (SELECT sum(influence*influence) AS variance,max(mu1-mu0) AS estimate,
  max(w1) AS w1,max(w0) AS w0,sum(overlap_weight*overlap_weight) FILTER (WHERE treat=1) AS s1,
  sum(overlap_weight*overlap_weight) FILTER (WHERE treat=0) AS s0 FROM active)
SELECT CASE WHEN NOT coalesce(isfinite(variance) AND variance>0 AND isfinite(estimate)
  AND isfinite(w1*w1) AND w1*w1>0 AND isfinite(w0*w0) AND w0*w0>0
  AND isfinite(s1) AND s1>0 AND isfinite(s0) AND s0>0
  AND isfinite(w1*w1/s1) AND isfinite(w0*w0/s0)
  AND isfinite(estimate-1.959963984540054*sqrt(variance))
  AND isfinite(estimate+1.959963984540054*sqrt(variance)),FALSE)
THEN error('overlap weighting requires finite positive variance and effective sample sizes') ELSE 1 END AS valid_inference FROM totals
"""
sql """
SELECT count(*) AS observations,sum(treat) AS treated_people,max(mu1) AS weighted_treated_mean,
  max(mu0) AS weighted_control_mean,max(mu1-mu0) AS overlap_effect,sqrt(sum(influence*influence)) AS sandwich_se,
  max(mu1-mu0)-1.959963984540054*sqrt(sum(influence*influence)) AS normal_95_lower,
  max(mu1-mu0)+1.959963984540054*sqrt(sum(influence*influence)) AS normal_95_upper,
  max(w1)*max(w1)/sum(overlap_weight*overlap_weight) FILTER (WHERE treat=1) AS treated_weight_ess,
  max(w0)*max(w0)/sum(overlap_weight*overlap_weight) FILTER (WHERE treat=0) AS control_weight_ess,
  max(overlap_weight) FILTER (WHERE treat=1) AS max_treated_weight,
  max(overlap_weight) FILTER (WHERE treat=0) AS max_control_weight,
  'Overlap target differs from ATT; sandwich includes estimated logit; causal claims require exchangeability and a correct score model' AS interpretation
FROM active
"""
sql """
SELECT treat,count(*) AS people,sum(overlap_weight) AS weight_sum,
  sum(overlap_weight*age_s)/sum(overlap_weight) AS age_s,
  sum(overlap_weight*age_s2)/sum(overlap_weight) AS age_s2,
  sum(overlap_weight*educ_s)/sum(overlap_weight) AS educ_s,
  sum(overlap_weight*educ_s2)/sum(overlap_weight) AS educ_s2,
  sum(overlap_weight*marr)/sum(overlap_weight) AS marr,
  sum(overlap_weight*nodegree)/sum(overlap_weight) AS nodegree,
  sum(overlap_weight*black)/sum(overlap_weight) AS black,
  sum(overlap_weight*hisp)/sum(overlap_weight) AS hisp,
  sum(overlap_weight*re74_s)/sum(overlap_weight) AS re74_s,
  sum(overlap_weight*re75_s)/sum(overlap_weight) AS re75_s,
  sum(overlap_weight*re74_s2)/sum(overlap_weight) AS re74_s2,
  sum(overlap_weight*re75_s2)/sum(overlap_weight) AS re75_s2,
  sum(overlap_weight*u74black)/sum(overlap_weight) AS u74black
FROM active GROUP BY treat ORDER BY treat
"""
sql """SELECT source_row,treat,re78,pscore,overlap_weight,mu1,mu0,nuisance_multiplier,influence FROM active ORDER BY source_row""" into overlap_effect_output
export overlap_effect_rows.csv
