# WIP: distinct-copy whole-firm shared-gamma Poisson bootstrap.
use patents.csv
isid id year
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(DISTINCT id)<2 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR pat IS NULL OR logr IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR NOT isfinite(pat) OR NOT isfinite(logr) OR id!=floor(id) OR id<1 OR id>346 OR year!=floor(year) OR year<1975 OR year>1979 OR pat<0 OR pat!=floor(pat) OR pat>1000 OR ABS(logr)>20)>0 THEN error('invalid gamma RE panel data') ELSE COUNT(*) END AS observations,COUNT(DISTINCT id) AS firms FROM active"""
sql """SELECT CASE WHEN COUNT(*)=0 OR MIN(n)!=5 OR MAX(n)!=5 THEN error('each firm requires all five years 1975–79') ELSE COUNT(*) END AS firms FROM (SELECT id,COUNT(*) AS n FROM active GROUP BY id)"""
poisson pat logr
predict pooled_xb, xb
sql """
WITH RECURSIVE
source AS (SELECT * FROM active),
groups AS (SELECT id,ROW_NUMBER() OVER (ORDER BY id)-1 AS index0 FROM (SELECT DISTINCT id FROM source)),
params AS (
 SELECT CASE WHEN COUNT(*)<2 OR COUNT(*)>346 THEN error('bootstrap requires 2..346 firms') ELSE COUNT(*) END AS g,
   500::BIGINT AS b,10001::UHUGEINT AS seed,18446744073709551615::UHUGEINT AS mask FROM groups
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
   THEN error('gamma RE pooled start cannot be reconstructed') ELSE a END AS a,b
 FROM source CROSS JOIN start_regression GROUP BY a,b
), search(rep,iter,a,b,t,g,old_a,old_b,old_t,u0,u1,u2,s0,s1,s2,i00,i01,i02,i11,i12,i22,ll,last_step) AS (
 SELECT rep,0::BIGINT,start.a,start.b,0.0::DOUBLE,g,NULL::DOUBLE,NULL::DOUBLE,NULL::DOUBLE,
   NULL::DOUBLE,NULL::DOUBLE,NULL::DOUBLE,NULL::DOUBLE,NULL::DOUBLE,NULL::DOUBLE,
   NULL::DOUBLE,NULL::DOUBLE,NULL::DOUBLE,NULL::DOUBLE,NULL::DOUBLE,NULL::DOUBLE,
   NULL::DOUBLE,1e100::DOUBLE FROM (SELECT DISTINCT rep FROM frequencies) r CROSS JOIN start CROSS JOIN params
 UNION ALL
 SELECT rep,iter+1,a+da,b+db,t+dt,g,a,b,t,step.u0,step.u1,step.u2,step.s0,step.s1,step.s2,step.i00,step.i01,step.i02,step.i11,step.i12,step.i22,step.ll,
   GREATEST(ABS(da)/(1+ABS(a)),ABS(db)/(1+ABS(b)),ABS(dt)/(1+ABS(t)))
 FROM search CROSS JOIN LATERAL (
   WITH observation AS (
     SELECT f.rep,f.id,f.multiplicity,s.year,s.pat,s.logr,
       CASE WHEN a IS NULL OR b IS NULL OR t IS NULL OR NOT isfinite(a) OR NOT isfinite(b) OR NOT isfinite(t)
         OR ABS(a)>20 OR ABS(b)>20 OR ABS(t)>=8 OR NOT isfinite(a+b*s.logr) OR ABS(a+b*s.logr)>40
         THEN error('gamma RE bootstrap parameters or predictor outside numerical bounds') ELSE EXP(a+b*s.logr) END AS mu
     FROM frequencies f JOIN source s USING(id) WHERE f.rep=search.rep
   ), panel AS (
     SELECT rep,id,MAX(multiplicity) AS multiplicity,COUNT(*) AS panel_n,
       SUM(pat ORDER BY year) AS y_total,SUM(pat*logr ORDER BY year) AS yx,
       SUM(ABS(pat*logr) ORDER BY year) AS yx_scale,SUM(lgamma(pat+1) ORDER BY year) AS log_factorial,
       SUM(mu ORDER BY year) AS m,SUM(mu*logr ORDER BY year) AS h,
       SUM(mu*logr*logr ORDER BY year) AS j,EXP(-t) AS r FROM observation GROUP BY rep,id,t
   ), gamma_terms AS (
     SELECT *,COALESCE(LIST_SUM(LIST_TRANSFORM(RANGE(0,y_total::BIGINT),k -> 1/(r+k))),0.0) AS d,
       COALESCE(LIST_SUM(LIST_TRANSFORM(RANGE(0,y_total::BIGINT),k -> 1/POWER(r+k,2))),0.0) AS d2 FROM panel
   ), system AS (
     SELECT SUM(multiplicity) AS group_count,SUM(multiplicity*panel_n) AS row_count,MIN(panel_n) AS min_panel_n,MAX(panel_n) AS max_panel_n,
       SUM(multiplicity*(y_total-(r+y_total)*m/(r+m)) ORDER BY id) AS u0,
       SUM(multiplicity*(yx-(r+y_total)*h/(r+m)) ORDER BY id) AS u1,
       -MAX(r)*(SUM(multiplicity*d ORDER BY id)+SUM(multiplicity)*(LN(MAX(r))+1)-SUM(multiplicity*(LN(r+m)+(r+y_total)/(r+m)) ORDER BY id)) AS u2,
       SUM(multiplicity*(ABS(y_total)+ABS((r+y_total)*m/(r+m))) ORDER BY id) AS s0,
       SUM(multiplicity*(yx_scale+ABS((r+y_total)*h/(r+m))) ORDER BY id) AS s1,
       MAX(r)*(SUM(multiplicity*ABS(d) ORDER BY id)+SUM(multiplicity)*(ABS(LN(MAX(r)))+1)+SUM(multiplicity*(ABS(LN(r+m))+ABS((r+y_total)/(r+m))) ORDER BY id)) AS s2,
       SUM(multiplicity*(r+y_total)/(r+m)*m*r/(r+m) ORDER BY id) AS i00,
       SUM(multiplicity*(r+y_total)/(r+m)*h*r/(r+m) ORDER BY id) AS i01,
       -SUM(multiplicity*r*(m-y_total)*m/POWER(r+m,2) ORDER BY id) AS i02,
       SUM(multiplicity*(r+y_total)/(r+m)*(j-h*h/(r+m)) ORDER BY id) AS i11,
       SUM(multiplicity*(r+y_total)/(r+m)*(ABS(j)+h*h/(r+m)) ORDER BY id) AS i11_scale,
       -SUM(multiplicity*r*(m-y_total)*h/POWER(r+m,2) ORDER BY id) AS i12,
       -MAX(r)*(SUM(multiplicity*d ORDER BY id)+SUM(multiplicity)*(LN(MAX(r))+1)-SUM(multiplicity*(LN(r+m)+(r+y_total)/(r+m)) ORDER BY id))
         -POWER(MAX(r),2)*(-SUM(multiplicity*d2 ORDER BY id)+SUM(multiplicity)/MAX(r)-SUM(multiplicity/(r+m) ORDER BY id)+SUM(multiplicity*(y_total-m)/POWER(r+m,2) ORDER BY id)) AS i22,
       MAX(r)*(SUM(multiplicity*ABS(d) ORDER BY id)+SUM(multiplicity)*(ABS(LN(MAX(r)))+1)+SUM(multiplicity*(ABS(LN(r+m))+ABS((r+y_total)/(r+m))) ORDER BY id))
         +POWER(MAX(r),2)*(SUM(multiplicity*ABS(d2) ORDER BY id)+SUM(multiplicity)/MAX(r)+SUM(multiplicity/(r+m) ORDER BY id)+SUM(multiplicity*(ABS(y_total)+ABS(m))/POWER(r+m,2) ORDER BY id)) AS i22_scale,
       SUM(multiplicity*(lgamma(r+y_total)-lgamma(r)+r*LN(r)-(r+y_total)*LN(r+m)+a*y_total+b*yx-log_factorial) ORDER BY id) AS ll
     FROM gamma_terms
   ), first_pivot AS (
     SELECT *,CASE WHEN group_count!=search.g OR row_count!=5*search.g OR min_panel_n!=5 OR max_panel_n!=5
       OR NOT isfinite(u0) OR NOT isfinite(u1) OR NOT isfinite(u2) OR NOT isfinite(s0) OR NOT isfinite(s1) OR NOT isfinite(s2)
       OR NOT isfinite(i00) OR i00<=0 OR NOT isfinite(i01) OR NOT isfinite(i02) OR NOT isfinite(i11) OR NOT isfinite(i11_scale)
       OR NOT isfinite(i12) OR NOT isfinite(i22) OR NOT isfinite(i22_scale)
       OR NOT isfinite(ll) OR (iter>0 AND ll<search.ll-1e-10*(1+ABS(search.ll)))
       THEN error('gamma RE bootstrap incomplete, nonfinite or nonascending iterate') ELSE i00 END AS p00 FROM system
   ), second_pivot AS (
     SELECT *,CASE WHEN NOT isfinite(i11-i01*i01/p00)
       OR i11-i01*i01/p00<=2.842170943040401e-14*(1+i11_scale+ABS(i01*i01/p00))
       THEN error('gamma RE bootstrap information is singular') ELSE i11-i01*i01/p00 END AS p11 FROM first_pivot
   ), third_pivot AS (
     SELECT *, (i12-i01*i02/p00)/p11 AS l21,
       CASE WHEN NOT isfinite(i22-i02*i02/p00-POWER(i12-i01*i02/p00,2)/p11)
       OR i22-i02*i02/p00-POWER(i12-i01*i02/p00,2)/p11<=2.842170943040401e-14*(1+i22_scale+ABS(i02*i02/p00)+ABS(POWER(i12-i01*i02/p00,2)/p11))
       THEN error('gamma RE bootstrap information is singular') ELSE i22-i02*i02/p00-POWER(i12-i01*i02/p00,2)/p11 END AS p22
     FROM second_pivot
   ), steps AS (
     SELECT *, (u2-(i02/p00)*u0-l21*(u1-(i01/p00)*u0))/p22 AS dt FROM third_pivot
   ), steps2 AS (
     SELECT *, (u1-(i01/p00)*u0)/p11-l21*dt AS db FROM steps
   )
   SELECT *, (u0-i01*db-i02*dt)/p00 AS da FROM steps2
 ) step
 WHERE iter<100 AND (iter=0 OR last_step>1e-12)
), ranked AS MATERIALIZED (
 SELECT *,ROW_NUMBER() OVER (PARTITION BY rep ORDER BY iter DESC) AS ordinal FROM search
), last_iteration AS MATERIALIZED (
 SELECT * FROM ranked WHERE ordinal=1
), final_checked AS (
 SELECT *,CASE WHEN iter=0 OR last_step IS NULL OR NOT isfinite(last_step) OR last_step>1e-12
   OR ABS(u0)>1e-10*(1+s0) OR ABS(u1)>1e-10*(1+s1) OR ABS(u2)>1e-10*(1+s2)
   THEN error('gamma RE bootstrap failed convergence or final score check') ELSE old_a END AS intercept
 FROM last_iteration
), inverse_base AS (
 SELECT *,i11-i01*i01/i00 AS p11,(i12-i01*i02/i00)/(i11-i01*i01/i00) AS l21,
   i22-i02*i02/i00-POWER(i12-i01*i02/i00,2)/(i11-i01*i01/i00) AS p22 FROM final_checked
), inverse_info AS (
 SELECT *,1/p22 AS c22,-l21/p22 AS c12,
   1/p11+l21*l21/p22 AS c11,
   -(i02/i00-l21*i01/i00)/p22 AS c02,
   -(i01/i00)/p11+l21*(i02/i00-l21*i01/i00)/p22 AS c01,
   1/i00+POWER(i01/i00,2)/p11+POWER(i02/i00-l21*i01/i00,2)/p22 AS c00
 FROM inverse_base
), fits AS MATERIALIZED (
 SELECT rep,intercept AS alpha,CASE WHEN NOT isfinite(c00) OR NOT isfinite(c01) OR NOT isfinite(c02) OR NOT isfinite(c11) OR NOT isfinite(c12) OR NOT isfinite(c22)
   OR c00<=0 OR c11<=0 OR c22<=0
   OR 2.842170943040401e-14*(ABS(c00)*s0+ABS(c01)*s1+ABS(c02)*s2)>=1e-9+1e-10*ABS(intercept)
   OR 2.842170943040401e-14*(ABS(c01)*s0+ABS(c11)*s1+ABS(c12)*s2)>=1e-9+1e-10*ABS(old_b)
   OR 2.842170943040401e-14*(ABS(c02)*s0+ABS(c12)*s1+ABS(c22)*s2)>=1e-9+1e-10*ABS(old_t)
   THEN error('gamma RE bootstrap covariance or coefficient is numerically unreliable') ELSE old_b END AS beta,
   old_t AS lnalpha,iter AS iterations,last_step
 FROM inverse_info
), summary AS (
 SELECT COUNT(*) AS replications,STDDEV_SAMP(alpha ORDER BY rep) AS se_a,
   STDDEV_SAMP(beta ORDER BY rep) AS se_b,STDDEV_SAMP(lnalpha ORDER BY rep) AS se_t FROM fits WHERE rep>0
), original_predictions AS (
 SELECT AVG(EXP(o.alpha+o.beta*s.logr) ORDER BY s.id,s.year) AS original_mu_mean,
   MIN(EXP(o.alpha+o.beta*s.logr)) AS original_mu_min,MAX(EXP(o.alpha+o.beta*s.logr)) AS original_mu_max,
   AVG(s.pat-EXP(o.alpha+o.beta*s.logr) ORDER BY s.id,s.year) AS original_residual_mean
 FROM source s CROSS JOIN (SELECT alpha,beta FROM fits WHERE rep=0) o
), result AS (
 SELECT CASE WHEN replications!=500 OR se_a IS NULL OR NOT isfinite(se_a) OR se_b IS NULL OR NOT isfinite(se_b) OR se_t IS NULL OR NOT isfinite(se_t)
   OR o.alpha IS NULL OR o.beta IS NULL OR o.lnalpha IS NULL OR NOT isfinite(o.alpha) OR NOT isfinite(o.beta) OR NOT isfinite(o.lnalpha)
   OR original_mu_mean IS NULL OR NOT isfinite(original_mu_mean) OR original_residual_mean IS NULL OR NOT isfinite(original_residual_mean)
   THEN error('gamma RE bootstrap requires 500 valid replicates and an original fit') ELSE replications END AS replications,
   o.alpha AS estimate_a,o.beta AS estimate_b,o.lnalpha AS estimate_t,p.*
 FROM summary LEFT JOIN (SELECT alpha,beta,lnalpha FROM fits WHERE rep=0) o ON TRUE CROSS JOIN original_predictions p
)
SELECT draws.rep AS replicate,draws.position,draws.step,draws.id AS firm_id,
  fits.alpha AS replicate_intercept,fits.beta AS replicate_slope,fits.lnalpha AS replicate_lnalpha,
  fits.iterations AS replicate_iterations,fits.last_step AS replicate_last_step,
  result.estimate_a,result.estimate_b,result.estimate_t,result.original_mu_mean,result.original_mu_min,result.original_mu_max,
  result.original_residual_mean,result.replications,(SELECT g FROM params) AS firms_per_draw,(SELECT COUNT(*) FROM source) AS candidate_rows
