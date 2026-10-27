# Two observations per person: cluster inference by the stable person key.
use nswpsid.csv
isid source_row
sql """
SELECT CASE WHEN COUNT(*)<=4 OR COUNT(*)>5000
  OR MIN(source_row)<>1 OR MAX(source_row)<>COUNT(*)
  OR COUNT(*) FILTER (WHERE source_row<>input_position)>0
  OR COUNT(*) FILTER (WHERE NOT coalesce(isfinite(source_row) AND source_row=floor(source_row)
    AND treat IN (0,1) AND isfinite(re75) AND isfinite(re78),FALSE))>0
  OR COUNT(*) FILTER (WHERE treat=0)<2 OR COUNT(*) FILTER (WHERE treat=1)<2
THEN error('paired comparisons require ordered unique dense person IDs, finite earnings, binary treatment and at least two people per group')
ELSE COUNT(*) END AS people FROM (SELECT *,row_number() OVER () AS input_position FROM active)
"""
sql """
SELECT source_row,treat,0 AS post,re75 AS earnings,0 AS interaction FROM active
UNION ALL
SELECT source_row,treat,1 AS post,re78 AS earnings,treat AS interaction FROM active
ORDER BY source_row,post
""" into paired_earnings
isid source_row post
sql """
SELECT treat,post,count(*) AS observations,avg(earnings) AS mean_earnings,
  'NSW treated versus PSID controls; person-clustered uncertainty, observational comparisons' AS interpretation
FROM active GROUP BY treat,post ORDER BY treat,post
"""
regress earnings post if treat == 1, cluster(source_row)
lincom post
estimates store before_after
regress earnings interaction treat post, cluster(source_row)
lincom interaction
estimates store diff_in_diff
predict fitted_earnings, xb
estimates table before_after diff_in_diff
sql """SELECT source_row,treat,post,earnings,interaction,fitted_earnings FROM active ORDER BY source_row,post""" into paired_predictions
export paired_predictions.csv
