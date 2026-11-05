# Manual block6: whole-person pairs bootstrap of fixed original ML-theta quasi OLS.
use MOM.csv
isid id year
isid source_row
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(*)>5320 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR source_row IS NULL OR lnhr IS NULL OR lnwg IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR NOT isfinite(source_row) OR NOT isfinite(lnhr) OR NOT isfinite(lnwg) OR id!=floor(id) OR id<1 OR id>532 OR year!=floor(year) OR year<1979 OR year>1988 OR source_row!=floor(source_row) OR source_row<1 OR source_row>5320 OR ABS(lnhr)>20 OR ABS(lnwg)>20)>0 THEN error('manual ML comparison requires finite bounded MOM model fields and exact keys') ELSE COUNT(*) END AS observations FROM active"""
sql """SELECT CASE WHEN COUNT(*)<3 OR COUNT(*)>532 OR MIN(t)!=10 OR MAX(t)!=10 THEN error('manual ML comparison requires complete ten-year people') ELSE COUNT(*) END AS people FROM (SELECT id,COUNT(*) AS t FROM active GROUP BY id)"""
sql """SELECT 'Bootstrap resamples complete people using frozen original ML theta and original person means; it does not refit ML variance components. Quasi intercept equals (1-theta) times population intercept. B500 seed10001 uses the versioned native RNG, not the historical Stata stream.' AS bootstrap_scope"""
xtreg lnhr lnwg, mle id(id) time(year)
predict population_xb, xb
predict population_residual, residuals
sql """
WITH means AS (
  SELECT *,AVG(lnhr ORDER BY year) OVER (PARTITION BY id) AS mean_lnhr,
    AVG(lnwg ORDER BY year) OVER (PARTITION BY id) AS mean_lnwg,
    AVG(population_residual ORDER BY year) OVER (PARTITION BY id) AS mean_residual FROM active
), moments AS (
  -- At the fitted ML coefficients, the balanced likelihood score gives
  -- e=sw/(N-G) and v=sb/G; when v<=e the constrained optimum is v=e.
  SELECT COUNT(*) AS n,COUNT(DISTINCT id) AS g,
    SUM((population_residual-mean_residual)^2 ORDER BY source_row) AS sw,
    SUM(mean_residual^2 ORDER BY source_row) AS sb FROM means
), candidate AS (
  SELECT *,sw/NULLIF(n-g,0) AS interior_e,sb/NULLIF(g,0) AS interior_v FROM moments
), components AS (
  SELECT CASE WHEN interior_v>interior_e THEN interior_e ELSE (sw+sb)/n END AS e,
    CASE WHEN interior_v>interior_e THEN interior_v ELSE (sw+sb)/n END AS v FROM candidate
), parameters AS (
  SELECT CASE WHEN e IS NULL OR v IS NULL OR NOT isfinite(e) OR NOT isfinite(v)
    OR e<=0 OR v<e THEN error('manual ML requires positive finite variance and identified ML solution') ELSE e END AS sigma_e_squared,
    (v-e)/10 AS sigma_u_squared,1-sqrt(e/v) AS theta FROM components
)
SELECT means.* EXCLUDE(mean_residual),lnhr-mean_lnhr AS within_lnhr,lnwg-mean_lnwg AS within_lnwg,
  sigma_e_squared,sigma_u_squared,theta,lnhr-theta*mean_lnhr AS quasi_lnhr,lnwg-theta*mean_lnwg AS quasi_lnwg
FROM means CROSS JOIN parameters ORDER BY source_row
""" into manual_ml_bootstrap_frame
regress quasi_lnhr quasi_lnwg
bootstrap quasi_lnhr quasi_lnwg, cluster(id) reps(500) seed(10001)
sql """SELECT 'Percentile intervals and B-1 covariance are conditional on the original transformed design; independent people and sufficient cluster asymptotics are required. Failed draws abort without retries. Predictions below belong to the original quasi OLS, not any bootstrap replicate.' AS bootstrap_limits"""
predict quasi_fitted, xb
predict quasi_residual, residuals
summarize theta sigma_e_squared sigma_u_squared quasi_fitted quasi_residual
export panel_manual_ml_bootstrap_predictions.csv
