# Figure9.2: conventional Epanechnikov density, three explicit bandwidths.
# Descriptive distribution, NOT regression, uncertainty or a general KDE command.
run prepare.td
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
