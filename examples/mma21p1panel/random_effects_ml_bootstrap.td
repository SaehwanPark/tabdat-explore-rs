# Balanced MOM RE-ML whole-person pairs bootstrap; not fixed-theta GLS.
use MOM.csv
isid id year
isid source_row
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(*)>5320 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR source_row IS NULL OR lnhr IS NULL OR lnwg IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR NOT isfinite(source_row) OR NOT isfinite(lnhr) OR NOT isfinite(lnwg) OR id!=floor(id) OR id<1 OR id>532 OR year!=floor(year) OR year<1979 OR year>1988 OR source_row!=floor(source_row) OR source_row<1 OR source_row>5320 OR ABS(lnhr)>20 OR ABS(lnwg)>20)>0 THEN error('RE-ML bootstrap requires finite bounded MOM fields and exact source/person/year keys') ELSE COUNT(*) END AS candidate_rows FROM active"""
sql """SELECT CASE WHEN COUNT(*)<3 OR COUNT(*)>532 OR MIN(t)!=10 OR MAX(t)!=10 THEN error('RE-ML bootstrap requires 3..532 people with all ten years 1979–1988') ELSE COUNT(*) END AS people FROM (SELECT id,COUNT(*) AS t FROM active GROUP BY id)"""
xtreg lnhr lnwg, mle id(id) time(year)
predict reml_xb, xb
predict reml_residual, residuals
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
 THEN error('RE-ML bootstrap RNG work bound exhausted') ELSE COUNT(*) END AS valid
 FROM (SELECT rep,MAX(accepted) AS max_accepted FROM rng GROUP BY rep)
), draws AS MATERIALIZED (
 SELECT rng.rep,rng.accepted AS position,rng.step,groups.id
 FROM rng CROSS JOIN params CROSS JOIN rng_health JOIN groups ON groups.index0=(rng.v%g::UHUGEINT)::BIGINT
 WHERE rng.v IS NOT NULL AND rng.v<mask-(mask%g::UHUGEINT) AND valid=500
), frequencies AS MATERIALIZED (
 -- Multiplicity counts independent copies; each copy contributes its own T10 determinant.
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
 THEN error('RE-ML bootstrap replicate has unidentified or invalid within/between moments') ELSE wxx END AS wxx
 FROM moments
), grid AS MATERIALIZED (
 SELECT rep,eta,b,sw,sb,s,0.5*(10*g*q*sb/s-g) AS score,
 -0.5*(10*g*(LN(2*PI())+1+LN(s/(10*g)))+g*eta) AS ll
 FROM checked CROSS JOIN range(0,49) t(k)
 CROSS JOIN LATERAL (SELECT k*0.5::DOUBLE AS eta,EXP(-k*0.5::DOUBLE) AS q) v
 CROSS JOIN LATERAL (SELECT (wxy+q*10*bxy)/(wxx+q*10*bxx) AS b) coef
 CROSS JOIN LATERAL (SELECT wyy-2*b*wxy+b*b*wxx AS sw,10*(byy-2*b*bxy+b*b*bxx) AS sb) residual
 CROSS JOIN LATERAL (SELECT sw+q*sb AS s) variance
), brackets AS (
 SELECT rep,eta-0.5 AS lo,eta AS hi FROM (
 SELECT *,LAG(score) OVER (PARTITION BY rep ORDER BY eta) AS previous_score FROM grid
 ) WHERE previous_score>0 AND score<=0
), search(rep,iter,lo,hi) AS (
 SELECT rep,0,lo,hi FROM brackets
 UNION ALL
 SELECT p.rep,iter+1,CASE WHEN score>0 THEN (lo+hi)/2 ELSE lo END,
 CASE WHEN score>0 THEN hi ELSE (lo+hi)/2 END
 FROM search p JOIN checked m ON m.rep=p.rep
 CROSS JOIN LATERAL (SELECT EXP(-(lo+hi)/2) AS q) v
 CROSS JOIN LATERAL (SELECT (wxy+q*10*bxy)/(wxx+q*10*bxx) AS b) coef
 CROSS JOIN LATERAL (SELECT wyy-2*b*wxy+b*b*wxx AS sw,10*(byy-2*b*bxy+b*b*bxx) AS sb) residual
 CROSS JOIN LATERAL (SELECT 0.5*(10*g*q*sb/(sw+q*sb)-g) AS score) step
 WHERE iter<48
), candidates AS (
 SELECT rep,0.0::DOUBLE AS eta FROM checked
 UNION ALL SELECT rep,(lo+hi)/2 AS eta FROM search WHERE iter=48
), evaluated AS MATERIALIZED (
 SELECT c.rep,eta,my-b*mx AS a,b,s/(10*g) AS e,(s/(10*g))*(EXP(eta)-1)/10 AS u,
 0.5*(10*g*q*sb/s-g) AS score,
 -0.5*(10*g*(LN(2*PI())+1+LN(s/(10*g)))+g*eta) AS ll,
 -0.5*10*g*(q*(sbprime-sb)/s+q*q*sb*sb/(s*s)) AS profile_information,sw,sb
 FROM candidates c JOIN checked m ON m.rep=c.rep
 CROSS JOIN LATERAL (SELECT EXP(-eta) AS q,wxx+EXP(-eta)*10*bxx AS d) v
 CROSS JOIN LATERAL (SELECT (wxy+q*10*bxy)/d AS b,q*10*(wxy*bxx-wxx*bxy)/(d*d) AS bprime) coef
 CROSS JOIN LATERAL (SELECT wyy-2*b*wxy+b*b*wxx AS sw,10*(byy-2*b*bxy+b*b*bxx) AS sb,
 20*(b*bxx-bxy)*bprime AS sbprime) residual
 CROSS JOIN LATERAL (SELECT sw+q*sb AS s) variance
), selected AS MATERIALIZED (
 SELECT * FROM evaluated QUALIFY ROW_NUMBER() OVER (PARTITION BY rep ORDER BY ll DESC,eta)=1
), fits AS MATERIALIZED (
 SELECT f.rep,a,b,e,u,eta,f.ll,profile_information,
 CASE WHEN a IS NULL OR b IS NULL OR e IS NULL OR u IS NULL OR eta IS NULL OR f.ll IS NULL
 OR f.score IS NULL OR NOT isfinite(a) OR NOT isfinite(b) OR NOT isfinite(e) OR NOT isfinite(u)
 OR NOT isfinite(eta) OR NOT isfinite(f.ll) OR NOT isfinite(f.score) OR e<=1e-12 OR u<0
 OR sw<=1e-12 OR sb<0 OR ABS(a)>100 OR ABS(b)>100
 OR (eta=0 AND f.score>1e-8) OR (eta>0 AND (ABS(f.score)>1e-8 OR NOT isfinite(profile_information) OR profile_information<=1e-8))
 OR edge.score IS NULL OR NOT isfinite(edge.score) OR edge.score>=0 OR edge.ll IS NULL OR NOT isfinite(edge.ll) OR edge.ll>=f.ll
 THEN error('RE-ML bootstrap replicate failed bounded profile, stationarity or curvature certificate') ELSE f.score END AS profile_score
 FROM selected f LEFT JOIN (SELECT rep,score,ll FROM grid WHERE eta=24) edge ON edge.rep=f.rep
), native AS (
 SELECT REGR_INTERCEPT(reml_xb,lnwg) AS a,REGR_SLOPE(reml_xb,lnwg) AS b,
 COUNT(*) FILTER (WHERE reml_xb IS NULL OR NOT isfinite(reml_xb)) AS bad FROM source
), health AS (
 SELECT CASE WHEN COUNT(*)!=501 OR COUNT(profile_score)!=501 OR MAX(native.bad)>0
 OR MAX(ABS(CASE WHEN rep=0 THEN fits.a-native.a ELSE 0 END))>1e-8
 OR MAX(ABS(CASE WHEN rep=0 THEN fits.b-native.b ELSE 0 END))>1e-8
 THEN error('RE-ML bootstrap requires 500 valid refits and matching native original coefficients') ELSE COUNT(*) END AS valid
 FROM fits CROSS JOIN native
)
SELECT draws.rep AS replicate,draws.position AS copy_id,draws.step,draws.id AS person_id,
 fits.a AS intercept,fits.b AS slope,fits.e AS sigma_e_squared,fits.u AS sigma_u_squared,
 fits.eta,fits.ll AS log_likelihood,fits.profile_score,fits.profile_information,
 original.a AS estimate_intercept,original.b AS estimate_slope,
 (SELECT g FROM params) AS people_per_draw,10001 AS base_seed
FROM draws LEFT JOIN fits ON draws.rep=fits.rep CROSS JOIN (SELECT * FROM fits WHERE rep=0) original CROSS JOIN health
WHERE health.valid=501 ORDER BY replicate,copy_id
""" into reml_bootstrap_draws
sql """WITH r AS (SELECT DISTINCT replicate,intercept AS a,slope AS b,estimate_intercept,estimate_slope,people_per_draw FROM active) SELECT COUNT(*) AS replications,MAX(people_per_draw) AS people_per_draw,MAX(estimate_intercept) AS estimate_intercept,MAX(estimate_slope) AS estimate_slope,AVG(a ORDER BY replicate) AS mean_intercept,AVG(b ORDER BY replicate) AS mean_slope,AVG(a ORDER BY replicate)-MAX(estimate_intercept) AS bias_intercept,AVG(b ORDER BY replicate)-MAX(estimate_slope) AS bias_slope,VAR_SAMP(a ORDER BY replicate) AS covariance_aa,COVAR_SAMP(a,b ORDER BY replicate) AS covariance_ab,VAR_SAMP(b ORDER BY replicate) AS covariance_bb,STDDEV_SAMP(a ORDER BY replicate) AS se_intercept,STDDEV_SAMP(b ORDER BY replicate) AS se_slope,QUANTILE_CONT(a,0.025 ORDER BY replicate) AS lower_intercept,QUANTILE_CONT(a,0.975 ORDER BY replicate) AS upper_intercept,QUANTILE_CONT(b,0.025 ORDER BY replicate) AS lower_slope,QUANTILE_CONT(b,0.975 ORDER BY replicate) AS upper_slope,10001 AS base_seed,'xoshiro256++/splitmix64/per_replicate_seed_plus_index/u64_rejection_v1' AS rng,'whole_person_pairs_refit_balanced_gaussian_ml' AS method,'type7_percentile_95' AS interval_method FROM r""" into reml_bootstrap_summary
export panel_re_ml_bootstrap_summary.csv
use reml_bootstrap_draws
export panel_re_ml_bootstrap_draws.csv
use MOM.csv
predict reml_xb, xb
predict reml_residual, residuals
summarize reml_xb reml_residual
export panel_re_ml_bootstrap_predictions.csv
