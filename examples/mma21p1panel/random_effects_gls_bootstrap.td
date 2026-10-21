# MMA §21.3 manual block5: fixed-theta RE-GLS transformed OLS bootstrap.
use MOM.csv
isid id year
isid source_row
sql """SELECT CASE WHEN COUNT(*) = 0 OR COUNT(*) FILTER (WHERE source_row IS NULL OR NOT isfinite(source_row) OR source_row!=floor(source_row) OR source_row<1 OR source_row>5320 OR id IS NULL OR year IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR id != floor(id) OR year != floor(year) OR id < 1 OR id > 532 OR year < 1979 OR year > 1988) > 0 THEN error('MOM requires nonempty exact integer id 1..532 and year 1979..1988 and integer source_row 1..5320') ELSE COUNT(*) END AS observations, COUNT(DISTINCT id) AS people, MIN(year) AS first_year, MAX(year) AS last_year FROM active"""
sql """WITH counts AS (SELECT id,COUNT(*) AS t FROM active WHERE isfinite(lnhr) AND isfinite(lnwg) GROUP BY id) SELECT CASE WHEN COUNT(*)<=2 OR MIN(t)<2 OR MIN(t)!=MAX(t) THEN error('RE-GLS bootstrap workflow requires balanced joint cases T>=2 and G>2') ELSE SUM(t) END AS retained_rows, COUNT(*) AS retained_people, MIN(t) AS retained_t FROM counts"""
sql """
WITH joint AS (
  SELECT * FROM active WHERE isfinite(lnhr) AND isfinite(lnwg)
), means AS (
  SELECT *, AVG(lnhr) OVER (PARTITION BY id ORDER BY source_row ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING) AS mean_lnhr,
    AVG(lnwg) OVER (PARTITION BY id ORDER BY source_row ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING) AS mean_lnwg,
    COUNT(*) OVER (PARTITION BY id) AS model_rows FROM joint
), centered AS (
  SELECT *, lnhr-mean_lnhr AS within_lnhr, lnwg-mean_lnwg AS within_lnwg FROM means
), within_stats AS (
  SELECT COUNT(*) AS n, COUNT(DISTINCT id) AS g, MIN(model_rows) AS t_min,
    MAX(model_rows) AS t_max, SUM(within_lnwg*within_lnwg ORDER BY source_row) AS within_ss,
    SUM(within_lnwg*within_lnhr ORDER BY source_row)/NULLIF(SUM(within_lnwg*within_lnwg ORDER BY source_row),0) AS b_within
  FROM centered
), within_variance AS (
  SELECT SUM((within_lnhr-b_within*within_lnwg)^2 ORDER BY source_row)/NULLIF(MAX(n-g-1),0) AS e
  FROM centered CROSS JOIN within_stats
), people AS (
  SELECT DISTINCT id, mean_lnhr, mean_lnwg FROM centered
), between_fit AS (
  SELECT REGR_SLOPE(mean_lnhr,mean_lnwg ORDER BY id) AS b_between,
    REGR_INTERCEPT(mean_lnhr,mean_lnwg ORDER BY id) AS a_between,
    VAR_POP(mean_lnwg ORDER BY id) AS between_variation FROM people
), between_variance AS (
  SELECT SUM((mean_lnhr-a_between-b_between*mean_lnwg)^2 ORDER BY id)/NULLIF(MAX(g-2),0)
    -MAX(e)/NULLIF(MAX(t_min),0) AS raw_u
  FROM people CROSS JOIN between_fit CROSS JOIN within_stats CROSS JOIN within_variance
), candidate AS (
  SELECT CASE WHEN n IS NULL OR n=0 OR g<=2 OR t_min<2 OR t_min!=t_max
    OR within_ss<=0 OR b_within IS NULL OR NOT isfinite(b_within)
    OR e IS NULL OR NOT isfinite(e) OR e<=0
    OR between_variation IS NULL OR between_variation<=0
    OR raw_u IS NULL OR NOT isfinite(raw_u)
    THEN error('RE-GLS bootstrap workflow requires balanced joint cases T>=2, G>2, positive within variance and identified within/between wage designs')
    ELSE 1-sqrt(e/(e+t_min*greatest(0,raw_u))) END AS theta
  FROM within_stats CROSS JOIN within_variance CROSS JOIN between_fit CROSS JOIN between_variance
), parameters AS (
  SELECT CASE WHEN theta IS NULL OR NOT isfinite(theta) OR theta<0 OR theta>=1
    THEN error('RE-GLS bootstrap theta must be finite in [0,1)') ELSE theta END AS theta
  FROM candidate
)
SELECT centered.* EXCLUDE(model_rows), theta,
  lnhr-theta*mean_lnhr AS quasi_lnhr, lnwg-theta*mean_lnwg AS quasi_lnwg
FROM centered CROSS JOIN parameters ORDER BY source_row
""" into regls_bootstrap_design
# Ordinary transformed intercept equals (1-theta) times the population intercept.
# Theta is fixed across bootstrap draws; this is not RE variance-component refitting.
regress quasi_lnhr quasi_lnwg
bootstrap quasi_lnhr quasi_lnwg, cluster(id) reps(500) seed(10001)
predict quasi_fitted, xb
predict quasi_residual, residuals
summarize theta quasi_fitted quasi_residual
export panel_regls_bootstrap_predictions.csv
