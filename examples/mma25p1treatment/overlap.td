# Assignment-model diagnostics; estimated scores do not establish causal overlap.
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
SELECT treat,count(*) AS people,count(*) FILTER (WHERE in_overlap) AS in_overlap,
  count(*) FILTER (WHERE NOT in_overlap) AS excluded,min(pscore) AS min_score,max(pscore) AS max_score,
  min(lower_bound) AS common_lower,max(upper_bound) AS common_upper,
  'Empirical score-range overlap is descriptive; estimated scores do not prove exchangeability or positivity' AS interpretation
FROM active GROUP BY treat ORDER BY treat
"""
sql """
WITH cells AS (SELECT avg(age) AS age,avg(educ) AS educ,avg(nodegree) AS nodegree,
  avg(black) AS black,avg(hisp) AS hisp,avg(marr) AS married,
  avg(re74) AS re74,avg(re75) AS re75,avg(u74black) AS u74black
FROM active WHERE in_overlap GROUP BY stratum,treat)
SELECT CASE WHEN COUNT(*) FILTER (WHERE NOT coalesce(isfinite(age) AND isfinite(educ) AND isfinite(nodegree)
  AND isfinite(black) AND isfinite(hisp) AND isfinite(married) AND isfinite(re74)
  AND isfinite(re75) AND isfinite(u74black),FALSE))>0
THEN error('propensity diagnostics require finite stratum means') ELSE COUNT(*) END AS valid_cells FROM cells
"""
sql """
WITH cells AS (SELECT stratum,treat,count(*) AS people,avg(age) AS age,avg(educ) AS educ,
  avg(nodegree) AS nodegree,avg(black) AS black,avg(hisp) AS hisp,avg(marr) AS married,
  avg(re74) AS re74,avg(re75) AS re75,avg(u74black) AS u74black
FROM active WHERE in_overlap GROUP BY stratum,treat)
SELECT strata.stratum,groups.treat,coalesce(cells.people,0) AS people,
  cells.age,cells.educ,cells.nodegree,cells.black,cells.hisp,cells.married,cells.re74,cells.re75,cells.u74black
FROM (SELECT unnest(generate_series(1,10)) AS stratum) strata
CROSS JOIN (SELECT 0 AS treat UNION ALL SELECT 1 AS treat) groups
LEFT JOIN cells ON cells.stratum=strata.stratum AND cells.treat=groups.treat
ORDER BY strata.stratum,groups.treat
"""
sql """SELECT source_row,treat,age,educ,black,hisp,marr,re74,re75,re78,nodegree,u74black,pscore,lower_bound,upper_bound,in_overlap,stratum FROM active ORDER BY source_row""" into propensity_output
export propensity_scores.csv
