# Shared native PSID preparation for descriptive density and LOWESS workflows.
# Fixed-width -> CSV conversion is separate/development-only (prepare.py).
use psid.csv
isid source_row
sql """
SELECT CASE WHEN COUNT(*)<2 OR COUNT(*)>10000 OR COUNT(*) FILTER (WHERE NOT coalesce(
  isfinite(source_row) AND source_row>0 AND source_row<=10000 AND source_row=floor(source_row)
  AND isfinite(intnum) AND intnum=floor(intnum) AND intnum BETWEEN 0 AND 9999
  AND isfinite(persnum) AND persnum=floor(persnum) AND persnum BETWEEN 0 AND 999
  AND isfinite(age) AND age=floor(age) AND age BETWEEN 30 AND 50
  AND (educatn IS NULL OR (isfinite(educatn) AND educatn=floor(educatn) AND educatn BETWEEN 0 AND 99))
  AND isfinite(earnings) AND earnings=floor(earnings) AND earnings BETWEEN 0 AND 999999
  AND isfinite(hours) AND hours=floor(hours) AND hours BETWEEN 0 AND 9999
  AND sex=2 AND isfinite(kids) AND kids=floor(kids) AND kids BETWEEN 0 AND 99
  AND isfinite(married) AND married=floor(married) AND married BETWEEN 1 AND 9,FALSE))>0
THEN error('PSID preparation requires2..10000 valid raw female age30..50 records with unique source_row; only education may be missing')
ELSE COUNT(*) END AS source_observations FROM active
"""
sql """
SELECT *,least(earnings,100000) AS capped_earnings,
  least(earnings,100000)::DOUBLE/hours AS hwage,
  ln(least(earnings,100000)::DOUBLE/hours) AS lnhwage
FROM active WHERE earnings>0 AND age=36 AND educatn NOT IN (0,98,99)
ORDER BY source_row
""" into psid_sample
sql """
SELECT CASE WHEN COUNT(*)<2 OR COUNT(*) FILTER (WHERE NOT coalesce(
  hours>0 AND isfinite(hwage) AND hwage>0 AND isfinite(lnhwage),FALSE))>0
THEN error('PSID selected cohort requires N>=2, positive hours/finite positive hourly wage and finite log wage')
ELSE COUNT(*) END AS selected_observations FROM active
"""
