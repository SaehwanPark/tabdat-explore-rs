# Chapter 23 one-slope exponential-mean NLS; auxiliary GEE covariance, not a generic GEE.
use patents.csv
isid id year
sql """SELECT CASE WHEN COUNT(*)<=2 OR COUNT(*)>10000 OR COUNT(DISTINCT id)<2 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR pat IS NULL OR logr IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR NOT isfinite(pat) OR NOT isfinite(logr) OR id != floor(id) OR id < 1 OR id > 346 OR year != floor(year) OR year < 1975 OR year > 1979 OR pat < 0 OR pat != floor(pat) OR pat > 1000 OR ABS(logr)>20) > 0 THEN error('invalid patents firm/year/count/model data') ELSE COUNT(*) END AS candidate_rows, COUNT(DISTINCT id) AS firms FROM active"""
sql """SELECT CASE WHEN COUNT(*) = 0 OR MIN(n) != 5 OR MAX(n) != 5 THEN error('each firm requires all five years 1975–79') ELSE COUNT(*) END AS firms FROM (SELECT id, COUNT(*) AS n FROM active GROUP BY id)"""
# Native NLS reports RSS/(N-2) and approximate t; the SQL table below reports
# the author's distinct RSS/N working and firm-cluster GEE conventions.
nl pat = exp(a+b*logr), params(a b) start(2.5 0.5)
predict nls_mu, mean
predict nls_residual, residuals
sql """
WITH base AS (
 SELECT id,year,pat,logr,nls_mu,nls_residual,LN(nls_mu) AS ln_mu FROM active
), initial AS (
 SELECT COUNT(*) AS n, COUNT(DISTINCT id) AS g,
   REGR_INTERCEPT(ln_mu,logr) AS a, REGR_SLOPE(ln_mu,logr) AS b,
   SUM(nls_mu*nls_mu ORDER BY id,year) AS s00,
   SUM(nls_mu*nls_mu*logr ORDER BY id,year) AS s01,
   SUM(nls_residual*nls_residual ORDER BY id,year) AS rss,
   SUM(nls_mu*nls_residual ORDER BY id,year) AS score_a,
   SUM(nls_mu*logr*nls_residual ORDER BY id,year) AS score_b,
   MAX(ABS(ln_mu)) AS ln_scale,
   COUNT(*) FILTER (WHERE nls_mu IS NULL OR nls_residual IS NULL OR NOT isfinite(nls_mu)
     OR NOT isfinite(nls_residual) OR nls_mu<=0) AS bad
 FROM base
), centered AS (
 SELECT initial.*,s01/s00 AS xbar FROM initial
), second AS (
 SELECT centered.*,
   (SELECT SUM(nls_mu*nls_mu*(logr-xbar)*(logr-xbar) ORDER BY id,year) FROM base) AS sxx,
   (SELECT MAX(ABS(ln_mu-(a+b*logr))) FROM base) AS reconstruction_error
 FROM centered
), guarded AS (
 SELECT * EXCLUDE(sxx),CASE WHEN n<=2 OR g<2 OR bad>0 OR a IS NULL OR b IS NULL
   OR NOT isfinite(a) OR NOT isfinite(b) OR NOT isfinite(s00) OR s00<=0
   OR NOT isfinite(xbar) OR NOT isfinite(sxx) OR sxx<=0 OR NOT isfinite(rss) OR rss<=0
   OR NOT isfinite(score_a) OR NOT isfinite(score_b) OR NOT isfinite(reconstruction_error)
   OR reconstruction_error>1e-12*(1+ln_scale)
   THEN error('invalid NLS fitted relation or rank-deficient covariance') ELSE sxx END AS sxx
 FROM second
), scores AS (
 -- Centering the Jacobian avoids a cancellation-prone 2x2 determinant.
 SELECT base.id,SUM(nls_mu*nls_residual ORDER BY year) AS u0,
   SUM(nls_mu*(logr-xbar)*nls_residual ORDER BY year) AS u1,
   SUM(ABS(nls_mu*nls_residual) ORDER BY year) AS scale0,
   SUM(ABS(nls_mu*(logr-xbar)*nls_residual) ORDER BY year) AS scale1
 FROM base CROSS JOIN guarded GROUP BY base.id
), group_influence AS (
 SELECT id,u0/s00-xbar*u1/sxx AS influence_a,u1/sxx AS influence_b,
   scale0/s00+ABS(xbar)*scale1/sxx AS influence_scale_a,scale1/sxx AS influence_scale_b
 FROM scores CROSS JOIN guarded
), cluster_cov AS (
 SELECT (g::DOUBLE/(g-1))*SUM(influence_a*influence_a ORDER BY id) AS vaa,
   (g::DOUBLE/(g-1))*SUM(influence_a*influence_b ORDER BY id) AS vab,
   (g::DOUBLE/(g-1))*SUM(influence_b*influence_b ORDER BY id) AS vbb,
   (g::DOUBLE/(g-1))*SUM(influence_scale_a*influence_scale_a ORDER BY id) AS vaa_scale,
   (g::DOUBLE/(g-1))*SUM(influence_scale_b*influence_scale_b ORDER BY id) AS vbb_scale
 FROM group_influence CROSS JOIN guarded GROUP BY g
), covariance AS (
 SELECT guarded.*,
   (rss/n)*(1/s00+xbar*xbar/sxx) AS working_aa,
   -(rss/n)*xbar/sxx AS working_ab,(rss/n)/sxx AS working_bb,
   vaa AS cluster_aa,vab AS cluster_ab,vbb AS cluster_bb,vaa_scale,vbb_scale
 FROM guarded CROSS JOIN cluster_cov
), checked AS (
 -- Reject influence variance indistinguishable from accumulated score cancellation.
 SELECT * EXCLUDE(cluster_aa),CASE WHEN NOT isfinite(working_aa) OR NOT isfinite(working_ab)
   OR NOT isfinite(working_bb) OR working_aa<=0 OR working_bb<=0
   OR NOT isfinite(cluster_aa) OR NOT isfinite(cluster_ab) OR NOT isfinite(cluster_bb)
   OR NOT isfinite(vaa_scale) OR NOT isfinite(vbb_scale)
   OR cluster_aa<=8.077935669463161e-28*vaa_scale
   OR cluster_bb<=8.077935669463161e-28*vbb_scale
   THEN error('invalid NLS working or cluster covariance') ELSE cluster_aa END AS cluster_aa
 FROM covariance
)
SELECT n AS observations,g AS firms,a AS intercept,b AS slope,rss,score_a,score_b,
  working_aa,working_ab,working_bb,cluster_aa,cluster_ab,cluster_bb,
  SQRT(working_aa) AS working_se_intercept,SQRT(working_bb) AS working_se_slope,
  SQRT(cluster_aa) AS cluster_se_intercept,SQRT(cluster_bb) AS cluster_se_slope,
  a/SQRT(working_aa) AS working_z_intercept,b/SQRT(working_bb) AS working_z_slope,
  a/SQRT(cluster_aa) AS cluster_z_intercept,b/SQRT(cluster_bb) AS cluster_z_slope,
  a-1.959963984540054*SQRT(working_aa) AS working_lo_intercept,
  a+1.959963984540054*SQRT(working_aa) AS working_hi_intercept,
  b-1.959963984540054*SQRT(working_bb) AS working_lo_slope,
  b+1.959963984540054*SQRT(working_bb) AS working_hi_slope,
  a-1.959963984540054*SQRT(cluster_aa) AS cluster_lo_intercept,
  a+1.959963984540054*SQRT(cluster_aa) AS cluster_hi_intercept,
  b-1.959963984540054*SQRT(cluster_bb) AS cluster_lo_slope,
  b+1.959963984540054*SQRT(cluster_bb) AS cluster_hi_slope,
  'rss_over_n_working_gauss_newton' AS working_covariance,
  'firm_score_gauss_newton_G_over_G_minus_1' AS cluster_covariance,
  'asymptotic_normal; p-values not computed' AS inference
FROM checked
"""
export exponential_nls_predictions.csv