FROM draws LEFT JOIN fits ON draws.rep=fits.rep CROSS JOIN result
ORDER BY replicate,position
""" into gamma_re_bootstrap_draws
sql """WITH r AS (SELECT DISTINCT replicate,replicate_intercept AS a,replicate_slope AS b,replicate_lnalpha AS t,estimate_a,estimate_b,estimate_t,original_mu_mean,original_mu_min,original_mu_max,original_residual_mean,replications,firms_per_draw,candidate_rows FROM active)
SELECT MAX(candidate_rows) AS candidate_rows,MAX(firms_per_draw) AS firms_per_draw,COUNT(*) AS replications,
 MAX(estimate_a) AS estimate_intercept,MAX(estimate_b) AS estimate_slope,MAX(estimate_t) AS estimate_lnalpha,
 MAX(original_mu_mean) AS original_mu_mean,MAX(original_mu_min) AS original_mu_min,MAX(original_mu_max) AS original_mu_max,MAX(original_residual_mean) AS original_residual_mean,
 AVG(a ORDER BY replicate) AS mean_intercept,AVG(b ORDER BY replicate) AS mean_slope,AVG(t ORDER BY replicate) AS mean_lnalpha,
 AVG(a ORDER BY replicate)-MAX(estimate_a) AS bias_intercept,AVG(b ORDER BY replicate)-MAX(estimate_b) AS bias_slope,AVG(t ORDER BY replicate)-MAX(estimate_t) AS bias_lnalpha,
 VAR_SAMP(a ORDER BY replicate) AS covariance_aa,COVAR_SAMP(a,b ORDER BY replicate) AS covariance_ab,COVAR_SAMP(a,t ORDER BY replicate) AS covariance_at,
 VAR_SAMP(b ORDER BY replicate) AS covariance_bb,COVAR_SAMP(b,t ORDER BY replicate) AS covariance_bt,VAR_SAMP(t ORDER BY replicate) AS covariance_tt,
 STDDEV_SAMP(a ORDER BY replicate) AS se_intercept,STDDEV_SAMP(b ORDER BY replicate) AS se_slope,STDDEV_SAMP(t ORDER BY replicate) AS se_lnalpha,
 QUANTILE_CONT(a,0.025 ORDER BY replicate) AS lower_intercept,QUANTILE_CONT(a,0.975 ORDER BY replicate) AS upper_intercept,
 QUANTILE_CONT(b,0.025 ORDER BY replicate) AS lower_slope,QUANTILE_CONT(b,0.975 ORDER BY replicate) AS upper_slope,
 QUANTILE_CONT(t,0.025 ORDER BY replicate) AS lower_lnalpha,QUANTILE_CONT(t,0.975 ORDER BY replicate) AS upper_lnalpha,
 10001 AS base_seed,'xoshiro256++/splitmix64/per_replicate_seed_plus_index/u64_rejection_v1' AS rng,
 'whole_firm_pairs_distinct_gamma_copies' AS method,'type7_percentile_95' AS interval_method FROM r"""
export gamma_re_bootstrap_draws.csv
