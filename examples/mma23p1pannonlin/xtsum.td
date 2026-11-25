# Source-specific reproduction of the author's full-decade xtsum, i(id) block.
use patents_decade_prepared.csv
isid id year
describe
sql """
WITH panels AS (
  SELECT id, COUNT(*) AS observations, MIN(year) AS first_year, MAX(year) AS last_year,
    COUNT(ardssic) AS ardssic_n,
    COUNT(logrl1) AS logrl1_n, COUNT(logrl2) AS logrl2_n,
    COUNT(logrl3) AS logrl3_n, COUNT(logrl4) AS logrl4_n,
    COUNT(logrl5) AS logrl5_n
  FROM active GROUP BY id
)
SELECT CASE WHEN COUNT(*) != 3460
  OR COUNT(DISTINCT id) != 346 OR MIN(id) != 1 OR MAX(id) != 346
  OR COUNT(DISTINCT year) != 10 OR MIN(year) != 1970 OR MAX(year) != 1979
  OR COUNT(*) FILTER (WHERE id IS NULL OR NOT isfinite(id) OR id != floor(id)
    OR year IS NULL OR NOT isfinite(year) OR year != floor(year)
    OR cusip IS NULL OR NOT isfinite(cusip)
    OR scisect IS NULL OR NOT isfinite(scisect)
    OR logk IS NULL OR NOT isfinite(logk)
    OR sumpat IS NULL OR NOT isfinite(sumpat)
    OR logr IS NULL OR NOT isfinite(logr)
    OR pat IS NULL OR NOT isfinite(pat)
    OR lpat IS NULL OR NOT isfinite(lpat)
    OR dpat IS NULL OR NOT isfinite(dpat)
    OR randd IS NULL OR NOT isfinite(randd)
    OR dyear2 IS NULL OR NOT isfinite(dyear2)
    OR dyear3 IS NULL OR NOT isfinite(dyear3)
    OR dyear4 IS NULL OR NOT isfinite(dyear4)
    OR dyear5 IS NULL OR NOT isfinite(dyear5)
    OR (ardssic IS NOT NULL AND NOT isfinite(ardssic))
    OR ((year < 1971 AND logrl1 IS NOT NULL) OR
        (year >= 1971 AND (logrl1 IS NULL OR NOT isfinite(logrl1))))
    OR ((year < 1972 AND logrl2 IS NOT NULL) OR
        (year >= 1972 AND (logrl2 IS NULL OR NOT isfinite(logrl2))))
    OR ((year < 1973 AND logrl3 IS NOT NULL) OR
        (year >= 1973 AND (logrl3 IS NULL OR NOT isfinite(logrl3))))
    OR ((year < 1974 AND logrl4 IS NOT NULL) OR
        (year >= 1974 AND (logrl4 IS NULL OR NOT isfinite(logrl4))))
    OR ((year < 1975 AND logrl5 IS NOT NULL) OR
        (year >= 1975 AND (logrl5 IS NULL OR NOT isfinite(logrl5))))
  ) != 0
  OR EXISTS (SELECT id FROM panels WHERE observations != 10 OR first_year != 1970 OR last_year != 1979)
  OR EXISTS (SELECT id FROM panels WHERE ardssic_n NOT IN (0, 10))
  OR (SELECT COUNT(*) FROM panels WHERE ardssic_n = 10) != 336
  OR (SELECT COUNT(*) FROM panels WHERE ardssic_n = 0) != 10
  OR EXISTS (SELECT id FROM panels WHERE logrl1_n != 9 OR logrl2_n != 8
    OR logrl3_n != 7 OR logrl4_n != 6 OR logrl5_n != 5)
  OR COUNT(ardssic) != 3360
  OR COUNT(logrl1) != 3114 OR COUNT(logrl2) != 2768
  OR COUNT(logrl3) != 2422 OR COUNT(logrl4) != 2076 OR COUNT(logrl5) != 1730
  THEN error('P-XTSUM requires the canonical 346-firm, 1970–1979 source and its variable-specific balanced missingness')
  ELSE COUNT(*)
END AS validated_rows
FROM active
"""
sql """
WITH long AS (
  SELECT 'id' AS variable, id, CAST(id AS DOUBLE) AS value FROM active
  -- The converter normalizes calendar years; restore the author's 70..79 scale for this summary row.
  UNION ALL SELECT 'year', id, CAST(year - 1900 AS DOUBLE) FROM active
  UNION ALL SELECT 'cusip', id, CAST(cusip AS DOUBLE) FROM active
  UNION ALL SELECT 'ardssic', id, CAST(ardssic AS DOUBLE) FROM active
  UNION ALL SELECT 'scisect', id, CAST(scisect AS DOUBLE) FROM active
  UNION ALL SELECT 'logk', id, CAST(logk AS DOUBLE) FROM active
  UNION ALL SELECT 'sumpat', id, CAST(sumpat AS DOUBLE) FROM active
  UNION ALL SELECT 'logr', id, CAST(logr AS DOUBLE) FROM active
  UNION ALL SELECT 'pat', id, CAST(pat AS DOUBLE) FROM active
  UNION ALL SELECT 'lpat', id, CAST(lpat AS DOUBLE) FROM active
  UNION ALL SELECT 'dpat', id, CAST(dpat AS DOUBLE) FROM active
  UNION ALL SELECT 'randd', id, CAST(randd AS DOUBLE) FROM active
  UNION ALL SELECT 'logrl1', id, CAST(logrl1 AS DOUBLE) FROM active
  UNION ALL SELECT 'logrl2', id, CAST(logrl2 AS DOUBLE) FROM active
  UNION ALL SELECT 'logrl3', id, CAST(logrl3 AS DOUBLE) FROM active
  UNION ALL SELECT 'logrl4', id, CAST(logrl4 AS DOUBLE) FROM active
  UNION ALL SELECT 'logrl5', id, CAST(logrl5 AS DOUBLE) FROM active
  UNION ALL SELECT 'dyear2', id, CAST(dyear2 AS DOUBLE) FROM active
  UNION ALL SELECT 'dyear3', id, CAST(dyear3 AS DOUBLE) FROM active
  UNION ALL SELECT 'dyear4', id, CAST(dyear4 AS DOUBLE) FROM active
  UNION ALL SELECT 'dyear5', id, CAST(dyear5 AS DOUBLE) FROM active
), overall AS (
  SELECT variable, COUNT(*) AS overall_N, AVG(value) AS overall_mean,
    STDDEV_SAMP(value) AS overall_sd, MIN(value) AS overall_min, MAX(value) AS overall_max
  FROM long WHERE value IS NOT NULL AND isfinite(value) GROUP BY variable
), person_means AS (
  SELECT variable, id, COUNT(*) AS observations, AVG(value) AS person_mean
  FROM long WHERE value IS NOT NULL AND isfinite(value) GROUP BY variable, id
), between_stats AS (
  SELECT variable, COUNT(*) AS between_n,
    STDDEV_SAMP(person_mean) AS between_sd,
    MIN(person_mean) AS between_min, MAX(person_mean) AS between_max,
    AVG(observations) AS mean_T
  FROM person_means GROUP BY variable
), within_stats AS (
  SELECT x.variable, COUNT(*) AS within_N,
    STDDEV_SAMP(x.value - p.person_mean + o.overall_mean) AS within_sd,
    MIN(x.value - p.person_mean + o.overall_mean) AS within_min,
    MAX(x.value - p.person_mean + o.overall_mean) AS within_max
  FROM long AS x
  JOIN person_means AS p USING (variable, id)
  JOIN overall AS o USING (variable)
  WHERE x.value IS NOT NULL AND isfinite(x.value)
  GROUP BY x.variable
)
SELECT o.variable, o.overall_N, o.overall_mean, o.overall_sd,
  o.overall_min, o.overall_max, b.between_n, b.between_sd,
  b.between_min, b.between_max, w.within_N, w.within_sd,
  w.within_min, w.within_max, b.mean_T
FROM overall AS o
JOIN between_stats AS b USING (variable)
JOIN within_stats AS w USING (variable)
ORDER BY CASE o.variable
  WHEN 'id' THEN 1 WHEN 'year' THEN 2 WHEN 'cusip' THEN 3
  WHEN 'ardssic' THEN 4 WHEN 'scisect' THEN 5 WHEN 'logk' THEN 6
  WHEN 'sumpat' THEN 7 WHEN 'logr' THEN 8 WHEN 'pat' THEN 9
  WHEN 'lpat' THEN 10 WHEN 'dpat' THEN 11 WHEN 'randd' THEN 12
  WHEN 'logrl1' THEN 13 WHEN 'logrl2' THEN 14 WHEN 'logrl3' THEN 15
  WHEN 'logrl4' THEN 16 WHEN 'logrl5' THEN 17 WHEN 'dyear2' THEN 18
  WHEN 'dyear3' THEN 19 WHEN 'dyear4' THEN 20 WHEN 'dyear5' THEN 21
END
"""
