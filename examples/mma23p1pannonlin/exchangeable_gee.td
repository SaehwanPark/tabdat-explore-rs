# Balanced five-year exchangeable Poisson GEE; SQL composition, not a native xtgee.
use patents.csv
isid id year
sql """SELECT CASE WHEN COUNT(*) = 0 OR COUNT(DISTINCT id)<2 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR pat IS NULL OR logr IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR NOT isfinite(pat) OR NOT isfinite(logr) OR id != floor(id) OR id < 1 OR id > 346 OR ABS(logr) > 20 OR year != floor(year) OR year < 1975 OR year > 1979 OR pat < 0 OR pat != floor(pat) OR pat > 1000) > 0 THEN error('invalid patents firm/year/count/model data') ELSE COUNT(*) END AS observations, COUNT(DISTINCT id) AS firms FROM active"""
sql """SELECT CASE WHEN COUNT(*) = 0 OR MIN(n) != 5 OR MAX(n) != 5 THEN error('each firm requires all five years 1975–79') ELSE COUNT(*) END AS firms FROM (SELECT id, COUNT(*) AS n FROM active GROUP BY id)"""
# Use the existing pooled Poisson fit only as an initial value, not as GEE state.
poisson pat logr
predict pooled_xb, xb
sql """
WITH RECURSIVE
start AS (
 SELECT REGR_INTERCEPT(pooled_xb,logr) AS a,REGR_SLOPE(pooled_xb,logr) AS b FROM active
), search(iter,a,b,last_step) AS (
 SELECT 0::BIGINT,a,b,1e100::DOUBLE FROM start
 UNION ALL
 SELECT iter+1,a+delta_a,b+delta_b,
   GREATEST(ABS(delta_a)/(1+ABS(a)),ABS(delta_b)/(1+ABS(b)))
 FROM search CROSS JOIN LATERAL (
   WITH observation AS (
     SELECT id,year,pat,logr,
       CASE WHEN a IS NULL OR b IS NULL OR NOT isfinite(a) OR NOT isfinite(b) OR ABS(a)>20 OR ABS(b)>20
         OR NOT isfinite(a+b*logr) OR ABS(a+b*logr)>40
         THEN error('GEE predictor or parameter outside numerical bounds') ELSE EXP(a+b*logr) END AS mu
     FROM active
   ), residual AS (
     SELECT *,SQRT(mu) AS root_mu,(pat-mu)/SQRT(mu) AS e,(ABS(pat)+mu)/SQRT(mu) AS e_scale FROM observation
   ), centered AS (
     SELECT *,e-AVG(e ORDER BY year) OVER (PARTITION BY id) AS centered_e FROM residual
   ), unit AS (
     SELECT id,SUM(e ORDER BY year) AS es,SUM(e*e ORDER BY year) AS e2,
       SUM(centered_e*centered_e ORDER BY year) AS centered_e2,
       SUM(e_scale*e_scale ORDER BY year) AS e2_scale,
       SUM(root_mu ORDER BY year) AS h0,SUM(root_mu*logr ORDER BY year) AS h1,
       SUM((pat-mu) ORDER BY year) AS d0,SUM(logr*(pat-mu) ORDER BY year) AS d1,
       SUM(mu ORDER BY year) AS q00,SUM(mu*logr ORDER BY year) AS q01,SUM(mu*logr*logr ORDER BY year) AS q11
     FROM centered GROUP BY id
   ), moment AS (
     -- Both correlation-boundary gaps are sums of squares, not differences of nearly equal moments.
     SELECT SUM(es*es ORDER BY id) AS lower_gap,SUM(5*centered_e2 ORDER BY id) AS upper_gap,
       SUM(e2 ORDER BY id) AS diag_e,SUM(e2_scale ORDER BY id) AS diag_scale FROM unit
   ), correlation AS (
     SELECT CASE WHEN NOT isfinite(diag_e) OR NOT isfinite(diag_scale)
       OR diag_e<=8.077935669463161e-28*diag_scale
       OR NOT isfinite(lower_gap) OR NOT isfinite(upper_gap)
       OR lower_gap<=2.842170943040401e-14*4*diag_e
       OR upper_gap<=2.842170943040401e-14*4*diag_e
       THEN error('GEE Pearson variation or exchangeable correlation is numerically invalid')
       WHEN lower_gap>=diag_e THEN 1-upper_gap/(4*diag_e)
       ELSE -0.25+lower_gap/(4*diag_e) END AS rho FROM moment
   ), system AS (
     SELECT SUM((d0-(rho/(1+4*rho))*h0*es)/(1-rho) ORDER BY id) AS u0,
       SUM((d1-(rho/(1+4*rho))*h1*es)/(1-rho) ORDER BY id) AS u1,
       SUM((q00-(rho/(1+4*rho))*h0*h0)/(1-rho) ORDER BY id) AS b00,
       SUM((q01-(rho/(1+4*rho))*h0*h1)/(1-rho) ORDER BY id) AS b01,
       SUM((q11-(rho/(1+4*rho))*h1*h1)/(1-rho) ORDER BY id) AS b11
     FROM unit CROSS JOIN correlation
   ), checked AS (
     SELECT *,CASE WHEN NOT isfinite(u0) OR NOT isfinite(u1) OR NOT isfinite(b00) OR b00<=0
       OR NOT isfinite(b01) OR NOT isfinite(b11) OR NOT isfinite(b11-b01*b01/b00)
       OR b11-b01*b01/b00<=2.842170943040401e-14*(1+ABS(b11)+ABS(b01*b01/b00))
       THEN error('GEE working bread is singular or numerically unidentified') ELSE b11-b01*b01/b00 END AS schur
     FROM system
   )
   SELECT u0/b00-(b01/b00)*(u1-(b01/b00)*u0)/schur AS delta_a,
     (u1-(b01/b00)*u0)/schur AS delta_b FROM checked
 ) step
 WHERE iter<100 AND (iter=0 OR last_step>1e-12)
), final_parameter AS MATERIALIZED (
 SELECT CASE WHEN last_step IS NULL OR NOT isfinite(last_step) OR last_step>1e-12
   THEN error('GEE did not converge within 100 full scoring steps') ELSE iter END AS iterations,a,b,last_step
 FROM (SELECT * FROM search ORDER BY iter DESC LIMIT 1) AS last_iteration
), final_observation AS MATERIALIZED (
 SELECT active.*,a,b,iterations,last_step,
   CASE WHEN a IS NULL OR b IS NULL OR NOT isfinite(a) OR NOT isfinite(b) OR ABS(a)>20 OR ABS(b)>20
     OR NOT isfinite(a+b*logr) OR ABS(a+b*logr)>40
     THEN error('GEE predictor or parameter outside numerical bounds') ELSE EXP(a+b*logr) END AS gee_mu
 FROM active CROSS JOIN final_parameter
), final_residual AS (
 SELECT *,pat-gee_mu AS gee_residual,SQRT(gee_mu) AS root_mu,
   (pat-gee_mu)/SQRT(gee_mu) AS e,(ABS(pat)+gee_mu)/SQRT(gee_mu) AS e_scale
 FROM final_observation
), final_centered AS (
 SELECT *,e-AVG(e ORDER BY year) OVER (PARTITION BY id) AS centered_e FROM final_residual
), final_unit AS MATERIALIZED (
 SELECT id,SUM(e ORDER BY year) AS es,SUM(e*e ORDER BY year) AS e2,
   SUM(centered_e*centered_e ORDER BY year) AS centered_e2,
   SUM(e_scale*e_scale ORDER BY year) AS e2_scale,
   SUM(root_mu ORDER BY year) AS h0,SUM(root_mu*logr ORDER BY year) AS h1,
   SUM(gee_residual ORDER BY year) AS d0,SUM(logr*gee_residual ORDER BY year) AS d1,
   SUM(gee_mu ORDER BY year) AS q00,SUM(gee_mu*logr ORDER BY year) AS q01,SUM(gee_mu*logr*logr ORDER BY year) AS q11
 FROM final_centered GROUP BY id
), final_moment AS (
 -- Recompute at the final coefficients; do not report the previous iteration's rho.
 SELECT SUM(es*es ORDER BY id) AS lower_gap,SUM(5*centered_e2 ORDER BY id) AS upper_gap,
   SUM(e2 ORDER BY id) AS diag_e,SUM(e2_scale ORDER BY id) AS diag_scale FROM final_unit
), final_correlation AS (
 SELECT CASE WHEN NOT isfinite(diag_e) OR NOT isfinite(diag_scale)
   OR diag_e<=8.077935669463161e-28*diag_scale
   OR NOT isfinite(lower_gap) OR NOT isfinite(upper_gap)
   OR lower_gap<=2.842170943040401e-14*4*diag_e
   OR upper_gap<=2.842170943040401e-14*4*diag_e
   THEN error('GEE Pearson variation or exchangeable correlation is numerically invalid')
   WHEN lower_gap>=diag_e THEN 1-upper_gap/(4*diag_e)
   ELSE -0.25+lower_gap/(4*diag_e) END AS rho FROM final_moment
), final_scores AS MATERIALIZED (
 SELECT id,rho,
   (d0-(rho/(1+4*rho))*h0*es)/(1-rho) AS u0,
   (d1-(rho/(1+4*rho))*h1*es)/(1-rho) AS u1,
   (ABS(d0)+ABS((rho/(1+4*rho))*h0*es))/(1-rho) AS scale0,
   (ABS(d1)+ABS((rho/(1+4*rho))*h1*es))/(1-rho) AS scale1,
   (q00-(rho/(1+4*rho))*h0*h0)/(1-rho) AS b00,
   (q01-(rho/(1+4*rho))*h0*h1)/(1-rho) AS b01,
   (q11-(rho/(1+4*rho))*h1*h1)/(1-rho) AS b11
 FROM final_unit CROSS JOIN final_correlation
), final_system AS (
 SELECT COUNT(*) AS firms,MAX(rho) AS rho,SUM(u0 ORDER BY id) AS u0,SUM(u1 ORDER BY id) AS u1,
   SUM(scale0 ORDER BY id) AS scale0,SUM(scale1 ORDER BY id) AS scale1,
   SUM(b00 ORDER BY id) AS b00,SUM(b01 ORDER BY id) AS b01,SUM(b11 ORDER BY id) AS b11
 FROM final_scores
), final_checked AS (
 SELECT *,CASE WHEN firms<2 OR NOT isfinite(u0) OR NOT isfinite(u1) OR NOT isfinite(scale0) OR NOT isfinite(scale1)
   OR ABS(u0)>1e-10*(1+scale0) OR ABS(u1)>1e-10*(1+scale1)
   OR NOT isfinite(b00) OR b00<=0 OR NOT isfinite(b01) OR NOT isfinite(b11)
   OR NOT isfinite(b11-b01*b01/b00)
   OR b11-b01*b01/b00<=2.842170943040401e-14*(1+ABS(b11)+ABS(b01*b01/b00))
   THEN error('GEE final score or working bread is numerically invalid') ELSE b11-b01*b01/b00 END AS schur
 FROM final_system
), bread_inverse AS (
 SELECT *,1/b00+POWER(b01/b00,2)/schur AS w00,-(b01/b00)/schur AS w01,1/schur AS w11 FROM final_checked
), influences AS (
 SELECT f.id,b.firms,b.w00*f.u0+b.w01*f.u1 AS i0,b.w01*f.u0+b.w11*f.u1 AS i1,
   ABS(b.w00)*f.scale0+ABS(b.w01)*f.scale1 AS iscale0,
   ABS(b.w01)*f.scale0+ABS(b.w11)*f.scale1 AS iscale1
 FROM final_scores f CROSS JOIN bread_inverse b
), sandwich AS (
 SELECT firms::DOUBLE/(firms-1)*SUM(i0*i0 ORDER BY id) AS c00,
   firms::DOUBLE/(firms-1)*SUM(i0*i1 ORDER BY id) AS c01,
   firms::DOUBLE/(firms-1)*SUM(i1*i1 ORDER BY id) AS c11,
   firms::DOUBLE/(firms-1)*SUM(iscale0*iscale0 ORDER BY id) AS c00_scale,
   firms::DOUBLE/(firms-1)*SUM(iscale1*iscale1 ORDER BY id) AS c11_scale
 FROM influences GROUP BY firms
), checked_covariance AS (
 SELECT b.*,c01,c11,CASE WHEN NOT isfinite(w00) OR NOT isfinite(w01) OR NOT isfinite(w11)
   OR w00<=0 OR w11<=0 OR NOT isfinite(c00) OR NOT isfinite(c01) OR NOT isfinite(c11)
   OR NOT isfinite(c00_scale) OR NOT isfinite(c11_scale)
   OR c00<=8.077935669463161e-28*c00_scale OR c11<=8.077935669463161e-28*c11_scale
   THEN error('GEE working or firm covariance is numerically invalid') ELSE c00 END AS c00
 FROM bread_inverse b CROSS JOIN sandwich
)
SELECT r.* EXCLUDE(root_mu,e,e_scale),r.a AS intercept,r.b AS slope,c.rho,c.firms,
  c.u0 AS score_intercept,c.u1 AS score_slope,c.w00 AS working_aa,c.w01 AS working_ab,c.w11 AS working_bb,
  c.c00 AS cluster_aa,c.c01 AS cluster_ab,c.c11 AS cluster_bb,
  SQRT(c.w00) AS working_se_intercept,SQRT(c.w11) AS working_se_slope,
  SQRT(c.c00) AS cluster_se_intercept,SQRT(c.c11) AS cluster_se_slope,
  r.a/SQRT(c.w00) AS working_z_intercept,r.b/SQRT(c.w11) AS working_z_slope,
  r.a/SQRT(c.c00) AS cluster_z_intercept,r.b/SQRT(c.c11) AS cluster_z_slope,
  r.a-1.959963984540054*SQRT(c.w00) AS working_lo_intercept,
  r.a+1.959963984540054*SQRT(c.w00) AS working_hi_intercept,
  r.b-1.959963984540054*SQRT(c.w11) AS working_lo_slope,
  r.b+1.959963984540054*SQRT(c.w11) AS working_hi_slope,
  r.a-1.959963984540054*SQRT(c.c00) AS cluster_lo_intercept,
  r.a+1.959963984540054*SQRT(c.c00) AS cluster_hi_intercept,
  r.b-1.959963984540054*SQRT(c.c11) AS cluster_lo_slope,
  r.b+1.959963984540054*SQRT(c.c11) AS cluster_hi_slope,
  'exchangeable_pearson_legacy_moment_fixed_scale_1' AS correlation,
  'gee_working_scale_1' AS working_covariance,'firm_score_G_over_G_minus_1' AS cluster_covariance,
  'asymptotic_normal; p-values not computed' AS inference
FROM final_residual r CROSS JOIN checked_covariance c
ORDER BY id,year
""" into exchangeable_gee_predictions
sql """SELECT COUNT(*) AS observations,MAX(firms) AS firms,MAX(intercept) AS intercept,MAX(slope) AS slope,MAX(rho) AS rho,MAX(iterations) AS iterations,MAX(last_step) AS last_step,MAX(score_intercept) AS score_intercept,MAX(score_slope) AS score_slope,MAX(working_aa) AS working_aa,MAX(working_ab) AS working_ab,MAX(working_bb) AS working_bb,MAX(cluster_aa) AS cluster_aa,MAX(cluster_ab) AS cluster_ab,MAX(cluster_bb) AS cluster_bb,MAX(working_se_intercept) AS working_se_intercept,MAX(working_se_slope) AS working_se_slope,MAX(cluster_se_intercept) AS cluster_se_intercept,MAX(cluster_se_slope) AS cluster_se_slope,MAX(working_z_intercept) AS working_z_intercept,MAX(working_z_slope) AS working_z_slope,MAX(cluster_z_intercept) AS cluster_z_intercept,MAX(cluster_z_slope) AS cluster_z_slope,MAX(working_lo_intercept) AS working_lo_intercept,MAX(working_hi_intercept) AS working_hi_intercept,MAX(working_lo_slope) AS working_lo_slope,MAX(working_hi_slope) AS working_hi_slope,MAX(cluster_lo_intercept) AS cluster_lo_intercept,MAX(cluster_hi_intercept) AS cluster_hi_intercept,MAX(cluster_lo_slope) AS cluster_lo_slope,MAX(cluster_hi_slope) AS cluster_hi_slope,MAX(correlation) AS correlation,MAX(working_covariance) AS working_covariance,MAX(cluster_covariance) AS cluster_covariance,MAX(inference) AS inference FROM active"""
export exchangeable_gee_predictions.csv
