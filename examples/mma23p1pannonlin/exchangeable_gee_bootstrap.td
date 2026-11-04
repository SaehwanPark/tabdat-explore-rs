# 500 distinct-copy whole-firm exchangeable Poisson GEE resamples.
use patents.csv
isid id year
sql """SELECT CASE WHEN COUNT(*) = 0 OR COUNT(DISTINCT id)<2 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR pat IS NULL OR logr IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR NOT isfinite(pat) OR NOT isfinite(logr) OR id != floor(id) OR id < 1 OR id > 346 OR ABS(logr) > 20 OR year != floor(year) OR year < 1975 OR year > 1979 OR pat < 0 OR pat != floor(pat) OR pat > 1000) > 0 THEN error('invalid patents firm/year/count/model data') ELSE COUNT(*) END AS observations, COUNT(DISTINCT id) AS firms FROM active"""
sql """SELECT CASE WHEN COUNT(*) = 0 OR MIN(n) != 5 OR MAX(n) != 5 THEN error('each firm requires all five years 1975–79') ELSE COUNT(*) END AS firms FROM (SELECT id, COUNT(*) AS n FROM active GROUP BY id)"""
poisson pat logr
predict pooled_xb, xb
summarize pooled_xb
sql """
WITH RECURSIVE
source AS (SELECT id,year,pat,logr,pooled_xb FROM active),
groups AS (
 SELECT id, ROW_NUMBER() OVER (ORDER BY id)-1 AS index0 FROM (SELECT DISTINCT id FROM source)
), params AS (
 SELECT CASE WHEN COUNT(*)<2 OR COUNT(*)>346 THEN error('bootstrap requires 2..346 firms') ELSE COUNT(*) END AS g,
   500::BIGINT AS b, 10001::UHUGEINT AS seed, 18446744073709551615::UHUGEINT AS mask
 FROM groups
),
-- Each replicate starts its own SplitMix64-expanded xoshiro256++ stream.
expansion AS (
 SELECT rep,k, ((seed+rep::UHUGEINT-1+k::UHUGEINT*11400714819323198485::UHUGEINT)&mask) AS z
 FROM params CROSS JOIN range(1,501) AS t(rep) CROSS JOIN range(1,5) AS q(k)
), mix1 AS (
 SELECT rep,k,(xor(z,z>>30)*13787848793156543929::UHUGEINT)&mask AS z FROM expansion CROSS JOIN params
), mix2 AS (
 SELECT rep,k,(xor(z,z>>27)*10723151780598845931::UHUGEINT)&mask AS z FROM mix1 CROSS JOIN params
), seeds AS (
 SELECT rep,MAX(xor(z,z>>31)) FILTER(WHERE k=1) AS s0,
   MAX(xor(z,z>>31)) FILTER(WHERE k=2) AS s1,
   MAX(xor(z,z>>31)) FILTER(WHERE k=3) AS s2,
   MAX(xor(z,z>>31)) FILTER(WHERE k=4) AS s3 FROM mix2 GROUP BY rep
), rng(rep,step,accepted,s0,s1,s2,s3,v) AS (
 SELECT rep,0::BIGINT,0::BIGINT,s0,s1,s2,s3,NULL::UHUGEINT FROM seeds
 UNION ALL
 SELECT rep,step+1,accepted+CASE WHEN v_next < zone THEN 1 ELSE 0 END,
   xor(s0,xor(s3,s1)), xor(s1,xor(s2,s0)), xor(xor(s2,s0),(s1<<17)&mask),
   ((xor(s3,s1)<<45)|(xor(s3,s1)>>19))&mask, v_next
 FROM rng CROSS JOIN params
 CROSS JOIN LATERAL (SELECT mask-(mask%g::UHUGEINT) AS zone, (s0+s3)&mask AS a) q
 CROSS JOIN LATERAL (SELECT ((((a<<23)|(a>>41))&mask)+s0)&mask AS v_next) v
 WHERE accepted<g AND step<g+1024
), rng_health AS (
 SELECT CASE WHEN COUNT(*)!=500 OR MIN(max_accepted)!=(SELECT g FROM params)
   THEN error('bootstrap RNG rejection work bound exhausted') ELSE COUNT(*) END AS valid_streams
 FROM (SELECT rep,MAX(accepted) AS max_accepted FROM rng GROUP BY rep)
), draws AS MATERIALIZED (
 SELECT rng.rep, rng.accepted AS position, rng.step, groups.id
 FROM rng CROSS JOIN params CROSS JOIN rng_health
 JOIN groups ON groups.index0=(rng.v%g::UHUGEINT)::BIGINT
 WHERE rng.v IS NOT NULL AND rng.v<mask-(mask%g::UHUGEINT) AND valid_streams=500
), frequencies AS MATERIALIZED (
 SELECT rep,id,COUNT(*) AS multiplicity FROM draws GROUP BY rep,id
 UNION ALL SELECT 0::BIGINT AS rep,id,1::BIGINT AS multiplicity FROM groups
), start_regression AS (
 SELECT REGR_INTERCEPT(pooled_xb,logr) AS a,REGR_SLOPE(pooled_xb,logr) AS b FROM source
), start AS (
 SELECT CASE WHEN a IS NULL OR b IS NULL OR NOT isfinite(a) OR NOT isfinite(b)
   OR COUNT(*) FILTER (WHERE pooled_xb IS NULL OR NOT isfinite(pooled_xb))>0
   OR MAX(ABS(pooled_xb-(a+b*logr)))>1e-12*(1+MAX(ABS(pooled_xb)))
   THEN error('GEE pooled-Poisson start cannot be reconstructed') ELSE a END AS a,b
 FROM source CROSS JOIN start_regression GROUP BY a,b
), search(rep,iter,a,b,last_step) AS (
 SELECT rep,0::BIGINT,a,b,1e100::DOUBLE FROM (SELECT DISTINCT rep FROM frequencies) r CROSS JOIN start
 UNION ALL
 SELECT rep,iter+1,a+delta_a,b+delta_b,
   GREATEST(ABS(delta_a)/(1+ABS(a)),ABS(delta_b)/(1+ABS(b)))
 FROM search CROSS JOIN LATERAL (
   WITH observation AS (
     SELECT f.rep,f.id,f.multiplicity,s.year,s.pat,s.logr,
       CASE WHEN a IS NULL OR b IS NULL OR NOT isfinite(a) OR NOT isfinite(b) OR ABS(a)>20 OR ABS(b)>20
         OR NOT isfinite(a+b*s.logr) OR ABS(a+b*s.logr)>40
         THEN error('GEE predictor or parameter outside numerical bounds') ELSE EXP(a+b*s.logr) END AS mu
     FROM frequencies f JOIN source s USING(id) WHERE f.rep=search.rep
   ), residual AS (
     SELECT *,SQRT(mu) AS root_mu,(pat-mu)/SQRT(mu) AS e,(ABS(pat)+mu)/SQRT(mu) AS e_scale FROM observation
   ), centered AS (
     SELECT *,e-AVG(e ORDER BY year) OVER (PARTITION BY rep,id) AS centered_e FROM residual
   ), unit AS (
     SELECT rep,id,MAX(multiplicity) AS multiplicity,SUM(e ORDER BY year) AS es,SUM(e*e ORDER BY year) AS e2,
       SUM(centered_e*centered_e ORDER BY year) AS centered_e2,SUM(e_scale*e_scale ORDER BY year) AS e2_scale,
       SUM(root_mu ORDER BY year) AS h0,SUM(root_mu*logr ORDER BY year) AS h1,
       SUM((pat-mu) ORDER BY year) AS d0,SUM(logr*(pat-mu) ORDER BY year) AS d1,
       SUM(mu ORDER BY year) AS q00,SUM(mu*logr ORDER BY year) AS q01,SUM(mu*logr*logr ORDER BY year) AS q11
     FROM centered GROUP BY rep,id
   ), moment AS (
     -- Multiplicity weights distinct five-row copies; correlation gaps are sums of squares.
     SELECT SUM(multiplicity*es*es ORDER BY id) AS lower_gap,SUM(multiplicity*5*centered_e2 ORDER BY id) AS upper_gap,
       SUM(multiplicity*e2 ORDER BY id) AS diag_e,SUM(multiplicity*e2_scale ORDER BY id) AS diag_scale FROM unit
   ), correlation AS (
     SELECT CASE WHEN NOT isfinite(diag_e) OR NOT isfinite(diag_scale)
       OR diag_e<=8.077935669463161e-28*diag_scale OR NOT isfinite(lower_gap) OR NOT isfinite(upper_gap)
       OR lower_gap<=2.842170943040401e-14*4*diag_e OR upper_gap<=2.842170943040401e-14*4*diag_e
       THEN error('GEE bootstrap Pearson variation or exchangeable correlation is numerically invalid')
       WHEN lower_gap>=diag_e THEN 1-upper_gap/(4*diag_e) ELSE -0.25+lower_gap/(4*diag_e) END AS rho FROM moment
   ), system AS (
     SELECT SUM(multiplicity*(d0-(rho/(1+4*rho))*h0*es)/(1-rho) ORDER BY id) AS u0,
       SUM(multiplicity*(d1-(rho/(1+4*rho))*h1*es)/(1-rho) ORDER BY id) AS u1,
       SUM(multiplicity*(q00-(rho/(1+4*rho))*h0*h0)/(1-rho) ORDER BY id) AS b00,
       SUM(multiplicity*(q01-(rho/(1+4*rho))*h0*h1)/(1-rho) ORDER BY id) AS b01,
       SUM(multiplicity*(q11-(rho/(1+4*rho))*h1*h1)/(1-rho) ORDER BY id) AS b11
     FROM unit CROSS JOIN correlation
   ), checked AS (
     SELECT *,CASE WHEN NOT isfinite(u0) OR NOT isfinite(u1) OR NOT isfinite(b00) OR b00<=0
       OR NOT isfinite(b01) OR NOT isfinite(b11) OR NOT isfinite(b11-b01*b01/b00)
       OR b11-b01*b01/b00<=2.842170943040401e-14*(1+ABS(b11)+ABS(b01*b01/b00))
       THEN error('GEE bootstrap working bread is singular or numerically unidentified') ELSE b11-b01*b01/b00 END AS schur
     FROM system
   )
   SELECT u0/b00-(b01/b00)*(u1-(b01/b00)*u0)/schur AS delta_a,
     (u1-(b01/b00)*u0)/schur AS delta_b FROM checked
 ) step
 WHERE iter<100 AND (iter=0 OR last_step>1e-12)
), ranked AS MATERIALIZED (
 SELECT *,ROW_NUMBER() OVER (PARTITION BY rep ORDER BY iter DESC) AS ordinal FROM search
), last_iteration AS MATERIALIZED (
 SELECT rep,iter,a,b,last_step FROM ranked WHERE ordinal=1
), final_parameter AS MATERIALIZED (
 SELECT rep,iter,CASE WHEN last_step IS NULL OR NOT isfinite(last_step) OR last_step>1e-12
   THEN error('GEE bootstrap did not converge within 100 full scoring steps') ELSE a END AS a,b,last_step
 FROM last_iteration
), final_observation AS MATERIALIZED (
 SELECT f.rep,f.id,f.multiplicity,s.year,s.pat,s.logr,p.a,p.b,p.iter,p.last_step,
   CASE WHEN p.a IS NULL OR p.b IS NULL OR NOT isfinite(p.a) OR NOT isfinite(p.b) OR ABS(p.a)>20 OR ABS(p.b)>20
     OR NOT isfinite(p.a+p.b*s.logr) OR ABS(p.a+p.b*s.logr)>40
     THEN error('GEE predictor or parameter outside numerical bounds') ELSE EXP(p.a+p.b*s.logr) END AS mu
 FROM final_parameter p JOIN frequencies f USING(rep) JOIN source s USING(id)
), final_residual AS (
 SELECT *,SQRT(mu) AS root_mu,(pat-mu)/SQRT(mu) AS e,(ABS(pat)+mu)/SQRT(mu) AS e_scale FROM final_observation
), final_centered AS (
 SELECT *,e-AVG(e ORDER BY year) OVER (PARTITION BY rep,id) AS centered_e FROM final_residual
), final_unit AS MATERIALIZED (
 SELECT rep,id,MAX(multiplicity) AS multiplicity,SUM(e ORDER BY year) AS es,SUM(e*e ORDER BY year) AS e2,
   SUM(centered_e*centered_e ORDER BY year) AS centered_e2,SUM(e_scale*e_scale ORDER BY year) AS e2_scale,
   SUM(root_mu ORDER BY year) AS h0,SUM(root_mu*logr ORDER BY year) AS h1,
   SUM((pat-mu) ORDER BY year) AS d0,SUM(logr*(pat-mu) ORDER BY year) AS d1,
   SUM(ABS(pat-mu) ORDER BY year) AS d0_scale,SUM(ABS(logr*(pat-mu)) ORDER BY year) AS d1_scale,
   SUM(mu ORDER BY year) AS q00,SUM(mu*logr ORDER BY year) AS q01,SUM(mu*logr*logr ORDER BY year) AS q11
 FROM final_centered GROUP BY rep,id
), final_moment AS (
 SELECT rep,SUM(multiplicity*es*es ORDER BY id) AS lower_gap,SUM(multiplicity*5*centered_e2 ORDER BY id) AS upper_gap,
   SUM(multiplicity*e2 ORDER BY id) AS diag_e,SUM(multiplicity*e2_scale ORDER BY id) AS diag_scale FROM final_unit GROUP BY rep
), final_correlation AS (
 SELECT rep,CASE WHEN NOT isfinite(diag_e) OR NOT isfinite(diag_scale)
   OR diag_e<=8.077935669463161e-28*diag_scale OR NOT isfinite(lower_gap) OR NOT isfinite(upper_gap)
   OR lower_gap<=2.842170943040401e-14*4*diag_e OR upper_gap<=2.842170943040401e-14*4*diag_e
   THEN error('GEE bootstrap Pearson variation or exchangeable correlation is numerically invalid')
   WHEN lower_gap>=diag_e THEN 1-upper_gap/(4*diag_e) ELSE -0.25+lower_gap/(4*diag_e) END AS rho FROM final_moment
), final_system AS (
 SELECT u.rep,MAX(p.a) AS a,MAX(p.b) AS b,MAX(p.iter) AS iterations,MAX(p.last_step) AS last_step,MAX(c.rho) AS rho,
   SUM(multiplicity*(d0-(c.rho/(1+4*c.rho))*h0*es)/(1-c.rho) ORDER BY id) AS u0,
   SUM(multiplicity*(d1-(c.rho/(1+4*c.rho))*h1*es)/(1-c.rho) ORDER BY id) AS u1,
   SUM(multiplicity*(d0_scale+ABS((c.rho/(1+4*c.rho))*h0*es))/(1-c.rho) ORDER BY id) AS scale0,
   SUM(multiplicity*(d1_scale+ABS((c.rho/(1+4*c.rho))*h1*es))/(1-c.rho) ORDER BY id) AS scale1,
   SUM(multiplicity*(q00-(c.rho/(1+4*c.rho))*h0*h0)/(1-c.rho) ORDER BY id) AS b00,
   SUM(multiplicity*(q01-(c.rho/(1+4*c.rho))*h0*h1)/(1-c.rho) ORDER BY id) AS b01,
   SUM(multiplicity*(q11-(c.rho/(1+4*c.rho))*h1*h1)/(1-c.rho) ORDER BY id) AS b11
 FROM final_unit u JOIN final_correlation c USING(rep) JOIN final_parameter p USING(rep) GROUP BY u.rep
), final_checked AS (
 SELECT *,CASE WHEN NOT isfinite(u0) OR NOT isfinite(u1) OR NOT isfinite(scale0) OR NOT isfinite(scale1)
   OR ABS(u0)>1e-10*(1+scale0) OR ABS(u1)>1e-10*(1+scale1)
   OR NOT isfinite(b00) OR b00<=0 OR NOT isfinite(b01) OR NOT isfinite(b11)
   OR NOT isfinite(b11-b01*b01/b00)
   OR b11-b01*b01/b00<=2.842170943040401e-14*(1+ABS(b11)+ABS(b01*b01/b00))
   THEN error('GEE bootstrap final score or working bread is numerically invalid') ELSE b11-b01*b01/b00 END AS schur
 FROM final_system
), bread_inverse AS (
 SELECT *,1/b00+POWER(b01/b00,2)/schur AS w00,-(b01/b00)/schur AS w01,1/schur AS w11 FROM final_checked
), fits AS MATERIALIZED (
 SELECT rep,a AS alpha,CASE WHEN NOT isfinite(w00) OR NOT isfinite(w01) OR NOT isfinite(w11)
   OR 2.842170943040401e-14*(ABS(w00)*scale0+ABS(w01)*scale1)>=1e-9+1e-10*ABS(a)
   OR 2.842170943040401e-14*(ABS(w01)*scale0+ABS(w11)*scale1)>=1e-9+1e-10*ABS(b)
   THEN error('GEE bootstrap coefficient is sensitive to score cancellation') ELSE b END AS beta,
   rho,iterations,last_step
 FROM bread_inverse
), summary AS (
 SELECT COUNT(*) AS replications,STDDEV_SAMP(alpha ORDER BY rep) AS se_a,
   STDDEV_SAMP(beta ORDER BY rep) AS se_b,COVAR_SAMP(alpha,beta ORDER BY rep) AS cov_ab
 FROM fits WHERE rep>0
), original_predictions AS (
 SELECT AVG(EXP(o.alpha+o.beta*s.logr) ORDER BY s.id,s.year) AS original_mu_mean,
   MIN(EXP(o.alpha+o.beta*s.logr)) AS original_mu_min,MAX(EXP(o.alpha+o.beta*s.logr)) AS original_mu_max,
   AVG(s.pat-EXP(o.alpha+o.beta*s.logr) ORDER BY s.id,s.year) AS original_residual_mean
 FROM source s CROSS JOIN (SELECT alpha,beta FROM fits WHERE rep=0) o
), result AS (
 SELECT CASE WHEN replications!=500 OR se_a IS NULL OR NOT isfinite(se_a)
   OR se_b IS NULL OR NOT isfinite(se_b) OR cov_ab IS NULL OR NOT isfinite(cov_ab)
   OR o.alpha IS NULL OR o.beta IS NULL OR o.rho IS NULL OR NOT isfinite(o.alpha) OR NOT isfinite(o.beta) OR NOT isfinite(o.rho)
   OR original_mu_mean IS NULL OR NOT isfinite(original_mu_mean) OR original_residual_mean IS NULL OR NOT isfinite(original_residual_mean)
   THEN error('GEE bootstrap requires 500 valid replicates and an original fit') ELSE replications END AS replications,
   o.alpha AS estimate_a,o.beta AS estimate_b,o.rho AS estimate_rho,p.*
 FROM summary LEFT JOIN (SELECT alpha,beta,rho FROM fits WHERE rep=0) o ON TRUE CROSS JOIN original_predictions p
)
SELECT draws.rep AS replicate,draws.position,draws.step,draws.id AS firm_id,
  fits.alpha AS replicate_intercept,fits.beta AS replicate_slope,fits.rho AS replicate_rho,
  fits.iterations AS replicate_iterations,fits.last_step AS replicate_last_step,
  result.estimate_a,result.estimate_b,result.estimate_rho,result.original_mu_mean,result.original_mu_min,result.original_mu_max,
  result.original_residual_mean,result.replications,(SELECT g FROM params) AS firms_per_draw,(SELECT COUNT(*) FROM source) AS candidate_rows
FROM draws LEFT JOIN fits ON draws.rep=fits.rep CROSS JOIN result
ORDER BY replicate,position
""" into exchangeable_gee_bootstrap_draws
sql """WITH r AS (SELECT DISTINCT replicate,replicate_intercept AS a,replicate_slope AS b,estimate_a,estimate_b,estimate_rho,original_mu_mean,original_mu_min,original_mu_max,original_residual_mean,replications,firms_per_draw,candidate_rows FROM active) SELECT MAX(candidate_rows) AS candidate_rows,MAX(firms_per_draw) AS firms_per_draw,COUNT(*) AS replications,MAX(estimate_a) AS estimate_intercept,MAX(estimate_b) AS estimate_slope,MAX(estimate_rho) AS estimate_rho,MAX(original_mu_mean) AS original_mu_mean,MAX(original_mu_min) AS original_mu_min,MAX(original_mu_max) AS original_mu_max,MAX(original_residual_mean) AS original_residual_mean,AVG(a ORDER BY replicate) AS mean_intercept,AVG(b ORDER BY replicate) AS mean_slope,AVG(a ORDER BY replicate)-MAX(estimate_a) AS bias_intercept,AVG(b ORDER BY replicate)-MAX(estimate_b) AS bias_slope,VAR_SAMP(a ORDER BY replicate) AS covariance_aa,COVAR_SAMP(a,b ORDER BY replicate) AS covariance_ab,VAR_SAMP(b ORDER BY replicate) AS covariance_bb,STDDEV_SAMP(a ORDER BY replicate) AS se_intercept,STDDEV_SAMP(b ORDER BY replicate) AS se_slope,QUANTILE_CONT(a,0.025 ORDER BY replicate) AS lower_intercept,QUANTILE_CONT(a,0.975 ORDER BY replicate) AS upper_intercept,QUANTILE_CONT(b,0.025 ORDER BY replicate) AS lower_slope,QUANTILE_CONT(b,0.975 ORDER BY replicate) AS upper_slope,10001 AS base_seed,'xoshiro256++/splitmix64/per_replicate_seed_plus_index/u64_rejection_v1' AS rng,'whole_firm_pairs_distinct_copies' AS method,'type7_percentile_95' AS interval_method FROM r"""
export exchangeable_gee_bootstrap_draws.csv
