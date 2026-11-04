# 500 whole-firm conditional-Poisson FE resamples; per-replicate xoshiro streams.
use patents.csv
isid id year
sql """SELECT CASE WHEN COUNT(*) = 0 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR pat IS NULL OR logr IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR NOT isfinite(pat) OR NOT isfinite(logr) OR id != floor(id) OR id < 1 OR id > 346 OR ABS(logr) > 20 OR year != floor(year) OR year < 1975 OR year > 1979 OR pat < 0 OR pat != floor(pat) OR pat > 1000) > 0 THEN error('invalid patents firm/year/count/model data') ELSE COUNT(*) END AS candidate_rows, COUNT(DISTINCT id) AS candidate_firms FROM active"""
sql """SELECT CASE WHEN COUNT(*) = 0 OR MIN(n) != 5 OR MAX(n) != 5 THEN error('each firm requires all five years 1975–79') ELSE COUNT(*) END AS firms FROM (SELECT id, COUNT(*) AS n FROM active GROUP BY id)"""
sql """
WITH RECURSIVE
raw AS (
 SELECT id,year,pat,logr, SUM(pat ORDER BY year) OVER (PARTITION BY id) AS firm_total,
   logr-AVG(logr ORDER BY year) OVER (PARTITION BY id) AS centered_x
 FROM active
), checked AS (
 SELECT * EXCLUDE(centered_x),
   CASE WHEN centered_x IS NULL OR NOT isfinite(centered_x) OR ABS(centered_x)>20
     THEN error('conditional FE requires finite within-firm log-R&D deviations <= 20') ELSE centered_x END AS centered_x
 FROM raw
), positive AS (SELECT * FROM checked WHERE firm_total>0),
groups AS (
 SELECT id, ROW_NUMBER() OVER (ORDER BY id)-1 AS index0 FROM (SELECT DISTINCT id FROM positive)
), params AS (
 SELECT CASE WHEN COUNT(*)<2 OR COUNT(*)>346 THEN error('bootstrap requires 2..346 positive-total firms') ELSE COUNT(*) END AS g,
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
 SELECT id,MAX(firm_total) AS total,SUM(pat*centered_x ORDER BY year) AS observed,
   SUM(EXP(-10*centered_x) ORDER BY year) AS lo_w,
   SUM(centered_x*EXP(-10*centered_x) ORDER BY year) AS lo_x,
   SUM(EXP(10*centered_x) ORDER BY year) AS hi_w,
   SUM(centered_x*EXP(10*centered_x) ORDER BY year) AS hi_x
 FROM positive GROUP BY id
), endpoint_summary AS (
 SELECT f.rep,
   SUM(multiplicity*(observed-total*lo_x/lo_w) ORDER BY id) AS lo_score,
   SUM(multiplicity*(observed-total*hi_x/hi_w) ORDER BY id) AS hi_score,
   SUM(multiplicity*(ABS(observed)+total*ABS(lo_x/lo_w)) ORDER BY id) AS lo_scale,
   SUM(multiplicity*(ABS(observed)+total*ABS(hi_x/hi_w)) ORDER BY id) AS hi_scale
 FROM frequencies f JOIN endpoint_units USING(id) GROUP BY f.rep
), bounds AS (
 SELECT rep,CASE WHEN lo_score IS NULL OR hi_score IS NULL OR NOT isfinite(lo_score)
   OR NOT isfinite(hi_score) OR NOT isfinite(lo_scale) OR NOT isfinite(hi_scale)
   OR lo_score<=2.842170943040401e-14*(1+lo_scale)
   OR hi_score>=-2.842170943040401e-14*(1+hi_scale)
   THEN error('conditional FE bootstrap replicate lacks a distinct score crossing') ELSE 1 END AS valid
 FROM endpoint_summary
), search(rep,iter,lo,hi) AS (
 SELECT rep,0,CAST(-10 AS DOUBLE),CAST(10 AS DOUBLE) FROM bounds WHERE valid=1
 UNION ALL
 SELECT rep,iter+1,CASE WHEN score>0 THEN (lo+hi)/2 ELSE lo END,
   CASE WHEN score>0 THEN hi ELSE (lo+hi)/2 END
 FROM search CROSS JOIN LATERAL (
   SELECT SUM(multiplicity*(observed-total*wx/w) ORDER BY id) AS score FROM (
     SELECT p.id,MAX(f.multiplicity) AS multiplicity,MAX(p.firm_total) AS total,
       SUM(p.pat*p.centered_x ORDER BY p.year) AS observed,
       SUM(EXP(((lo+hi)/2)*p.centered_x) ORDER BY p.year) AS w,
       SUM(p.centered_x*EXP(((lo+hi)/2)*p.centered_x) ORDER BY p.year) AS wx
     FROM frequencies f JOIN positive p USING(id) WHERE f.rep=search.rep GROUP BY p.id
   )
 ) AS step WHERE iter<80
), parameters AS MATERIALIZED (
 SELECT rep,(lo+hi)/2 AS beta FROM search WHERE iter=80
), eval_units AS (
 SELECT p.rep,p.beta,f.id,MAX(f.multiplicity) AS multiplicity,MAX(q.firm_total) AS total,
   SUM(q.pat*q.centered_x ORDER BY q.year) AS observed,
   SUM(EXP(p.beta*q.centered_x) ORDER BY q.year) AS w,
   SUM(q.centered_x*EXP(p.beta*q.centered_x) ORDER BY q.year) AS wx
 FROM parameters p JOIN frequencies f ON p.rep=f.rep JOIN positive q USING(id)
 GROUP BY p.rep,p.beta,f.id
), eval_variances AS (
 SELECT u.*,
   SUM(EXP(u.beta*q.centered_x)*POWER(q.centered_x-u.wx/u.w,2) ORDER BY q.year)/u.w AS variance
 FROM eval_units u JOIN positive q USING(id)
 GROUP BY u.rep,u.beta,u.id,u.multiplicity,u.total,u.observed,u.w,u.wx
), checked_fit AS (
 SELECT rep,MAX(beta) AS beta,
   SUM(multiplicity*(observed-total*wx/w) ORDER BY id) AS score,
   SUM(multiplicity*(ABS(observed)+total*ABS(wx/w)) ORDER BY id) AS score_scale,
   SUM(multiplicity*total*variance ORDER BY id) AS information
 FROM eval_variances GROUP BY rep
), fits AS MATERIALIZED (
 SELECT rep,CASE WHEN beta IS NULL OR NOT isfinite(beta) OR score IS NULL OR NOT isfinite(score)
   OR score_scale IS NULL OR NOT isfinite(score_scale) OR ABS(score)>1e-10*(1+score_scale)
   OR information IS NULL OR NOT isfinite(information) OR information<=0
   THEN error('conditional FE bootstrap replicate failed convergence or information') ELSE beta END AS beta
 FROM checked_fit
), summary AS (
 SELECT COUNT(*) AS replications,AVG(beta ORDER BY rep) AS bootstrap_mean,
   STDDEV_SAMP(beta ORDER BY rep) AS bootstrap_se,
   QUANTILE_CONT(beta,0.025 ORDER BY rep) AS ci_lower,
   QUANTILE_CONT(beta,0.975 ORDER BY rep) AS ci_upper
 FROM fits WHERE rep>0
), result AS (
 SELECT CASE WHEN replications!=500 OR bootstrap_se IS NULL OR NOT isfinite(bootstrap_se)
   THEN error('conditional FE bootstrap requires 500 valid replicates') ELSE replications END AS replications,
   (SELECT beta FROM fits WHERE rep=0) AS estimate,bootstrap_mean,bootstrap_se,ci_lower,ci_upper
 FROM summary
)
SELECT draws.rep AS replicate,draws.position,draws.step,draws.id AS firm_id,fits.beta AS replicate_beta,
  result.estimate,result.replications,(SELECT g FROM params) AS firms_per_draw,
  (SELECT COUNT(*) FROM checked) AS candidate_rows,(SELECT COUNT(DISTINCT id) FROM checked) AS candidate_firms,
  (SELECT COUNT(*) FROM positive) AS retained_rows
FROM draws JOIN fits ON draws.rep=fits.rep CROSS JOIN result
ORDER BY replicate,position
""" into conditional_fe_bootstrap_draws
sql """WITH replicate_fits AS (SELECT DISTINCT replicate,replicate_beta,estimate,replications,firms_per_draw,candidate_rows,candidate_firms,retained_rows FROM active) SELECT MAX(candidate_rows) AS candidate_rows,MAX(candidate_firms) AS candidate_firms,MAX(retained_rows) AS retained_rows,MAX(firms_per_draw) AS firms_per_draw,COUNT(*) AS replications,MAX(estimate) AS estimate,AVG(replicate_beta ORDER BY replicate) AS bootstrap_mean,AVG(replicate_beta ORDER BY replicate)-MAX(estimate) AS bias,STDDEV_SAMP(replicate_beta ORDER BY replicate) AS bootstrap_se,QUANTILE_CONT(replicate_beta,0.025 ORDER BY replicate) AS ci_lower,QUANTILE_CONT(replicate_beta,0.975 ORDER BY replicate) AS ci_upper,10001 AS base_seed,'xoshiro256++/splitmix64/per_replicate_seed_plus_index/u64_rejection_v1' AS rng,'whole_firm_pairs' AS method,'type7_percentile_95' AS interval_method FROM replicate_fits"""
export conditional_fe_bootstrap_draws.csv
