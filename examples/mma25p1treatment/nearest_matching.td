# Descriptive nearest-score matching; no estimated uncertainty.
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
SELECT CASE WHEN COUNT(*) FILTER (WHERE NOT coalesce(isfinite(pscore) AND pscore>=0 AND pscore<=1,FALSE))>0
THEN error('nearest matching requires finite scores between zero and one') ELSE COUNT(*) END AS valid_scores FROM active
"""
sql """
WITH bounds AS (SELECT greatest(min(pscore) FILTER (WHERE treat=0),min(pscore) FILTER (WHERE treat=1)) AS lower_bound,
  least(max(pscore) FILTER (WHERE treat=0),max(pscore) FILTER (WHERE treat=1)) AS upper_bound FROM active)
SELECT CASE WHEN NOT coalesce(isfinite(lower_bound) AND isfinite(upper_bound) AND lower_bound<upper_bound,FALSE)
THEN error('propensity diagnostics require a nonempty common score range') ELSE 1 END AS valid_overlap FROM bounds
"""
sql """
WITH bounds AS (SELECT greatest(min(pscore) FILTER (WHERE treat=0),min(pscore) FILTER (WHERE treat=1)) AS lower_bound,
  least(max(pscore) FILTER (WHERE treat=0),max(pscore) FILTER (WHERE treat=1)) AS upper_bound FROM active)
SELECT active.*, lower_bound,upper_bound,pscore>=lower_bound AND pscore<=upper_bound AS in_overlap,
  CASE WHEN pscore>=lower_bound AND pscore<=upper_bound THEN CAST(ceil(10*pscore) AS INTEGER) ELSE NULL END AS stratum
FROM active CROSS JOIN bounds ORDER BY source_row
""" into propensity_scores
sql """SELECT count(*) FILTER (WHERE pscore IN (0,1)) AS rounded_boundary_scores,
  count(*) FILTER (WHERE pscore IN (0,1) AND NOT in_overlap) AS rounded_boundary_excluded,
  'Scores rounded to zero or one are allowed only outside the common range' AS interpretation FROM active"""
sql """
SELECT CASE WHEN COUNT(*) FILTER (WHERE in_overlap AND NOT coalesce(pscore>0 AND pscore<1,FALSE))>0
  OR COUNT(*) FILTER (WHERE in_overlap AND treat=1)<2
  OR COUNT(*) FILTER (WHERE in_overlap AND treat=0)<2
  OR COUNT(*) FILTER (WHERE in_overlap AND treat=1)*COUNT(*) FILTER (WHERE in_overlap AND treat=0)>2000000
THEN error('nearest matching requires interior common-range scores, at least two people per group and at most 2000000 candidate pairs')
ELSE COUNT(*) END AS valid_candidates FROM active
"""
sql """
WITH treated AS (SELECT source_row AS treated_id,pscore AS treated_score,re78 AS treated_outcome,
  age AS treated_age,educ AS treated_educ,black AS treated_black,re74 AS treated_re74,re75 AS treated_re75
  FROM active WHERE treat=1 AND in_overlap),
controls AS (SELECT source_row AS control_id,pscore AS control_score,re78 AS control_outcome,
  age AS control_age,educ AS control_educ,black AS control_black,re74 AS control_re74,re75 AS control_re75
  FROM active WHERE treat=0 AND in_overlap),
pairs AS (SELECT *,abs(treated_score-control_score) AS distance FROM treated CROSS JOIN controls),
ranked AS (SELECT *,min(distance) OVER (PARTITION BY treated_id) AS nearest_distance FROM pairs)
SELECT treated_id,treated_score,treated_outcome,control_id,control_score,control_outcome,distance,nearest_distance,
  treated_age,control_age,treated_educ,control_educ,treated_black,control_black,
  treated_re74,control_re74,treated_re75,control_re75
FROM ranked WHERE distance=nearest_distance ORDER BY treated_id,control_id
""" into nearest_raw_pairs
sql """
SELECT treated_id,treated_score,treated_outcome,control_id,control_score,control_outcome,distance,nearest_distance,
  count(*) OVER (PARTITION BY treated_id) AS tied_controls,
  avg(control_outcome ORDER BY control_id) OVER (PARTITION BY treated_id) AS matched_outcome,
  treated_age,control_age,treated_educ,control_educ,treated_black,control_black,
  treated_re74,control_re74,treated_re75,control_re75
FROM active ORDER BY treated_id,control_id
""" into nearest_pairs
sql """
SELECT CASE WHEN COUNT(*) FILTER (WHERE NOT coalesce(isfinite(distance) AND isfinite(matched_outcome)
  AND isfinite(treated_outcome-matched_outcome) AND tied_controls>0,FALSE))>0 THEN error('nearest matching requires finite distances and matched outcomes')
ELSE COUNT(*) END AS valid_matches FROM active
"""
sql """
WITH treated AS (SELECT treated_id,first(treated_outcome-matched_outcome) AS difference FROM active GROUP BY treated_id)
SELECT CASE WHEN NOT coalesce(isfinite(avg(difference ORDER BY treated_id)),FALSE)
THEN error('nearest matching requires a finite aggregate contrast') ELSE COUNT(*) END AS valid_contrasts FROM treated
"""
sql """
WITH treated AS (SELECT treated_id,first(treated_outcome) AS treated_outcome,first(matched_outcome) AS matched_outcome,
  first(distance) AS distance,first(tied_controls) AS tied_controls FROM active GROUP BY treated_id),
controls AS (SELECT control_id,sum(1.0/tied_controls ORDER BY treated_id) AS reuse_weight FROM active GROUP BY control_id),
summary AS (SELECT count(*) AS matched_treated,avg(treated_outcome-matched_outcome ORDER BY treated_id) AS contrast,
  max(distance) AS max_distance,count(*) FILTER (WHERE tied_controls>1) AS tied_treated FROM treated),
reuse AS (SELECT count(*) AS distinct_controls,max(reuse_weight) AS max_reuse FROM controls)
SELECT matched_treated,distinct_controls,contrast,max_distance,tied_treated,max_reuse,
  CAST(NULL AS DOUBLE) AS standard_error,
  'Descriptive contrast; no standard error; empirical common range and exact-tie averaging' AS interpretation
FROM summary CROSS JOIN reuse
"""
sql """
WITH covariates AS (
  SELECT 'age' AS covariate,treated_age AS treated_value,control_age AS control_value,tied_controls FROM active
  UNION ALL SELECT 'education',treated_educ,control_educ,tied_controls FROM active
  UNION ALL SELECT 'black',treated_black,control_black,tied_controls FROM active
  UNION ALL SELECT 're74',treated_re74,control_re74,tied_controls FROM active
  UNION ALL SELECT 're75',treated_re75,control_re75,tied_controls FROM active
), means AS (
  SELECT covariate,sum(treated_value/tied_controls)/sum(1.0/tied_controls) AS treated_mean,
    sum(control_value/tied_controls)/sum(1.0/tied_controls) AS matched_control_mean
  FROM covariates GROUP BY covariate
)
SELECT covariate,treated_mean,matched_control_mean,treated_mean-matched_control_mean AS difference,
  'Matched descriptive means; controls are weighted by reuse and exact-tie shares' AS interpretation
FROM means ORDER BY covariate
"""
export nearest_pairs.csv
