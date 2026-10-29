# One-slope conditional Poisson FE, Table 23.1: model-based, not panel robust.
use patents.csv
isid id year
sql """SELECT CASE WHEN COUNT(*) = 0 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR pat IS NULL OR logr IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR NOT isfinite(pat) OR NOT isfinite(logr) OR id != floor(id) OR id < 1 OR id > 346 OR ABS(logr) > 20 OR year != floor(year) OR year < 1975 OR year > 1979 OR pat < 0 OR pat != floor(pat) OR pat > 1000) > 0 THEN error('invalid patents firm/year/count/model data') ELSE COUNT(*) END AS candidate_rows, COUNT(DISTINCT id) AS candidate_firms FROM active"""
sql """SELECT CASE WHEN COUNT(*) = 0 OR MIN(n) != 5 OR MAX(n) != 5 THEN error('each firm requires all five years 1975–79') ELSE COUNT(*) END AS firms FROM (SELECT id, COUNT(*) AS n FROM active GROUP BY id)"""
sql """
WITH RECURSIVE
raw AS (
 SELECT *, SUM(pat ORDER BY year) OVER (PARTITION BY id) AS firm_total,
   logr-AVG(logr ORDER BY year) OVER (PARTITION BY id) AS centered_x
 FROM active
), checked AS (
 SELECT * EXCLUDE(centered_x),
   CASE WHEN centered_x IS NULL OR NOT isfinite(centered_x) OR ABS(centered_x)>20
     THEN error('conditional FE requires finite within-firm log-R&D deviations <= 20') ELSE centered_x END AS centered_x
 FROM raw
), positive AS (SELECT * FROM checked WHERE firm_total>0),
endpoint_units AS (
 SELECT id, MAX(firm_total) AS total,
   SUM(pat*centered_x ORDER BY year) AS observed,
   SUM(EXP(-10*centered_x) ORDER BY year) AS lo_w,
   SUM(centered_x*EXP(-10*centered_x) ORDER BY year) AS lo_x,
   SUM(EXP(10*centered_x) ORDER BY year) AS hi_w,
   SUM(centered_x*EXP(10*centered_x) ORDER BY year) AS hi_x
 FROM positive GROUP BY id
), bounds AS (
 SELECT CASE WHEN COUNT(*)<2 OR SUM(observed-total*lo_x/lo_w ORDER BY id)<=0
   OR SUM(observed-total*hi_x/hi_w ORDER BY id)>=0
   THEN error('conditional FE requires at least two positive-total firms and an interior score crossing in (-10,10)')
   ELSE COUNT(*) END AS n_groups FROM endpoint_units
), search(iter,lo,hi) AS (
 SELECT 0, CAST(-10 AS DOUBLE), CAST(10 AS DOUBLE) FROM bounds WHERE n_groups>=2
 UNION ALL
 SELECT iter+1, CASE WHEN score>0 THEN (lo+hi)/2 ELSE lo END,
   CASE WHEN score>0 THEN hi ELSE (lo+hi)/2 END
 FROM search CROSS JOIN LATERAL (
   SELECT SUM(observed-total*wx/w ORDER BY id) AS score FROM (
     SELECT id, MAX(firm_total) AS total, SUM(pat*centered_x ORDER BY year) AS observed,
       SUM(EXP(((lo+hi)/2)*centered_x) ORDER BY year) AS w,
       SUM(centered_x*EXP(((lo+hi)/2)*centered_x) ORDER BY year) AS wx
     FROM positive GROUP BY id
   )
 ) AS step WHERE iter<80
), parameter AS (SELECT (lo+hi)/2 AS beta FROM search WHERE iter=80),
weighted AS (
 SELECT checked.*, beta, EXP(beta*centered_x) AS weight,
   SUM(EXP(beta*centered_x) ORDER BY year) OVER (PARTITION BY id) AS denom,
   SUM(centered_x*EXP(beta*centered_x) ORDER BY year) OVER (PARTITION BY id)
     / SUM(EXP(beta*centered_x) ORDER BY year) OVER (PARTITION BY id) AS weighted_x
 FROM checked CROSS JOIN parameter
), unit_stats AS (
 SELECT id, MAX(firm_total) AS total,
   SUM(pat*(beta*centered_x-LN(denom))-LGAMMA(pat+1) ORDER BY year)+LGAMMA(MAX(firm_total)+1) AS conditional_ll,
   SUM(pat*centered_x ORDER BY year)-MAX(firm_total)*MAX(weighted_x) AS score,
   SUM(weight*(centered_x-weighted_x)^2/denom ORDER BY year)*MAX(firm_total) AS information
 FROM weighted WHERE firm_total>0 GROUP BY id
), aggregate AS (
 SELECT COUNT(*) AS retained_firms, SUM(conditional_ll ORDER BY id) AS conditional_ll,
   SUM(score ORDER BY id) AS score, SUM(ABS(score) ORDER BY id) AS score_scale,
   SUM(information ORDER BY id) AS information
 FROM unit_stats
), result AS (
 SELECT beta, retained_firms, conditional_ll, score,
   CASE WHEN information IS NULL OR NOT isfinite(information) OR information<=0
     OR conditional_ll IS NULL OR NOT isfinite(conditional_ll) OR score IS NULL OR NOT isfinite(score)
     OR score_scale IS NULL OR NOT isfinite(score_scale) OR ABS(score)>1e-10*(1+score_scale)
     THEN error('conditional FE requires finite likelihood, positive information and a converged score') ELSE information END AS information
 FROM aggregate CROSS JOIN parameter
), stats0 AS (
 SELECT *, SQRT(1/information) AS se FROM result
), stats AS (
 SELECT *, CASE WHEN se IS NULL OR NOT isfinite(se) OR se<=0 OR NOT isfinite(beta/se)
   THEN error('conditional FE normal inference must be finite') ELSE beta/se END AS z FROM stats0
), tails(iter,term,partial) AS (
 SELECT 0, ABS(z)/SQRT(2), ABS(z)/SQRT(2) FROM stats WHERE ABS(z)<=3
 UNION ALL
 SELECT iter+1, term*(-(z*z/2))*(2*iter+1)/((iter+1)*(2*iter+3)),
   partial+term*(-(z*z/2))*(2*iter+1)/((iter+1)*(2*iter+3))
 FROM tails CROSS JOIN stats WHERE iter<80
), tail AS (SELECT 1-2/SQRT(PI())*partial AS p_value FROM tails WHERE iter=80),
report AS (
 SELECT stats.*, 'conditional_poisson_fixed_effects_one_slope' AS estimator,
   'model_based_conditional_information_nonrobust' AS covariance,
   CASE WHEN ABS(z)<=3 THEN 'asymptotic_standard_normal' ELSE 'unavailable_abs_z_exceeds_3' END AS p_reference,
   CASE WHEN ABS(z)<=3 THEN CASE WHEN (SELECT p_value FROM tail) BETWEEN 0 AND 1
   THEN (SELECT p_value FROM tail) ELSE error('conditional FE normal tail is invalid') END ELSE NULL END AS p_value,
   beta-1.959963984540054*se AS ci_lower, beta+1.959963984540054*se AS ci_upper,
   (SELECT COUNT(*) FROM checked) AS candidate_rows,
   (SELECT COUNT(DISTINCT id) FROM checked) AS candidate_firms,
   (SELECT COUNT(*) FROM positive) AS retained_rows
 FROM stats
)
SELECT weighted.* EXCLUDE(firm_total,centered_x,beta,weight,denom,weighted_x),
  firm_total>0 AS included_in_fe,
  CASE WHEN firm_total>0 THEN firm_total*weight/denom ELSE NULL END AS fe_mean,
  CASE WHEN firm_total>0 THEN pat-firm_total*weight/denom ELSE NULL END AS fe_residual,
  report.*
FROM weighted CROSS JOIN report ORDER BY id,year
""" into conditional_fe_predictions
sql """SELECT DISTINCT candidate_rows,candidate_firms,retained_rows,retained_firms,candidate_rows-retained_rows AS excluded_rows,candidate_firms-retained_firms AS excluded_firms,estimator,covariance,p_reference,beta,se,z,p_value,ci_lower,ci_upper,conditional_ll,score,information FROM active"""
summarize fe_mean fe_residual
export conditional_fe_predictions.csv
