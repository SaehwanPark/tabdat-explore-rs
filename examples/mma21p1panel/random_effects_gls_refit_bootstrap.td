# Balanced MOM Swamy–Arora RE-GLS bootstrap; re-estimate components per draw.
use MOM.csv
isid id year
isid source_row
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(*)>5320 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR source_row IS NULL OR lnhr IS NULL OR lnwg IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR NOT isfinite(source_row) OR NOT isfinite(lnhr) OR NOT isfinite(lnwg) OR id!=floor(id) OR id<1 OR id>532 OR year!=floor(year) OR year<1979 OR year>1988 OR source_row!=floor(source_row) OR source_row<1 OR source_row>5320 OR ABS(lnhr)>20 OR ABS(lnwg)>20)>0 THEN error('RE-GLS refit bootstrap requires finite bounded model fields and exact source/person/year keys') ELSE COUNT(*) END AS candidate_rows FROM active"""
sql """SELECT CASE WHEN COUNT(*)<3 OR COUNT(*)>532 OR MIN(t)!=10 OR MAX(t)!=10 THEN error('RE-GLS refit bootstrap requires 3..532 people with all ten years 1979–1988') ELSE COUNT(*) END AS people FROM (SELECT id,COUNT(*) AS t FROM active GROUP BY id)"""
xtreg lnhr lnwg, re id(id) time(year)
predict regls_xb, xb
predict regls_residual, residuals
sql """
WITH RECURSIVE
source AS MATERIALIZED (SELECT * FROM active),
groups AS (SELECT id,ROW_NUMBER() OVER (ORDER BY id)-1 AS index0 FROM (SELECT DISTINCT id FROM source)),
params AS (SELECT COUNT(*)::BIGINT AS g,10001::UHUGEINT AS seed,18446744073709551615::UHUGEINT AS mask FROM groups),
expansion AS (
 SELECT rep,k,((seed+rep::UHUGEINT-1+k::UHUGEINT*11400714819323198485::UHUGEINT)&mask) AS z
 FROM params CROSS JOIN range(1,501) t(rep) CROSS JOIN range(1,5) q(k)
), mix1 AS (SELECT rep,k,(xor(z,z>>30)*13787848793156543929::UHUGEINT)&mask AS z FROM expansion CROSS JOIN params),
mix2 AS (SELECT rep,k,(xor(z,z>>27)*10723151780598845931::UHUGEINT)&mask AS z FROM mix1 CROSS JOIN params),
seeds AS (
 SELECT rep,MAX(xor(z,z>>31)) FILTER(WHERE k=1) AS s0,MAX(xor(z,z>>31)) FILTER(WHERE k=2) AS s1,
 MAX(xor(z,z>>31)) FILTER(WHERE k=3) AS s2,MAX(xor(z,z>>31)) FILTER(WHERE k=4) AS s3 FROM mix2 GROUP BY rep
), rng(rep,step,accepted,s0,s1,s2,s3,v) AS (
 SELECT rep,0::BIGINT,0::BIGINT,s0,s1,s2,s3,NULL::UHUGEINT FROM seeds
 UNION ALL
 SELECT rep,step+1,accepted+CASE WHEN v_next<zone THEN 1 ELSE 0 END,
 xor(s0,xor(s3,s1)),xor(s1,xor(s2,s0)),xor(xor(s2,s0),(s1<<17)&mask),
 ((xor(s3,s1)<<45)|(xor(s3,s1)>>19))&mask,v_next
 FROM rng CROSS JOIN params
 CROSS JOIN LATERAL (SELECT mask-(mask%g::UHUGEINT) AS zone,(s0+s3)&mask AS a) q
 CROSS JOIN LATERAL (SELECT ((((a<<23)|(a>>41))&mask)+s0)&mask AS v_next) v
 WHERE accepted<g AND step<g+1024
), rng_health AS (
 SELECT CASE WHEN COUNT(*)!=500 OR MIN(max_accepted)!=(SELECT g FROM params)
 THEN error('RE-GLS refit bootstrap RNG work bound exhausted') ELSE COUNT(*) END AS valid
 FROM (SELECT rep,MAX(accepted) AS max_accepted FROM rng GROUP BY rep)
), draws AS MATERIALIZED (
 SELECT rng.rep,rng.accepted AS position,rng.step,groups.id
 FROM rng CROSS JOIN params CROSS JOIN rng_health JOIN groups ON groups.index0=(rng.v%g::UHUGEINT)::BIGINT
 WHERE rng.v IS NOT NULL AND rng.v<mask-(mask%g::UHUGEINT) AND valid=500
), frequencies AS MATERIALIZED (
 -- Repeated people are independent copies; multiplicity contributes to G and residual DF.
 SELECT rep,id,COUNT(*) AS m FROM draws GROUP BY rep,id
 UNION ALL SELECT 0::BIGINT AS rep,id,1::BIGINT AS m FROM groups
), means AS MATERIALIZED (
 SELECT id,AVG(lnhr ORDER BY year) AS my,AVG(lnwg ORDER BY year) AS mx FROM source GROUP BY id
), units AS MATERIALIZED (
 SELECT id,MAX(my) AS my,MAX(mx) AS mx,
 SUM(POWER(lnwg-mx,2) ORDER BY year) AS wxx,
 SUM((lnwg-mx)*(lnhr-my) ORDER BY year) AS wxy,
 SUM(POWER(lnhr-my,2) ORDER BY year) AS wyy
 FROM source JOIN means USING(id) GROUP BY id
), centers AS MATERIALIZED (
 SELECT rep,SUM(m*mx ORDER BY id)/SUM(m) AS mx,SUM(m*my ORDER BY id)/SUM(m) AS my
 FROM frequencies JOIN units USING(id) GROUP BY rep
), moments AS MATERIALIZED (
 SELECT f.rep,MAX(c.mx) AS mx,MAX(c.my) AS my,SUM(m) AS g,
 SUM(m*wxx ORDER BY id) AS wxx,SUM(m*wxy ORDER BY id) AS wxy,SUM(m*wyy ORDER BY id) AS wyy,
 SUM(m*POWER(u.mx-c.mx,2) ORDER BY id) AS bxx,
 SUM(m*(u.mx-c.mx)*(u.my-c.my) ORDER BY id) AS bxy,
 SUM(m*POWER(u.my-c.my,2) ORDER BY id) AS byy
 FROM frequencies f JOIN units u USING(id) JOIN centers c ON c.rep=f.rep GROUP BY f.rep
), checked AS MATERIALIZED (
 SELECT * EXCLUDE(wxx),CASE WHEN g!=(SELECT g FROM params) OR NOT isfinite(wxx) OR wxx<=1e-12
 OR NOT isfinite(wxy) OR NOT isfinite(wyy) OR wyy<=1e-12 OR NOT isfinite(bxx) OR bxx<=1e-12
 OR NOT isfinite(bxy) OR NOT isfinite(byy) OR NOT isfinite(mx) OR NOT isfinite(my)
 THEN error('RE-GLS refit bootstrap replicate has unidentified or invalid within/between moments') ELSE wxx END AS wxx
 FROM moments
), preliminaries AS MATERIALIZED (
 SELECT c.*,wxy/wxx AS within_slope,bxy/bxx AS between_slope,ew/(9*g-1) AS e,eb/(g-2) AS between_variance
 FROM checked c
 CROSS JOIN LATERAL (
 SELECT SUM(f.m*POWER((p.lnhr-v.my)-(c.wxy/c.wxx)*(p.lnwg-v.mx),2) ORDER BY p.id,p.year) AS ew
 FROM frequencies f JOIN source p USING(id) JOIN means v USING(id) WHERE f.rep=c.rep
 ) within_residuals
 CROSS JOIN LATERAL (
 SELECT SUM(f.m*POWER((u.my-c.my)-(c.bxy/c.bxx)*(u.mx-c.mx),2) ORDER BY u.id) AS eb
 FROM frequencies f JOIN units u USING(id) WHERE f.rep=c.rep
 ) between_residuals
), components AS MATERIALIZED (
 SELECT * EXCLUDE(e),CASE WHEN e IS NULL OR NOT isfinite(e) OR e<=1e-12
 OR between_variance IS NULL OR NOT isfinite(between_variance) OR between_variance<0
 THEN error('RE-GLS refit bootstrap replicate has invalid preliminary variance components') ELSE e END AS e,
 between_variance-e/10 AS raw_u,GREATEST(0,between_variance-e/10) AS u
 FROM preliminaries
), parameters AS MATERIALIZED (
 SELECT *,e/(e+10*u) AS q,1-SQRT(e/(e+10*u)) AS theta,
 (wxy+(e/(e+10*u))*10*bxy)/(wxx+(e/(e+10*u))*10*bxx) AS b
 FROM components
), fits AS MATERIALIZED (
 SELECT rep,my-b*mx AS a,b,e,u,raw_u,theta,within_slope,between_slope,
 CASE WHEN b IS NULL OR NOT isfinite(b) OR ABS(b)>100 OR NOT isfinite(my-b*mx) OR ABS(my-b*mx)>100
 OR u IS NULL OR raw_u IS NULL OR theta IS NULL OR NOT isfinite(u) OR NOT isfinite(raw_u)
 OR NOT isfinite(theta) OR theta<0 OR theta>=1 OR NOT isfinite(q) OR q<=0 OR q>1
 OR rss IS NULL OR NOT isfinite(rss) OR rss<=1e-12
 THEN error('RE-GLS refit bootstrap replicate failed component, coefficient or residual bounds')
 ELSE rss END AS transformed_rss
 FROM parameters c
 CROSS JOIN LATERAL (
 SELECT SUM(f.m*POWER((p.lnhr-v.my)-c.b*(p.lnwg-v.mx)
 +SQRT(c.q)*((v.my-c.my)-c.b*(v.mx-c.mx)),2) ORDER BY p.id,p.year) AS rss
 FROM frequencies f JOIN source p USING(id) JOIN means v USING(id) WHERE f.rep=c.rep
 ) residuals
), native AS (
 SELECT REGR_INTERCEPT(regls_xb,lnwg) AS a,REGR_SLOPE(regls_xb,lnwg) AS b,
 COUNT(*) FILTER (WHERE regls_xb IS NULL OR NOT isfinite(regls_xb)) AS bad FROM source
), health AS (
 SELECT CASE WHEN COUNT(*)!=501 OR COUNT(transformed_rss)!=501 OR MAX(native.bad)>0
 OR MAX(ABS(CASE WHEN rep=0 THEN fits.a-native.a ELSE 0 END))>1e-8
 OR MAX(ABS(CASE WHEN rep=0 THEN fits.b-native.b ELSE 0 END))>1e-8
 THEN error('RE-GLS refit bootstrap requires 500 valid refits and matching native original coefficients') ELSE COUNT(*) END AS valid
 FROM fits CROSS JOIN native
)
SELECT draws.rep AS replicate,draws.position AS copy_id,draws.step,draws.id AS person_id,
 fits.a AS intercept,fits.b AS slope,fits.e AS sigma_e_squared,fits.u AS sigma_u_squared,
 fits.raw_u AS raw_sigma_u_squared,fits.theta,fits.within_slope,fits.between_slope,fits.transformed_rss,
 original.a AS estimate_intercept,original.b AS estimate_slope,
 (SELECT g FROM params) AS people_per_draw,10001 AS base_seed
