# Figure9.2: conventional Epanechnikov density, three explicit bandwidths.
# Descriptive distribution, NOT regression, uncertainty or a general KDE command.
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
THEN error('PSID density requires2..10000 valid raw female age30..50 records with unique source_row; only education may be missing')
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
THEN error('PSID selected cohort requires N>=2, positive hours and finite positive hourly/log wage')
ELSE COUNT(*) END AS selected_observations FROM active
"""
sql """
WITH moments AS (SELECT COUNT(*) AS n,AVG(lnhwage) AS sample_mean,
  stddev_samp(lnhwage) AS sample_sd,quantile_cont(lnhwage,0.25) AS q25,
  quantile_cont(lnhwage,0.75) AS q75,MIN(lnhwage) AS min_x,MAX(lnhwage) AS max_x
FROM active), scales AS (SELECT *,least(sample_sd,(q75-q25)/1.349) AS scale FROM moments)
SELECT n,sample_mean,sample_sd,q25,q75,scale,1.3643*1.7188*scale/pow(n,0.2) AS bandwidth,min_x,max_x FROM scales
""" into psid_bandwidth
sql """
SELECT CASE WHEN n<2 OR NOT coalesce(isfinite(bandwidth) AND bandwidth>0 AND isfinite(min_x) AND isfinite(max_x) AND max_x>min_x,FALSE)
THEN error('PSID density requires finite positive plugin bandwidth/nondegenerate selected log wages')
ELSE bandwidth END AS conventional_epanechnikov_bandwidth FROM active
"""
export psid_bandwidth.csv
# Named datasets reactivate with use; SQL exposes the selected relation as active.
use psid_sample
sql """
WITH parameters AS (SELECT MIN(lnhwage) AS min_x,MAX(lnhwage) AS max_x,
  1.3643*1.7188*least(stddev_samp(lnhwage),
    (quantile_cont(lnhwage,0.75)-quantile_cont(lnhwage,0.25))/1.349)/pow(COUNT(*),0.2) AS bandwidth
FROM active), grid AS (SELECT r.range AS grid_index,min_x-2*bandwidth+
  (max_x-min_x+4*bandwidth)*r.range/511.0 AS log_wage_grid,bandwidth
FROM parameters CROSS JOIN range(512) AS r), differences AS (
SELECT grid_index,log_wage_grid,bandwidth,(log_wage_grid-lnhwage)/bandwidth AS u
FROM grid CROSS JOIN active)
SELECT grid_index,log_wage_grid,
  AVG(CASE WHEN abs(u)<0.5 THEN 0.75*(1-4*u*u) ELSE 0.0 END)/(0.5*bandwidth) AS density_half,
  AVG(CASE WHEN abs(u)<1 THEN 0.75*(1-u*u) ELSE 0.0 END)/bandwidth AS density_plugin,
  AVG(CASE WHEN abs(u)<2 THEN 0.75*(1-u*u/4) ELSE 0.0 END)/(2*bandwidth) AS density_double
FROM differences GROUP BY grid_index,log_wage_grid,bandwidth ORDER BY grid_index
""" into psid_density
export psid_density.csv
# Existing scatter overlays: evaluation marks coincide with plugin curve,
# not raw wage observations or confidence limits. Three lines are the estimates.
label variable log_wage_grid "Log hourly wage"
label variable density_half "One-half plug-in h"
label variable density_plugin "Density (plug-in h)"
label variable density_double "Two times plug-in h"
scatter density_plugin log_wage_grid, lines(density_half density_plugin density_double) title("Epanechnikov Density as Bandwidth Varies") saving("psid_density.svg") noopen
use psid_sample
export psid_selected.csv
