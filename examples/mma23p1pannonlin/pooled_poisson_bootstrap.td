# 500 whole-firm pooled-Poisson resamples; per-replicate xoshiro streams.
use patents.csv
isid id year
sql """SELECT CASE WHEN COUNT(*) = 0 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR pat IS NULL OR logr IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR NOT isfinite(pat) OR NOT isfinite(logr) OR id != floor(id) OR id < 1 OR id > 346 OR ABS(logr) > 20 OR year != floor(year) OR year < 1975 OR year > 1979 OR pat < 0 OR pat != floor(pat) OR pat > 1000) > 0 THEN error('invalid patents firm/year/count/model data') ELSE COUNT(*) END AS candidate_rows, COUNT(DISTINCT id) AS candidate_firms FROM active"""
sql """SELECT CASE WHEN COUNT(*) = 0 OR MIN(n) != 5 OR MAX(n) != 5 THEN error('each firm requires all five years 1975–79') ELSE COUNT(*) END AS firms FROM (SELECT id, COUNT(*) AS n FROM active GROUP BY id)"""
poisson pat logr
predict pooled_mu, n
predict pooled_xb, xb
summarize pooled_mu pooled_xb
sql """
WITH RECURSIVE
source AS (SELECT id,year,pat,logr FROM active),
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
), endpoint_units AS (
 -- The pooled intercept is profiled globally, not separately by firm.
 SELECT id,SUM(pat ORDER BY year) AS total,SUM(pat*logr ORDER BY year) AS observed,
   SUM(ABS(pat*logr) ORDER BY year) AS observed_abs,
   SUM(EXP(-10*logr) ORDER BY year) AS lo_w,SUM(logr*EXP(-10*logr) ORDER BY year) AS lo_x,
   SUM(ABS(logr)*EXP(-10*logr) ORDER BY year) AS lo_absx,
   SUM(EXP(10*logr) ORDER BY year) AS hi_w,SUM(logr*EXP(10*logr) ORDER BY year) AS hi_x,
   SUM(ABS(logr)*EXP(10*logr) ORDER BY year) AS hi_absx
 FROM source GROUP BY id
), endpoint_sums AS (
 SELECT f.rep,SUM(multiplicity*total ORDER BY id) AS total,
   SUM(multiplicity*observed ORDER BY id) AS observed,SUM(multiplicity*observed_abs ORDER BY id) AS observed_abs,
   SUM(multiplicity*lo_w ORDER BY id) AS lo_w,SUM(multiplicity*lo_x ORDER BY id) AS lo_x,
   SUM(multiplicity*lo_absx ORDER BY id) AS lo_absx,
   SUM(multiplicity*hi_w ORDER BY id) AS hi_w,SUM(multiplicity*hi_x ORDER BY id) AS hi_x,
   SUM(multiplicity*hi_absx ORDER BY id) AS hi_absx
 FROM frequencies f JOIN endpoint_units USING(id) GROUP BY f.rep
), bounds AS (
 SELECT rep,CASE WHEN total IS NULL OR total<=0 OR NOT isfinite(total)
   OR NOT isfinite(observed) OR NOT isfinite(observed_abs) OR NOT isfinite(lo_w) OR lo_w<=0 OR NOT isfinite(lo_x)
   OR NOT isfinite(lo_absx) OR NOT isfinite(hi_w) OR hi_w<=0 OR NOT isfinite(hi_x) OR NOT isfinite(hi_absx)
   OR observed-total*lo_x/lo_w<=2.842170943040401e-14*(1+observed_abs+total*lo_absx/lo_w)
   OR observed-total*hi_x/hi_w>=-2.842170943040401e-14*(1+observed_abs+total*hi_absx/hi_w)
   THEN error('pooled Poisson bootstrap replicate lacks a distinct interior score crossing') ELSE 1 END AS valid
 FROM endpoint_sums
), search(rep,iter,lo,hi) AS (
 SELECT rep,0,CAST(-10 AS DOUBLE),CAST(10 AS DOUBLE) FROM bounds WHERE valid=1
 UNION ALL
 SELECT rep,iter+1,CASE WHEN score>0 THEN (lo+hi)/2 ELSE lo END,
   CASE WHEN score>0 THEN hi ELSE (lo+hi)/2 END
 FROM search CROSS JOIN LATERAL (
   SELECT SUM(multiplicity*observed ORDER BY id)-SUM(multiplicity*total ORDER BY id)*
     SUM(multiplicity*wx ORDER BY id)/SUM(multiplicity*w ORDER BY id) AS score FROM (
     SELECT p.id,MAX(f.multiplicity) AS multiplicity,SUM(p.pat ORDER BY p.year) AS total,
       SUM(p.pat*p.logr ORDER BY p.year) AS observed,
       SUM(EXP(((lo+hi)/2)*p.logr) ORDER BY p.year) AS w,
       SUM(p.logr*EXP(((lo+hi)/2)*p.logr) ORDER BY p.year) AS wx
     FROM frequencies f JOIN source p USING(id) WHERE f.rep=search.rep GROUP BY p.id
   )
 ) AS step WHERE iter<80
), parameters AS MATERIALIZED (
 SELECT rep,(lo+hi)/2 AS beta FROM search WHERE iter=80
), eval_units AS (
 SELECT p.rep,p.beta,f.id,MAX(f.multiplicity) AS multiplicity,
   SUM(q.pat ORDER BY q.year) AS total,SUM(q.pat*q.logr ORDER BY q.year) AS observed,
   SUM(ABS(q.pat*q.logr) ORDER BY q.year) AS observed_abs,
   SUM(EXP(p.beta*q.logr) ORDER BY q.year) AS w,
   SUM(q.logr*EXP(p.beta*q.logr) ORDER BY q.year) AS wx,
   SUM(ABS(q.logr)*EXP(p.beta*q.logr) ORDER BY q.year) AS w_absx
 FROM parameters p JOIN frequencies f ON p.rep=f.rep JOIN source q USING(id)
 GROUP BY p.rep,p.beta,f.id
), fit_sums AS MATERIALIZED (
 SELECT rep,MAX(beta) AS beta,SUM(multiplicity*total ORDER BY id) AS total,
   SUM(multiplicity*observed ORDER BY id) AS observed,SUM(multiplicity*observed_abs ORDER BY id) AS observed_abs,
   SUM(multiplicity*w ORDER BY id) AS w,SUM(multiplicity*wx ORDER BY id) AS wx,
   SUM(multiplicity*w_absx ORDER BY id) AS w_absx
 FROM eval_units GROUP BY rep
), fit_variance AS (
 -- Center before squaring to avoid subtracting near-equal weighted moments.
 SELECT s.rep,SUM(f.multiplicity*EXP(s.beta*q.logr)*POWER(q.logr-s.wx/s.w,2) ORDER BY q.id,q.year)/MAX(s.w) AS variance
 FROM fit_sums s JOIN frequencies f ON s.rep=f.rep JOIN source q USING(id) GROUP BY s.rep
), checked_fit AS (
 SELECT s.rep,s.beta,LN(s.total/s.w) AS alpha,
   s.observed-s.total*s.wx/s.w AS score,
   s.observed_abs+s.total*s.w_absx/s.w AS score_scale,s.total*v.variance AS information
 FROM fit_sums s JOIN fit_variance v USING(rep)
), fits AS MATERIALIZED (
 SELECT rep,alpha,CASE WHEN alpha IS NULL OR NOT isfinite(alpha) OR beta IS NULL OR NOT isfinite(beta)
   OR score IS NULL OR NOT isfinite(score) OR score_scale IS NULL OR NOT isfinite(score_scale)
   OR ABS(score)>1e-10*(1+score_scale) OR information IS NULL OR NOT isfinite(information) OR information<=0
   THEN error('pooled Poisson bootstrap replicate failed convergence or information') ELSE beta END AS beta
 FROM checked_fit
), summary AS (
 SELECT COUNT(*) AS replications,STDDEV_SAMP(alpha ORDER BY rep) AS se_a,
   STDDEV_SAMP(beta ORDER BY rep) AS se_b,COVAR_SAMP(alpha,beta ORDER BY rep) AS cov_ab
 FROM fits WHERE rep>0
), result AS (
 SELECT CASE WHEN replications!=500 OR se_a IS NULL OR NOT isfinite(se_a)
   OR se_b IS NULL OR NOT isfinite(se_b) OR cov_ab IS NULL OR NOT isfinite(cov_ab)
   THEN error('pooled Poisson bootstrap requires 500 valid replicates') ELSE replications END AS replications,
   (SELECT alpha FROM fits WHERE rep=0) AS estimate_a,(SELECT beta FROM fits WHERE rep=0) AS estimate_b
 FROM summary
)
SELECT draws.rep AS replicate,draws.position,draws.step,draws.id AS firm_id,
  fits.alpha AS replicate_intercept,fits.beta AS replicate_slope,result.estimate_a,result.estimate_b,
  result.replications,(SELECT g FROM params) AS firms_per_draw,(SELECT COUNT(*) FROM source) AS candidate_rows
FROM draws JOIN fits ON draws.rep=fits.rep CROSS JOIN result
ORDER BY replicate,position
""" into pooled_poisson_bootstrap_draws
sql """WITH r AS (SELECT DISTINCT replicate,replicate_intercept AS a,replicate_slope AS b,estimate_a,estimate_b,replications,firms_per_draw,candidate_rows FROM active) SELECT MAX(candidate_rows) AS candidate_rows,MAX(firms_per_draw) AS firms_per_draw,COUNT(*) AS replications,MAX(estimate_a) AS estimate_intercept,MAX(estimate_b) AS estimate_slope,AVG(a ORDER BY replicate) AS mean_intercept,AVG(b ORDER BY replicate) AS mean_slope,AVG(a ORDER BY replicate)-MAX(estimate_a) AS bias_intercept,AVG(b ORDER BY replicate)-MAX(estimate_b) AS bias_slope,VAR_SAMP(a ORDER BY replicate) AS covariance_aa,COVAR_SAMP(a,b ORDER BY replicate) AS covariance_ab,VAR_SAMP(b ORDER BY replicate) AS covariance_bb,STDDEV_SAMP(a ORDER BY replicate) AS se_intercept,STDDEV_SAMP(b ORDER BY replicate) AS se_slope,QUANTILE_CONT(a,0.025 ORDER BY replicate) AS lower_intercept,QUANTILE_CONT(a,0.975 ORDER BY replicate) AS upper_intercept,QUANTILE_CONT(b,0.025 ORDER BY replicate) AS lower_slope,QUANTILE_CONT(b,0.975 ORDER BY replicate) AS upper_slope,10001 AS base_seed,'xoshiro256++/splitmix64/per_replicate_seed_plus_index/u64_rejection_v1' AS rng,'whole_firm_pairs' AS method,'type7_percentile_95' AS interval_method FROM r"""
export pooled_poisson_bootstrap_draws.csv
