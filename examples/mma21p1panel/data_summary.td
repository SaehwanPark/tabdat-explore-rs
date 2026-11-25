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

# Explicit keys keep this diagnostic read-only and do not retain panel state.
xtdes, i(id) t(year)

xtsum lnhr lnwg kids ageh agesq disab, i(id)
