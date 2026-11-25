# Bounded reproduction of the source program's Chapter 21 data-inspection block.
use MOM.csv
isid source_row
isid id year
sql """
SELECT CASE
  WHEN COUNT(*) != 5320
    OR COUNT(DISTINCT source_row) != 5320
    OR MIN(source_row) != 1 OR MAX(source_row) != 5320
    OR COUNT(DISTINCT id) != 532 OR MIN(id) != 1 OR MAX(id) != 532
    OR COUNT(DISTINCT year) != 10 OR MIN(year) != 1979 OR MAX(year) != 1988
    OR COUNT(*) FILTER (WHERE
      source_row IS NULL OR NOT isfinite(source_row) OR source_row != floor(source_row)
      OR id IS NULL OR NOT isfinite(id) OR id != floor(id)
      OR year IS NULL OR NOT isfinite(year) OR year != floor(year)
      OR lnhr IS NULL OR NOT isfinite(lnhr)
      OR lnwg IS NULL OR NOT isfinite(lnwg)
      OR kids IS NULL OR NOT isfinite(kids)
      OR ageh IS NULL OR NOT isfinite(ageh)
      OR agesq IS NULL OR NOT isfinite(agesq)
      OR disab IS NULL OR NOT isfinite(disab)) > 0
    OR (SELECT MIN(n) FROM (SELECT COUNT(*) AS n FROM active GROUP BY id)) != 10
    OR (SELECT MAX(n) FROM (SELECT COUNT(*) AS n FROM active GROUP BY id)) != 10
    OR (SELECT COUNT(*) FROM (
      SELECT id FROM active GROUP BY id
      HAVING COUNT(DISTINCT year) != 10 OR MIN(year) != 1979 OR MAX(year) != 1988
    )) != 0
  THEN error('Chapter 21 data summary requires the complete finite MOM 532-person, 10-year source sample')
  ELSE COUNT(*)
END AS validated_source_rows,
COUNT(DISTINCT id) AS people,
MIN(year) AS first_year,
MAX(year) AS last_year
FROM active
"""
tabulate year
# source_row is converter provenance, not one of the author's 18 variables.
drop source_row
generate dyear1 = year == 1979
generate dyear2 = year == 1980
generate dyear3 = year == 1981
generate dyear4 = year == 1982
generate dyear5 = year == 1983
generate dyear6 = year == 1984
generate dyear7 = year == 1985
generate dyear8 = year == 1986
generate dyear9 = year == 1987
generate dyear10 = year == 1988
describe
summarize

# xtdes facts are explicit because this workflow does not set persistent panel state.
sql """
WITH people AS (
  SELECT id, COUNT(*) AS observations, MIN(year) AS first_year, MAX(year) AS last_year,
    STRING_AGG('1', '' ORDER BY year) AS pattern
  FROM active GROUP BY id
), steps AS (
  SELECT year - LAG(year) OVER (PARTITION BY id ORDER BY year) AS delta
  FROM active
)
SELECT (SELECT COUNT(*) FROM active) AS observations,
  COUNT(*) AS people, MIN(id) AS first_id, MAX(id) AS last_id,
  MIN(first_year) AS first_year, MAX(last_year) AS last_year,
  MIN(observations) AS min_T, AVG(observations) AS mean_T,
  MAX(observations) AS max_T,
  (SELECT MIN(delta) FROM steps WHERE delta IS NOT NULL) AS min_delta,
  (SELECT MAX(delta) FROM steps WHERE delta IS NOT NULL) AS max_delta
FROM people
"""

sql """
WITH people AS (
  SELECT id, COUNT(*) AS observations, STRING_AGG('1', '' ORDER BY year) AS pattern
  FROM active GROUP BY id
), frequencies AS (
  SELECT observations AS T_i, pattern, COUNT(*) AS people
  FROM people GROUP BY observations, pattern
)
SELECT T_i, pattern, people,
  100.0 * people / SUM(people) OVER () AS percent,
  100.0 * SUM(people) OVER (ORDER BY T_i, pattern ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW)
    / SUM(people) OVER () AS cumulative_percent
FROM frequencies ORDER BY T_i, pattern
"""

# Stata xtsum centers within values by person, then shifts them to the overall mean.
sql """
WITH long AS (
  SELECT 'lnhr' AS variable, id, lnhr AS value FROM active
  UNION ALL SELECT 'lnwg', id, lnwg FROM active
  UNION ALL SELECT 'kids', id, kids FROM active
  UNION ALL SELECT 'ageh', id, ageh FROM active
  UNION ALL SELECT 'agesq', id, agesq FROM active
  UNION ALL SELECT 'disab', id, disab FROM active
), overall AS (
  SELECT variable, COUNT(*) AS observations, AVG(value) AS overall_mean,
    STDDEV_SAMP(value) AS overall_sd, MIN(value) AS overall_min, MAX(value) AS overall_max
  FROM long GROUP BY variable
), person_means AS (
  SELECT variable, id, AVG(value) AS person_mean, COUNT(*) AS person_observations
  FROM long GROUP BY variable, id
), between_stats AS (
  SELECT variable, COUNT(*) AS people, STDDEV_SAMP(person_mean) AS between_sd,
    MIN(person_mean) AS between_min, MAX(person_mean) AS between_max,
    AVG(person_observations) AS mean_T
  FROM person_means GROUP BY variable
), within_stats AS (
  SELECT x.variable, COUNT(*) AS within_observations,
    AVG(x.value - p.person_mean + o.overall_mean) AS within_mean,
    STDDEV_SAMP(x.value - p.person_mean + o.overall_mean) AS within_sd,
    MIN(x.value - p.person_mean + o.overall_mean) AS within_min,
    MAX(x.value - p.person_mean + o.overall_mean) AS within_max
  FROM long AS x
  JOIN person_means AS p USING (variable, id)
  JOIN overall AS o USING (variable)
  GROUP BY x.variable
)
SELECT o.variable, o.observations AS overall_N, o.overall_mean, o.overall_sd,
  o.overall_min, o.overall_max, b.people AS between_n, b.between_sd,
  b.between_min, b.between_max, w.within_observations, b.mean_T,
  w.within_mean, w.within_sd, w.within_min, w.within_max
FROM overall AS o
JOIN between_stats AS b USING (variable)
JOIN within_stats AS w USING (variable)
ORDER BY CASE o.variable
  WHEN 'lnhr' THEN 1 WHEN 'lnwg' THEN 2 WHEN 'kids' THEN 3
  WHEN 'ageh' THEN 4 WHEN 'agesq' THEN 5 WHEN 'disab' THEN 6
END
"""
