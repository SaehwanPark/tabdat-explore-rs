# Combined pooled, FE and RE residual snapshot; descriptive, not a test.
use MOM.csv
isid id year
isid source_row
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(*)>5320 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR source_row IS NULL OR lnhr IS NULL OR lnwg IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR NOT isfinite(source_row) OR NOT isfinite(lnhr) OR NOT isfinite(lnwg) OR id!=floor(id) OR id<1 OR id>532 OR year!=floor(year) OR year<1979 OR year>1988 OR source_row!=floor(source_row) OR source_row<1 OR source_row>5320 OR ABS(lnhr)>20 OR ABS(lnwg)>20)>0 THEN error('joint residual summary requires finite bounded MOM model fields and exact keys') ELSE COUNT(*) END AS observations FROM active"""
sql """SELECT CASE WHEN COUNT(*)<3 OR COUNT(*)>532 OR MIN(t)!=10 OR MAX(t)!=10 THEN error('joint residual summary requires complete ten-year people') ELSE COUNT(*) END AS people FROM (SELECT id,COUNT(*) AS t FROM active GROUP BY id)"""
sql """SELECT 'Pooled OLS and within FE residuals share the complete sample. Native RE population residual includes the random effect; this estimation-sample composition predicts u from shrinkage and e as population residual minus u. Summaries are descriptive, not a test or covariance correction.' AS residual_scope"""
regress lnhr lnwg
predict pooled_xb, xb
predict pooled_residual, residuals
xtreg lnhr lnwg, fe id(id) time(year)
predict fe_xb, xb
predict fe_xbu, xbu
predict fe_u, u
predict fe_residual, residuals
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
    OR raw_u IS NULL OR NOT isfinite(raw_u) THEN error('joint residual summary requires positive within error variance and identified within/between designs') ELSE e END AS sigma_e_squared,
    raw_u AS raw_sigma_u_squared,GREATEST(0,raw_u) AS sigma_u_squared
  FROM within_stats CROSS JOIN within_variance CROSS JOIN between_fit CROSS JOIN between_variance
), candidate AS (
  SELECT *,1-sqrt(sigma_e_squared/(sigma_e_squared+10*sigma_u_squared)) AS theta FROM guarded
), parameters AS (
  SELECT sigma_e_squared,raw_sigma_u_squared,sigma_u_squared,
    CASE WHEN theta IS NULL OR NOT isfinite(theta) OR theta<0 OR theta>=1 THEN error('joint residual summary requires finite RE theta in [0,1)') ELSE theta END AS theta FROM candidate
)
SELECT centered.*,parameters.*,1-(1-theta)^2 AS shrink,
  (1-(1-theta)^2)*mean_population_residual AS re_unit_effect,
  population_xb+(1-(1-theta)^2)*mean_population_residual AS conditional_xbu,
  population_residual-(1-(1-theta)^2)*mean_population_residual AS re_e
FROM centered CROSS JOIN parameters ORDER BY source_row
""" into joint_residual_rows
sql """SELECT CASE WHEN COUNT(*) FILTER (WHERE pooled_residual IS NULL OR fe_residual IS NULL OR re_e IS NULL OR NOT isfinite(pooled_residual) OR NOT isfinite(fe_residual) OR NOT isfinite(re_e))>0 THEN error('joint residual summary requires finite residuals') ELSE COUNT(*) END AS validated_rows FROM active"""
export panel_joint_residual_rows.csv
sql """
WITH long_values AS (
  SELECT source_row,year,'lnhr' AS variable,lnhr AS value FROM active UNION ALL
  SELECT source_row,year,'lnwg' AS variable,lnwg AS value FROM active UNION ALL
  SELECT source_row,year,'pooled_residual' AS variable,pooled_residual AS value FROM active UNION ALL
  SELECT source_row,year,'fe_residual' AS variable,fe_residual AS value FROM active UNION ALL
  SELECT source_row,year,'re_e' AS variable,re_e AS value FROM active
), summaries AS (
  SELECT 'overall' AS scope,CAST(NULL AS DOUBLE) AS year,variable,COUNT(*) AS n,
    AVG(value ORDER BY source_row) AS mean,STDDEV_SAMP(value ORDER BY source_row) AS std_dev,MIN(value) AS minimum,MAX(value) AS maximum
  FROM long_values GROUP BY variable UNION ALL
  SELECT 'year' AS scope,year,variable,COUNT(*) AS n,
    AVG(value ORDER BY source_row) AS mean,STDDEV_SAMP(value ORDER BY source_row) AS std_dev,MIN(value) AS minimum,MAX(value) AS maximum
  FROM long_values GROUP BY year,variable
)
SELECT * FROM summaries ORDER BY scope,year,variable
""" into joint_residual_summary
sql """SELECT scope,year,variable,n,mean,std_dev,minimum,maximum FROM active ORDER BY scope,year,variable"""
# RE retains its population prediction state across the materialized summary.
use joint_residual_rows
predict re_owned_xb, xb
sql """SELECT CASE WHEN COUNT(*) FILTER (WHERE re_owned_xb IS NULL OR NOT isfinite(re_owned_xb) OR ABS(re_owned_xb-population_xb)>1e-12)>0 THEN error('retained RE population prediction changed') ELSE COUNT(*) END AS retained_re_prediction_rows FROM active"""
use joint_residual_summary
export panel_joint_residual_summary.csv
