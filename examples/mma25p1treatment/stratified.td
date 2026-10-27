# Source-program stratified observed group differences; no overall uncertainty is inferred.
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
THEN error('propensity diagnostics require finite scores strictly between zero and one') ELSE COUNT(*) END AS valid_scores FROM active
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
sql """
WITH observed AS (
  SELECT stratum,count(*) FILTER (WHERE treat=1) AS treated_n,
    count(*) FILTER (WHERE treat=0) AS control_n,
    avg(re78) FILTER (WHERE treat=1) AS treated_mean,
    avg(re78) FILTER (WHERE treat=0) AS control_mean
  FROM active WHERE in_overlap GROUP BY stratum
), cells AS (
  SELECT bins.stratum,coalesce(observed.treated_n,0) AS treated_n,coalesce(observed.control_n,0) AS control_n,
    observed.treated_mean,observed.control_mean
  FROM (SELECT unnest(generate_series(1,10)) AS stratum) bins
  LEFT JOIN observed ON bins.stratum=observed.stratum
)
SELECT stratum,treated_n,control_n,treated_mean,control_mean,
  CASE WHEN treated_n>0 AND control_n>0 THEN treated_mean-control_mean ELSE NULL END AS difference,
  CASE WHEN treated_n=0 AND control_n=0 THEN 'empty'
    WHEN treated_n=0 THEN 'no_treated' WHEN control_n=0 THEN 'no_control' ELSE 'supported' END AS status
FROM cells ORDER BY stratum
""" into strata_results
sql """
SELECT CASE WHEN COUNT(*) FILTER (WHERE NOT coalesce(
  (treated_n=0 OR isfinite(treated_mean)) AND (control_n=0 OR isfinite(control_mean))
  AND (status<>'supported' OR isfinite(difference)),FALSE))>0
  OR sum(treated_n) FILTER (WHERE status='supported') IS NULL
  OR NOT coalesce(isfinite(sum(treated_n*difference) FILTER (WHERE status='supported'))
    AND isfinite(sum(treated_n*difference) FILTER (WHERE status='supported')/sum(treated_n) FILTER (WHERE status='supported')),FALSE)
THEN error('stratified comparison requires at least one supported cell and finite group means and contrast')
ELSE COUNT(*) END AS valid_strata FROM active
"""
sql """SELECT stratum,treated_n,control_n,treated_mean,control_mean,difference,status FROM active ORDER BY stratum"""
sql """
SELECT sum(treated_n) AS common_range_treated,
  sum(treated_n) FILTER (WHERE status='supported') AS supported_treated,
  coalesce(sum(treated_n) FILTER (WHERE status<>'supported'),0) AS unsupported_treated,
  count(*) FILTER (WHERE status='supported') AS supported_strata,
  sum(treated_n*difference) FILTER (WHERE status='supported')/sum(treated_n) FILTER (WHERE status='supported') AS treated_weighted_difference,
  CAST(NULL AS DOUBLE) AS standard_error,
  'Observed outcome contrast; no overall uncertainty estimated; fitted scores and empirical range are data-dependent' AS interpretation
FROM active
"""
use propensity_scores
sql """
WITH cells AS (SELECT stratum,count(*) FILTER (WHERE treat=1) AS treated_n,
  count(*) FILTER (WHERE treat=0) AS control_n,avg(re78) FILTER (WHERE treat=1) AS treated_mean,
  avg(re78) FILTER (WHERE treat=0) AS control_mean FROM active WHERE in_overlap GROUP BY stratum)
SELECT p.source_row,p.treat,p.re78,p.pscore,p.lower_bound,p.upper_bound,p.in_overlap,p.stratum,
  c.treated_n,c.control_n,c.treated_mean,c.control_mean,
  CASE WHEN c.treated_n>0 AND c.control_n>0 THEN c.treated_mean-c.control_mean ELSE NULL END AS difference,
  CASE WHEN c.stratum IS NULL THEN NULL WHEN c.treated_n=0 THEN 'no_treated'
    WHEN c.control_n=0 THEN 'no_control' ELSE 'supported' END AS status
FROM active p LEFT JOIN cells c ON p.stratum=c.stratum ORDER BY p.source_row
""" into stratified_records
export stratified_records.csv
