# B500 FE refits: normalized level intercept, not the auxiliary centered-OLS intercept.
use MOM.csv
isid id year
isid source_row
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(*)>5320 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR source_row IS NULL OR lnhr IS NULL OR lnwg IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR NOT isfinite(source_row) OR NOT isfinite(lnhr) OR NOT isfinite(lnwg) OR id!=floor(id) OR id<1 OR id>532 OR year!=floor(year) OR year<1979 OR year>1988 OR source_row!=floor(source_row) OR source_row<1 OR source_row>5320 OR ABS(lnhr)>20 OR ABS(lnwg)>20)>0 THEN error('FE level bootstrap requires finite bounded model fields and exact keys') ELSE COUNT(*) END AS candidate_rows FROM active"""
sql """SELECT CASE WHEN COUNT(*)<3 OR COUNT(*)>532 OR MIN(t)!=10 OR MAX(t)!=10 THEN error('FE level bootstrap requires 3..532 people with all ten years 1979–1988') ELSE COUNT(*) END AS people FROM (SELECT id,COUNT(*) AS t FROM active GROUP BY id)"""
xtreg lnhr lnwg, fe id(id) time(year)
predict fe_xb, xb
predict fe_xbu, xbu
predict fe_u, u
predict fe_residual, residuals
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
 THEN error('FE level bootstrap RNG work bound exhausted') ELSE COUNT(*) END AS valid
 FROM (SELECT rep,MAX(accepted) AS max_accepted FROM rng GROUP BY rep)
), draws AS MATERIALIZED (
 SELECT rng.rep,rng.accepted AS position,rng.step,groups.id
 FROM rng CROSS JOIN params CROSS JOIN rng_health JOIN groups ON groups.index0=(rng.v%g::UHUGEINT)::BIGINT
 WHERE rng.v IS NOT NULL AND rng.v<mask-(mask%g::UHUGEINT) AND valid=500
), frequencies AS MATERIALIZED (
 -- Multiplicities represent independent complete copies; G and residual DF count copies.
 SELECT rep,id,COUNT(*) AS m FROM draws GROUP BY rep,id
 UNION ALL SELECT 0::BIGINT AS rep,id,1::BIGINT AS m FROM groups
), means AS MATERIALIZED (
 SELECT id,AVG(lnhr ORDER BY year) AS my,AVG(lnwg ORDER BY year) AS mx FROM source GROUP BY id
), units AS MATERIALIZED (
 SELECT id,MAX(my) AS my,MAX(mx) AS mx,SUM(POWER(lnwg-mx,2) ORDER BY year) AS wxx,
 SUM((lnwg-mx)*(lnhr-my) ORDER BY year) AS wxy
 FROM source JOIN means USING(id) GROUP BY id
), moments AS MATERIALIZED (
 SELECT rep,SUM(m) AS g,SUM(m*mx ORDER BY id)/SUM(m) AS mx,SUM(m*my ORDER BY id)/SUM(m) AS my,
 SUM(m*wxx ORDER BY id) AS wxx,SUM(m*wxy ORDER BY id) AS wxy
 FROM frequencies JOIN units USING(id) GROUP BY rep
), checked AS MATERIALIZED (
 SELECT * EXCLUDE(wxx),CASE WHEN g!=(SELECT g FROM params) OR wxx IS NULL OR NOT isfinite(wxx) OR wxx<=1e-12
 OR wxy IS NULL OR NOT isfinite(wxy) OR mx IS NULL OR my IS NULL OR NOT isfinite(mx) OR NOT isfinite(my)
 THEN error('FE level bootstrap replicate has unidentified or invalid within moments') ELSE wxx END AS wxx
 FROM moments
), parameters AS MATERIALIZED (
 SELECT *,wxy/wxx AS b,my-(wxy/wxx)*mx AS a FROM checked
), fits AS MATERIALIZED (
 SELECT rep,a,b,mx,my,9*g-1 AS residual_df,
 CASE WHEN a IS NULL OR b IS NULL OR NOT isfinite(a) OR NOT isfinite(b) OR ABS(a)>100 OR ABS(b)>100
 OR rss IS NULL OR NOT isfinite(rss) OR rss<0 OR effect_mean IS NULL OR NOT isfinite(effect_mean)
 OR ABS(effect_mean)>1e-10*(1+ABS(a))
 THEN error('FE level bootstrap replicate failed coefficient, residual or normalization bounds') ELSE rss END AS within_rss,
 effect_mean
 FROM parameters p
 CROSS JOIN LATERAL (
 SELECT SUM(f.m*POWER((s.lnhr-v.my)-p.b*(s.lnwg-v.mx),2) ORDER BY s.id,s.year) AS rss
 FROM frequencies f JOIN source s USING(id) JOIN means v USING(id) WHERE f.rep=p.rep
 ) residuals
 CROSS JOIN LATERAL (
 SELECT SUM(f.m*(u.my-p.b*u.mx-p.a) ORDER BY id)/p.g AS effect_mean
 FROM frequencies f JOIN units u USING(id) WHERE f.rep=p.rep
 ) normalization
), native AS (
 SELECT REGR_INTERCEPT(fe_xb,lnwg) AS a,REGR_SLOPE(fe_xb,lnwg) AS b,
 COUNT(*) FILTER (WHERE fe_xb IS NULL OR fe_xbu IS NULL OR fe_u IS NULL OR fe_residual IS NULL
 OR NOT isfinite(fe_xb) OR NOT isfinite(fe_xbu) OR NOT isfinite(fe_u) OR NOT isfinite(fe_residual)) AS bad FROM source
), health AS (
 SELECT CASE WHEN COUNT(*)!=501 OR COUNT(within_rss)!=501 OR MAX(native.bad)>0
 OR MAX(ABS(CASE WHEN rep=0 THEN fits.a-native.a ELSE 0 END))>1e-8
 OR MAX(ABS(CASE WHEN rep=0 THEN fits.b-native.b ELSE 0 END))>1e-8
 THEN error('FE level bootstrap requires 500 valid refits and matching native original normalization') ELSE COUNT(*) END AS valid
 FROM fits CROSS JOIN native
)
SELECT draws.rep AS replicate,draws.position AS copy_id,draws.step,draws.id AS person_id,
 fits.a AS intercept,fits.b AS slope,fits.mx AS sample_mean_lnwg,fits.my AS sample_mean_lnhr,
 fits.residual_df,fits.within_rss,fits.effect_mean AS normalized_effect_mean,
 original.a AS estimate_intercept,original.b AS estimate_slope,(SELECT g FROM params) AS people_per_draw,10001 AS base_seed
FROM draws LEFT JOIN fits ON draws.rep=fits.rep CROSS JOIN (SELECT * FROM fits WHERE rep=0) original CROSS JOIN health
WHERE health.valid=501 ORDER BY replicate,copy_id
""" into fe_level_bootstrap_draws
sql """WITH r AS (SELECT DISTINCT replicate,intercept AS a,slope AS b,estimate_intercept,estimate_slope,people_per_draw FROM active) SELECT COUNT(*) AS replications,MAX(people_per_draw) AS people_per_draw,MAX(estimate_intercept) AS estimate_intercept,MAX(estimate_slope) AS estimate_slope,AVG(a ORDER BY replicate) AS mean_intercept,AVG(b ORDER BY replicate) AS mean_slope,AVG(a ORDER BY replicate)-MAX(estimate_intercept) AS bias_intercept,AVG(b ORDER BY replicate)-MAX(estimate_slope) AS bias_slope,VAR_SAMP(a ORDER BY replicate) AS covariance_aa,COVAR_SAMP(a,b ORDER BY replicate) AS covariance_ab,VAR_SAMP(b ORDER BY replicate) AS covariance_bb,STDDEV_SAMP(a ORDER BY replicate) AS se_intercept,STDDEV_SAMP(b ORDER BY replicate) AS se_slope,QUANTILE_CONT(a,0.025 ORDER BY replicate) AS lower_intercept,QUANTILE_CONT(a,0.975 ORDER BY replicate) AS upper_intercept,QUANTILE_CONT(b,0.025 ORDER BY replicate) AS lower_slope,QUANTILE_CONT(b,0.975 ORDER BY replicate) AS upper_slope,10001 AS base_seed,'xoshiro256++/splitmix64/per_replicate_seed_plus_index/u64_rejection_v1' AS rng,'whole_person_pairs_refit_normalized_level_fe' AS method,'type7_percentile_95' AS interval_method FROM r""" into fe_level_bootstrap_summary
sql """SELECT '_cons' AS parameter,estimate_intercept AS estimate,se_intercept AS bootstrap_se,lower_intercept AS percentile95_lower,upper_intercept AS percentile95_upper FROM active UNION ALL SELECT 'lnwg',estimate_slope,se_slope,lower_slope,upper_slope FROM active"""
export panel_fe_level_bootstrap_summary.csv
use fe_level_bootstrap_draws
export panel_fe_level_bootstrap_draws.csv
use MOM.csv
predict fe_xb, xb
predict fe_xbu, xbu
predict fe_u, u
predict fe_residual, residuals
summarize fe_xb fe_xbu fe_u fe_residual
export panel_fe_level_bootstrap_predictions.csv
