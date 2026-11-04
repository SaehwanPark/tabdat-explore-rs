# Manual block5: original Swamy-Arora theta, fixed across covariance comparisons.
use MOM.csv
isid id year
isid source_row
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(*)>5320 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR source_row IS NULL OR lnhr IS NULL OR lnwg IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR NOT isfinite(source_row) OR NOT isfinite(lnhr) OR NOT isfinite(lnwg) OR id!=floor(id) OR id<1 OR id>532 OR year!=floor(year) OR year<1979 OR year>1988 OR source_row!=floor(source_row) OR source_row<1 OR source_row>5320 OR ABS(lnhr)>20 OR ABS(lnwg)>20)>0 THEN error('manual GLS comparison requires finite bounded MOM model fields and exact keys') ELSE COUNT(*) END AS observations FROM active"""
sql """SELECT CASE WHEN COUNT(*)<3 OR COUNT(*)>532 OR MIN(t)!=10 OR MAX(t)!=10 THEN error('manual GLS comparison requires complete ten-year people') ELSE COUNT(*) END AS people FROM (SELECT id,COUNT(*) AS t FROM active GROUP BY id)"""
sql """SELECT 'Native RE population coefficients below use normal inference. Subsequent auxiliary OLS freezes the original Swamy-Arora theta; its intercept equals (1-theta)*population intercept and its intervals use Student-t. These are working/large-sample comparisons, not ML information or a nominal-coverage guarantee.' AS inference_scope"""
xtreg lnhr lnwg, re id(id) time(year)
predict population_xb, xb
predict population_residual, residuals
sql """
WITH means AS (
  SELECT *,AVG(lnhr ORDER BY year) OVER (PARTITION BY id) AS mean_lnhr,
    AVG(lnwg ORDER BY year) OVER (PARTITION BY id) AS mean_lnwg FROM active
), centered AS (
  SELECT *,lnhr-mean_lnhr AS within_lnhr,lnwg-mean_lnwg AS within_lnwg FROM means
), within_stats AS (
  SELECT COUNT(*) AS n,COUNT(DISTINCT id) AS g,SUM(within_lnwg^2 ORDER BY source_row) AS within_ss,
    SUM(within_lnwg*within_lnhr ORDER BY source_row)/NULLIF(SUM(within_lnwg^2 ORDER BY source_row),0) AS b_within FROM centered
), within_variance AS (
  SELECT SUM((within_lnhr-b_within*within_lnwg)^2 ORDER BY source_row)/NULLIF(MAX(n-g-1),0) AS e FROM centered CROSS JOIN within_stats
), people AS (
  SELECT DISTINCT id,mean_lnhr,mean_lnwg FROM centered
), between_fit AS (
  SELECT REGR_SLOPE(mean_lnhr,mean_lnwg ORDER BY id) AS b_between,REGR_INTERCEPT(mean_lnhr,mean_lnwg ORDER BY id) AS a_between,
    VAR_POP(mean_lnwg ORDER BY id) AS between_variation FROM people
), between_variance AS (
  SELECT SUM((mean_lnhr-a_between-b_between*mean_lnwg)^2 ORDER BY id)/NULLIF(MAX(g-2),0)-MAX(e)/10 AS raw_u
  FROM people CROSS JOIN between_fit CROSS JOIN within_stats CROSS JOIN within_variance
), guarded AS (
  SELECT CASE WHEN within_ss IS NULL OR NOT isfinite(within_ss) OR within_ss<=0 OR b_within IS NULL OR NOT isfinite(b_within)
    OR e IS NULL OR NOT isfinite(e) OR e<=0 OR between_variation IS NULL OR NOT isfinite(between_variation) OR between_variation<=0
    OR raw_u IS NULL OR NOT isfinite(raw_u) THEN error('manual GLS requires positive within error variance and identified within/between designs') ELSE e END AS sigma_e_squared,
    raw_u AS raw_sigma_u_squared,GREATEST(0,raw_u) AS sigma_u_squared
  FROM within_stats CROSS JOIN within_variance CROSS JOIN between_fit CROSS JOIN between_variance
), candidate AS (
  SELECT *,1-sqrt(sigma_e_squared/(sigma_e_squared+10*sigma_u_squared)) AS theta FROM guarded
), parameters AS (
  SELECT sigma_e_squared,raw_sigma_u_squared,sigma_u_squared,
    CASE WHEN theta IS NULL OR NOT isfinite(theta) OR theta<0 OR theta>=1 THEN error('manual GLS requires finite theta in [0,1)') ELSE theta END AS theta FROM candidate
)
SELECT centered.*,parameters.*,lnhr-theta*mean_lnhr AS quasi_lnhr,lnwg-theta*mean_lnwg AS quasi_lnwg
FROM centered CROSS JOIN parameters ORDER BY source_row
""" into manual_gls_covariance_frame
regress quasi_lnhr quasi_lnwg
estimates store p2_gls_iid_working
lincom intercept
lincom quasi_lnwg
regress quasi_lnhr quasi_lnwg, robust
estimates store p2_gls_hc1_working
lincom intercept
lincom quasi_lnwg
regress quasi_lnhr quasi_lnwg, cluster(id)
estimates store p2_gls_person_cluster
estimates table p2_gls_iid_working p2_gls_hc1_working p2_gls_person_cluster
sql """SELECT 'Named columns are fixed-theta quasi OLS, not saved native RE models. CR1 assumes independent people/adequate cluster asymptotics. Auxiliary residual DF=N-2; clustered coefficient inference DF=G-1. Quasi-intercept covariance scales by (1-theta) per intercept axis; no full-joint covariance inversion is required.' AS comparison_limits"""
predict quasi_fitted, xb
predict quasi_residual, residuals
summarize theta sigma_e_squared raw_sigma_u_squared sigma_u_squared quasi_fitted quasi_residual
# Owned cluster covariance must survive reports, notes and materialized predictions.
lincom intercept
lincom quasi_lnwg
export panel_manual_gls_covariance_predictions.csv
