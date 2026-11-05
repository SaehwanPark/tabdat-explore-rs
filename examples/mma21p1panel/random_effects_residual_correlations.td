# P3-RE: estimation-sample RE idiosyncratic residual correlations.
use MOM.csv
isid id year
isid source_row
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(*)>5320 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR source_row IS NULL OR lnhr IS NULL OR lnwg IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR NOT isfinite(source_row) OR NOT isfinite(lnhr) OR NOT isfinite(lnwg) OR id!=floor(id) OR id<1 OR id>532 OR year!=floor(year) OR year<1979 OR year>1988 OR source_row!=floor(source_row) OR source_row<1 OR source_row>5320 OR ABS(lnhr)>20 OR ABS(lnwg)>20)>0 THEN error('RE residual correlations require finite bounded MOM model fields and exact keys') ELSE COUNT(*) END AS observations FROM active"""
sql """SELECT CASE WHEN COUNT(*)<3 OR COUNT(*)>532 OR MIN(t)!=10 OR MAX(t)!=10 THEN error('RE residual correlations require complete ten-year people') ELSE COUNT(*) END AS people FROM (SELECT id,COUNT(*) AS t FROM active GROUP BY id)"""
sql """SELECT 'The native population residual includes the random effect. This estimation-sample SQL composition predicts u as shrinkage times mean population residual and e as population residual minus u. Correlations are descriptive across people, not a test or covariance correction. Every year requires positive finite cross-person e variance.' AS correlation_scope"""
xtreg lnhr lnwg, re id(id) time(year)
predict population_xb, xb
predict population_residual, residuals
sql """
WITH means AS (
  SELECT *,AVG(lnhr ORDER BY year) OVER (PARTITION BY id) AS mean_lnhr,
    AVG(lnwg ORDER BY year) OVER (PARTITION BY id) AS mean_lnwg,
    AVG(population_residual ORDER BY year) OVER (PARTITION BY id) AS mean_population_residual FROM active
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
    OR raw_u IS NULL OR NOT isfinite(raw_u) THEN error('RE residual correlations require positive within error variance and identified within/between designs') ELSE e END AS sigma_e_squared,
    raw_u AS raw_sigma_u_squared,GREATEST(0,raw_u) AS sigma_u_squared
  FROM within_stats CROSS JOIN within_variance CROSS JOIN between_fit CROSS JOIN between_variance
), candidate AS (
  SELECT *,1-sqrt(sigma_e_squared/(sigma_e_squared+10*sigma_u_squared)) AS theta FROM guarded
), parameters AS (
  SELECT sigma_e_squared,raw_sigma_u_squared,sigma_u_squared,
    CASE WHEN theta IS NULL OR NOT isfinite(theta) OR theta<0 OR theta>=1 THEN error('RE residual correlations require finite theta in [0,1)') ELSE theta END AS theta FROM candidate
)
SELECT centered.*,parameters.*,1-(1-theta)^2 AS shrink,
  (1-(1-theta)^2)*mean_population_residual AS re_unit_effect,
  population_xb+(1-(1-theta)^2)*mean_population_residual AS conditional_xbu,
  population_residual-(1-(1-theta)^2)*mean_population_residual AS re_e
FROM centered CROSS JOIN parameters ORDER BY source_row
""" into re_residual_rows
sql """WITH years AS (SELECT year,COUNT(*) AS n,VAR_POP(re_e) AS v FROM active GROUP BY year) SELECT CASE WHEN COUNT(*)!=10 OR COUNT(*) FILTER (WHERE n!=(SELECT COUNT(DISTINCT id) FROM active) OR v IS NULL OR NOT isfinite(v) OR v<=0)>0 THEN error('RE residual correlations require positive finite cross-person e variance in every year') ELSE COUNT(*) END AS validated_years FROM years"""
sql """WITH pairs AS (SELECT a.year AS year_a,b.year AS year_b,COUNT(*) AS n,CORR(a.re_e,b.re_e) AS r FROM active a JOIN active b ON a.id=b.id GROUP BY a.year,b.year) SELECT CASE WHEN COUNT(*)!=100 OR COUNT(*) FILTER (WHERE n!=(SELECT COUNT(DISTINCT id) FROM active) OR r IS NULL OR NOT isfinite(r) OR ABS(r)>1.000000000001)>0 THEN error('RE residual correlations require 100 finite, identified year-pair correlations') ELSE COUNT(*) END AS validated_pairs FROM pairs"""
export panel_re_residual_rows.csv
sql """SELECT a.year AS year_a,b.year AS year_b,COUNT(*) AS people,CORR(a.re_e,b.re_e) AS correlation FROM active a JOIN active b ON a.id=b.id GROUP BY a.year,b.year ORDER BY a.year,b.year""" into re_residual_correlations
sql """SELECT year_a,year_b,people,correlation FROM active ORDER BY year_a,year_b"""
# RE retains its population prediction state across the SQL materialized matrix.
use re_residual_rows
predict re_owned_xb, xb
sql """SELECT CASE WHEN COUNT(*) FILTER (WHERE re_owned_xb IS NULL OR NOT isfinite(re_owned_xb) OR ABS(re_owned_xb-population_xb)>1e-12)>0 THEN error('retained RE population prediction changed') ELSE COUNT(*) END AS retained_re_prediction_rows FROM active"""
use re_residual_correlations
export panel_re_residual_correlations.csv
