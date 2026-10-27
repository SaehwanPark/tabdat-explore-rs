# NSW treated versus PSID controls: descriptive and adjusted comparisons.
use nswpsid.csv
isid source_row
sql """
SELECT CASE WHEN COUNT(*)<=10 OR COUNT(*)>10000
  OR MIN(source_row)<>1 OR MAX(source_row)<>COUNT(*)
  OR COUNT(*) FILTER (WHERE source_row<>input_position)>0
  OR COUNT(*) FILTER (WHERE NOT coalesce(isfinite(source_row) AND source_row=floor(source_row)
    AND treat IN (0,1) AND isfinite(age) AND isfinite(educ)
    AND black IN (0,1) AND hisp IN (0,1) AND marr IN (0,1)
    AND isfinite(re74) AND isfinite(re75) AND isfinite(re78),FALSE))>0
  OR COUNT(*) FILTER (WHERE treat=0)<2 OR COUNT(*) FILTER (WHERE treat=1)<2
THEN error('NSW comparison requires ordered unique dense rows, finite data, binary groups and at least two per group')
ELSE COUNT(*) END AS observations FROM (SELECT *,row_number() OVER () AS input_position FROM active)
"""
sql """SELECT *,age*age AS age_sq,CASE WHEN educ<12 THEN 1 ELSE 0 END AS nodegree FROM active ORDER BY source_row""" into treatment_design
sql """
SELECT treat,count(*) AS observations,avg(age) AS age,avg(educ) AS educ,
  avg(nodegree) AS nodegree,avg(black) AS black,avg(hisp) AS hisp,
  avg(marr) AS married,avg(re74) AS earnings_1974,avg(re75) AS earnings_1975,
  avg(re78) AS earnings_1978,
  'NSW treated versus PSID controls; descriptive group means' AS interpretation
FROM active GROUP BY treat ORDER BY treat
"""
regress re78 treat, robust
estimates store unadjusted
predict unadjusted_mean, xb
regress re78 treat age age_sq educ nodegree black hisp re74 re75, robust
estimates store adjusted
predict adjusted_mean, xb
estimates table unadjusted adjusted
sql """SELECT source_row,treat,age,educ,black,hisp,marr,re74,re75,re78,age_sq,nodegree,unadjusted_mean,adjusted_mean FROM active ORDER BY source_row""" into treatment_predictions
export treatment_predictions.csv