FROM draws LEFT JOIN fits ON draws.rep=fits.rep CROSS JOIN (SELECT * FROM fits WHERE rep=0) original CROSS JOIN health
WHERE health.valid=501 ORDER BY replicate,copy_id
""" into regls_refit_bootstrap_draws
sql """WITH r AS (SELECT DISTINCT replicate,intercept AS a,slope AS b,estimate_intercept,estimate_slope,people_per_draw FROM active) SELECT COUNT(*) AS replications,MAX(people_per_draw) AS people_per_draw,MAX(estimate_intercept) AS estimate_intercept,MAX(estimate_slope) AS estimate_slope,AVG(a ORDER BY replicate) AS mean_intercept,AVG(b ORDER BY replicate) AS mean_slope,AVG(a ORDER BY replicate)-MAX(estimate_intercept) AS bias_intercept,AVG(b ORDER BY replicate)-MAX(estimate_slope) AS bias_slope,VAR_SAMP(a ORDER BY replicate) AS covariance_aa,COVAR_SAMP(a,b ORDER BY replicate) AS covariance_ab,VAR_SAMP(b ORDER BY replicate) AS covariance_bb,STDDEV_SAMP(a ORDER BY replicate) AS se_intercept,STDDEV_SAMP(b ORDER BY replicate) AS se_slope,QUANTILE_CONT(a,0.025 ORDER BY replicate) AS lower_intercept,QUANTILE_CONT(a,0.975 ORDER BY replicate) AS upper_intercept,QUANTILE_CONT(b,0.025 ORDER BY replicate) AS lower_slope,QUANTILE_CONT(b,0.975 ORDER BY replicate) AS upper_slope,10001 AS base_seed,'xoshiro256++/splitmix64/per_replicate_seed_plus_index/u64_rejection_v1' AS rng,'whole_person_pairs_refit_swamy_arora_gls' AS method,'type7_percentile_95' AS interval_method FROM r""" into regls_refit_bootstrap_summary
sql """SELECT '_cons' AS parameter,estimate_intercept AS estimate,se_intercept AS bootstrap_se,lower_intercept AS percentile95_lower,upper_intercept AS percentile95_upper FROM active UNION ALL SELECT 'lnwg',estimate_slope,se_slope,lower_slope,upper_slope FROM active"""
export panel_re_gls_refit_bootstrap_summary.csv
use regls_refit_bootstrap_draws
export panel_re_gls_refit_bootstrap_draws.csv
use MOM.csv
predict regls_xb, xb
predict regls_residual, residuals
summarize regls_xb regls_residual
export panel_re_gls_refit_bootstrap_predictions.csv
