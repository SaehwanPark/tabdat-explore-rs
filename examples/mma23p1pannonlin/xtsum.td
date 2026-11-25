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
xtsum, i(id)
