# 500 whole-firm exponential-mean NLS resamples; per-replicate xoshiro streams.
use patents.csv
isid id year
sql """SELECT CASE WHEN COUNT(*)<=2 OR COUNT(*)>10000 OR COUNT(DISTINCT id)<2 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR pat IS NULL OR logr IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR NOT isfinite(pat) OR NOT isfinite(logr) OR id != floor(id) OR id < 1 OR id > 346 OR year != floor(year) OR year < 1975 OR year > 1979 OR pat < 0 OR pat != floor(pat) OR pat > 1000 OR ABS(logr)>20) > 0 THEN error('invalid patents firm/year/count/model data') ELSE COUNT(*) END AS candidate_rows, COUNT(DISTINCT id) AS firms FROM active"""
sql """SELECT CASE WHEN COUNT(*) = 0 OR MIN(n) != 5 OR MAX(n) != 5 THEN error('each firm requires all five years 1975–79') ELSE COUNT(*) END AS firms FROM (SELECT id, COUNT(*) AS n FROM active GROUP BY id)"""
nl pat = exp(a+b*logr), params(a b) start(2.5 0.5)
predict nls_mu, mean
predict nls_residual, residuals
summarize nls_mu nls_residual
sql """
WITH RECURSIVE
source AS (SELECT id,year,pat,logr,nls_mu FROM active),
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
), native_parameters AS (
 SELECT REGR_INTERCEPT(LN(nls_mu),logr) AS a,REGR_SLOPE(LN(nls_mu),logr) AS b,
   MAX(ABS(LN(nls_mu))) AS ln_scale,
   COUNT(*) FILTER (WHERE nls_mu IS NULL OR NOT isfinite(nls_mu) OR nls_mu<=0) AS bad
 FROM source
), native_checked AS (
 SELECT CASE WHEN a IS NULL OR b IS NULL OR NOT isfinite(a) OR NOT isfinite(b) OR bad>0
   OR MAX(ABS(LN(nls_mu)-(a+b*logr)))>1e-12*(1+MAX(ln_scale))
   THEN error('native NLS start cannot be reconstructed') ELSE a END AS a,b
 FROM native_parameters CROSS JOIN source GROUP BY a,b,bad
), endpoint_units AS (
 -- Profile the global amplitude; the resulting slope score need not be monotone.
 SELECT id,SUM(pat*EXP(-2*logr) ORDER BY year) AS lo_y,
   SUM(pat*logr*EXP(-2*logr) ORDER BY year) AS lo_yx,SUM(pat*ABS(logr)*EXP(-2*logr) ORDER BY year) AS lo_yabs,
   SUM(EXP(-4*logr) ORDER BY year) AS lo_w,SUM(logr*EXP(-4*logr) ORDER BY year) AS lo_wx,
   SUM(ABS(logr)*EXP(-4*logr) ORDER BY year) AS lo_wabs,
   SUM(pat*EXP(2*logr) ORDER BY year) AS hi_y,
   SUM(pat*logr*EXP(2*logr) ORDER BY year) AS hi_yx,SUM(pat*ABS(logr)*EXP(2*logr) ORDER BY year) AS hi_yabs,
   SUM(EXP(4*logr) ORDER BY year) AS hi_w,SUM(logr*EXP(4*logr) ORDER BY year) AS hi_wx,
   SUM(ABS(logr)*EXP(4*logr) ORDER BY year) AS hi_wabs
 FROM source GROUP BY id
), endpoint_sums AS (
 SELECT f.rep,SUM(multiplicity*lo_y ORDER BY id) AS lo_y,SUM(multiplicity*lo_yx ORDER BY id) AS lo_yx,
   SUM(multiplicity*lo_yabs ORDER BY id) AS lo_yabs,SUM(multiplicity*lo_w ORDER BY id) AS lo_w,
   SUM(multiplicity*lo_wx ORDER BY id) AS lo_wx,SUM(multiplicity*lo_wabs ORDER BY id) AS lo_wabs,
   SUM(multiplicity*hi_y ORDER BY id) AS hi_y,SUM(multiplicity*hi_yx ORDER BY id) AS hi_yx,
   SUM(multiplicity*hi_yabs ORDER BY id) AS hi_yabs,SUM(multiplicity*hi_w ORDER BY id) AS hi_w,
   SUM(multiplicity*hi_wx ORDER BY id) AS hi_wx,SUM(multiplicity*hi_wabs ORDER BY id) AS hi_wabs
 FROM frequencies f JOIN endpoint_units USING(id) GROUP BY f.rep
), bounds AS (
 SELECT rep,CASE WHEN NOT isfinite(lo_y) OR lo_y<=0 OR NOT isfinite(lo_yx) OR NOT isfinite(lo_yabs)
   OR NOT isfinite(lo_w) OR lo_w<=0 OR NOT isfinite(lo_wx) OR NOT isfinite(lo_wabs)
   OR NOT isfinite(hi_y) OR hi_y<=0 OR NOT isfinite(hi_yx) OR NOT isfinite(hi_yabs)
   OR NOT isfinite(hi_w) OR hi_w<=0 OR NOT isfinite(hi_wx) OR NOT isfinite(hi_wabs)
   OR lo_yx/lo_y-lo_wx/lo_w<=2.842170943040401e-14*(1+lo_yabs/lo_y+lo_wabs/lo_w)
   OR hi_yx/hi_y-hi_wx/hi_w>=-2.842170943040401e-14*(1+hi_yabs/hi_y+hi_wabs/hi_w)
   THEN error('NLS bootstrap replicate lacks distinct interior score bounds') ELSE 1 END AS valid
 FROM endpoint_sums
), search(rep,iter,lo,hi) AS (
 SELECT rep,0,CAST(-2 AS DOUBLE),CAST(2 AS DOUBLE) FROM bounds WHERE valid=1
 UNION ALL
 SELECT rep,iter+1,CASE WHEN score>0 THEN (lo+hi)/2 ELSE lo END,
   CASE WHEN score>0 THEN hi ELSE (lo+hi)/2 END
 FROM search CROSS JOIN LATERAL (
   SELECT SUM(multiplicity*yx ORDER BY id)/SUM(multiplicity*y ORDER BY id)
     -SUM(multiplicity*wx ORDER BY id)/SUM(multiplicity*w ORDER BY id) AS score FROM (
     SELECT p.id,MAX(f.multiplicity) AS multiplicity,
       SUM(p.pat*EXP(((lo+hi)/2)*p.logr) ORDER BY p.year) AS y,
       SUM(p.pat*p.logr*EXP(((lo+hi)/2)*p.logr) ORDER BY p.year) AS yx,
       SUM(EXP((lo+hi)*p.logr) ORDER BY p.year) AS w,
       SUM(p.logr*EXP((lo+hi)*p.logr) ORDER BY p.year) AS wx
     FROM frequencies f JOIN source p USING(id) WHERE f.rep=search.rep GROUP BY p.id
   )
 ) AS step WHERE iter<80
), parameters AS MATERIALIZED (
 SELECT rep,(lo+hi)/2 AS beta FROM search WHERE iter=80
), eval_units AS (
 SELECT p.rep,p.beta,f.id,MAX(f.multiplicity) AS multiplicity,
   SUM(q.pat*EXP(p.beta*q.logr) ORDER BY q.year) AS y,
   SUM(q.pat*q.logr*EXP(p.beta*q.logr) ORDER BY q.year) AS yx,
   SUM(q.pat*ABS(q.logr)*EXP(p.beta*q.logr) ORDER BY q.year) AS yabs,
   SUM(EXP(2*p.beta*q.logr) ORDER BY q.year) AS w,
   SUM(q.logr*EXP(2*p.beta*q.logr) ORDER BY q.year) AS wx,
   SUM(ABS(q.logr)*EXP(2*p.beta*q.logr) ORDER BY q.year) AS wabs
 FROM parameters p JOIN frequencies f ON p.rep=f.rep JOIN source q USING(id)
 GROUP BY p.rep,p.beta,f.id
), fit_sums AS MATERIALIZED (
 SELECT rep,MAX(beta) AS beta,SUM(multiplicity*y ORDER BY id) AS y,SUM(multiplicity*yx ORDER BY id) AS yx,
   SUM(multiplicity*yabs ORDER BY id) AS yabs,SUM(multiplicity*w ORDER BY id) AS w,
   SUM(multiplicity*wx ORDER BY id) AS wx,SUM(multiplicity*wabs ORDER BY id) AS wabs
 FROM eval_units GROUP BY rep
), centers AS MATERIALIZED (
 SELECT *,LN(y/w) AS alpha,yx/y AS ymean,wx/w AS wmean FROM fit_sums
), final_moments AS (
 -- Center both weighted variances before squaring; compare observed-loss curvature.
 SELECT s.rep,SUM(f.multiplicity*q.pat*EXP(s.beta*q.logr)*POWER(q.logr-s.ymean,2) ORDER BY q.id,q.year)/MAX(s.y) AS vy,
   SUM(f.multiplicity*EXP(2*s.beta*q.logr)*POWER(q.logr-s.wmean,2) ORDER BY q.id,q.year)/MAX(s.w) AS vw,
   SUM(f.multiplicity*POWER(q.pat-EXP(s.alpha+s.beta*q.logr),2) ORDER BY q.id,q.year) AS rss,
   SUM(f.multiplicity*POWER(ABS(q.pat)+EXP(s.alpha+s.beta*q.logr),2) ORDER BY q.id,q.year) AS rss_scale,
   COUNT(*) FILTER (WHERE NOT isfinite(s.alpha+s.beta*q.logr) OR ABS(s.alpha+s.beta*q.logr)>40) AS invalid_eta
 FROM centers s JOIN frequencies f ON s.rep=f.rep JOIN source q USING(id) GROUP BY s.rep
), fits AS MATERIALIZED (
 SELECT s.rep,s.alpha,CASE WHEN NOT isfinite(s.alpha) OR ABS(s.alpha)>20 OR NOT isfinite(s.beta)
   OR NOT isfinite(s.ymean) OR NOT isfinite(s.wmean) OR NOT isfinite(s.yabs) OR NOT isfinite(s.wabs)
   OR ABS(s.ymean-s.wmean)>1e-10*(1+s.yabs/s.y+s.wabs/s.w)
   OR NOT isfinite(m.vy) OR NOT isfinite(m.vw)
   OR 2*m.vw-m.vy<=2.842170943040401e-14*(1+2*ABS(m.vw)+ABS(m.vy))
   OR (2*m.vw-m.vy)*(1e-9+1e-10*ABS(s.beta))<=2.842170943040401e-14*(1+s.yabs/s.y+s.wabs/s.w)
   OR NOT isfinite(m.rss) OR NOT isfinite(m.rss_scale) OR m.rss<=8.077935669463161e-28*m.rss_scale
   OR m.invalid_eta>0 THEN error('NLS bootstrap replicate failed score, curvature, residual or predictor bounds')
   ELSE s.beta END AS beta
 FROM centers s JOIN final_moments m USING(rep)
), summary AS (
 SELECT COUNT(*) AS replications,STDDEV_SAMP(alpha ORDER BY rep) AS se_a,
   STDDEV_SAMP(beta ORDER BY rep) AS se_b,COVAR_SAMP(alpha,beta ORDER BY rep) AS cov_ab
 FROM fits WHERE rep>0
), result AS (
 SELECT CASE WHEN replications!=500 OR se_a IS NULL OR NOT isfinite(se_a)
   OR se_b IS NULL OR NOT isfinite(se_b) OR cov_ab IS NULL OR NOT isfinite(cov_ab)
   OR o.alpha IS NULL OR o.beta IS NULL OR NOT isfinite(o.alpha) OR NOT isfinite(o.beta)
   OR ABS(o.alpha-n.a)>1e-9+1e-10*ABS(n.a) OR ABS(o.beta-n.b)>1e-9+1e-10*ABS(n.b)
   THEN error('NLS bootstrap requires 500 valid replicates and matching native original fit')
   ELSE replications END AS replications,o.alpha AS estimate_a,o.beta AS estimate_b
 FROM summary LEFT JOIN (SELECT alpha,beta FROM fits WHERE rep=0) o ON TRUE CROSS JOIN native_checked n
)
SELECT draws.rep AS replicate,draws.position,draws.step,draws.id AS firm_id,
  fits.alpha AS replicate_intercept,fits.beta AS replicate_slope,result.estimate_a,result.estimate_b,
  result.replications,(SELECT g FROM params) AS firms_per_draw,(SELECT COUNT(*) FROM source) AS candidate_rows
FROM draws LEFT JOIN fits ON draws.rep=fits.rep CROSS JOIN result
ORDER BY replicate,position
""" into exponential_nls_bootstrap_draws
sql """WITH r AS (SELECT DISTINCT replicate,replicate_intercept AS a,replicate_slope AS b,estimate_a,estimate_b,replications,firms_per_draw,candidate_rows FROM active) SELECT MAX(candidate_rows) AS candidate_rows,MAX(firms_per_draw) AS firms_per_draw,COUNT(*) AS replications,MAX(estimate_a) AS estimate_intercept,MAX(estimate_b) AS estimate_slope,AVG(a ORDER BY replicate) AS mean_intercept,AVG(b ORDER BY replicate) AS mean_slope,AVG(a ORDER BY replicate)-MAX(estimate_a) AS bias_intercept,AVG(b ORDER BY replicate)-MAX(estimate_b) AS bias_slope,VAR_SAMP(a ORDER BY replicate) AS covariance_aa,COVAR_SAMP(a,b ORDER BY replicate) AS covariance_ab,VAR_SAMP(b ORDER BY replicate) AS covariance_bb,STDDEV_SAMP(a ORDER BY replicate) AS se_intercept,STDDEV_SAMP(b ORDER BY replicate) AS se_slope,QUANTILE_CONT(a,0.025 ORDER BY replicate) AS lower_intercept,QUANTILE_CONT(a,0.975 ORDER BY replicate) AS upper_intercept,QUANTILE_CONT(b,0.025 ORDER BY replicate) AS lower_slope,QUANTILE_CONT(b,0.975 ORDER BY replicate) AS upper_slope,10001 AS base_seed,'xoshiro256++/splitmix64/per_replicate_seed_plus_index/u64_rejection_v1' AS rng,'whole_firm_pairs' AS method,'type7_percentile_95' AS interval_method FROM r"""
export exponential_nls_bootstrap_draws.csv
