# Chapter24 unbalanced commune FE bootstrap. source_row orders cross-sectional households; it is not time.
use vietnam.csv
isid source_row
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(*)>5999 OR COUNT(*) FILTER (WHERE typeof(source_row)!='BIGINT' OR source_row IS NULL OR source_row<1 OR source_row>5999 OR typeof(commune)!='BIGINT' OR commune IS NULL OR commune<1 OR commune>194 OR age IS NULL OR NOT isfinite(age) OR abs(age)>1000000 OR sex IS NULL OR NOT isfinite(sex) OR abs(sex)>1000000 OR hhsize IS NULL OR NOT isfinite(hhsize) OR abs(hhsize)>1000000 OR farm IS NULL OR NOT isfinite(farm) OR abs(farm)>1000000 OR educ IS NULL OR NOT isfinite(educ) OR abs(educ)>1000000 OR lnhhexp IS NULL OR NOT isfinite(lnhhexp) OR abs(lnhhexp)>1000000 OR (lnexp12m IS NOT NULL AND (NOT isfinite(lnexp12m) OR abs(lnexp12m)>1000000)))>0 THEN error('invalid household source, model values or commune key') ELSE COUNT(*) END AS source_rows, COUNT(*) FILTER (WHERE lnexp12m IS NULL) AS missing_outcomes FROM active"""
sql """SELECT * FROM active WHERE lnexp12m IS NOT NULL ORDER BY source_row""" into household_fe_bootstrap_cohort
sql """SELECT CASE WHEN COUNT(*)<7 OR COUNT(*)>194 OR MAX(n)>39 OR SUM(n)<=COUNT(*)+6 THEN error('FE bootstrap requires 7..194 communes, each size<=39, and positive original residual DF') ELSE COUNT(*) END AS communes, SUM(n) AS observations, MIN(n) AS minimum_size, MAX(n) AS maximum_size FROM (SELECT commune,COUNT(*) AS n FROM active GROUP BY commune)"""
xtreg lnexp12m lnhhexp age sex hhsize farm educ, fe id(commune) time(source_row)
predict fe_xb, xb
predict fe_xbu, xbu
predict fe_u, u
predict fe_residual, residuals
sql """
WITH RECURSIVE
source AS MATERIALIZED (SELECT * FROM active),
-- Each list is one commune's 6x6 within cross-product, six cross-products
-- with y, observation count, and seven raw sums; singletons have zero within moments.
means AS MATERIALIZED (
 SELECT commune,COUNT(*) AS n,AVG(lnexp12m ORDER BY source_row) AS my,
 [AVG(lnhhexp ORDER BY source_row),AVG(age ORDER BY source_row),AVG(sex ORDER BY source_row),AVG(hhsize ORDER BY source_row),AVG(farm ORDER BY source_row),AVG(educ ORDER BY source_row)] AS mx
 FROM source GROUP BY commune
),
vector_source AS MATERIALIZED (SELECT commune,source_row,lnexp12m,[lnhhexp::DOUBLE,age::DOUBLE,sex::DOUBLE,hhsize::DOUBLE,farm::DOUBLE,educ::DOUBLE] AS x FROM source),
units AS MATERIALIZED (
 SELECT commune,i,j,SUM((x[i]-mx[i])*(x[j]-mx[j]) ORDER BY source_row) AS wxx,SUM((x[i]-mx[i])*(lnexp12m-my) ORDER BY source_row) AS wxy
 FROM vector_source JOIN means USING(commune) CROSS JOIN range(1,7) ri(i) CROSS JOIN range(1,7) rj(j) GROUP BY commune,i,j
),
unit_arrays AS MATERIALIZED (
 SELECT commune,LIST(wxx ORDER BY i,j) AS mat,LIST(wxy ORDER BY i) FILTER(WHERE j=1) AS rhs FROM units GROUP BY commune
),
groups AS MATERIALIZED (
 SELECT commune,ROW_NUMBER() OVER (ORDER BY commune)-1 AS index0,
 LIST_CONCAT(mat,rhs,[n::DOUBLE,n*my,n*mx[1],n*mx[2],n*mx[3],n*mx[4],n*mx[5],n*mx[6]]) AS moments
 FROM unit_arrays JOIN means USING(commune)
),
params AS MATERIALIZED (SELECT CASE WHEN COUNT(*)<7 OR COUNT(*)>194 THEN error('FE bootstrap requires 7..194 retained communes') ELSE COUNT(*)::BIGINT END AS g,18446744073709551615::UHUGEINT AS mask FROM groups),
expansion AS (SELECT rep,k,((10001::UHUGEINT+rep::UHUGEINT-1+k::UHUGEINT*11400714819323198485::UHUGEINT)&mask) AS z FROM params CROSS JOIN range(1,501) t(rep) CROSS JOIN range(1,5) q(k)),
mix1 AS (SELECT rep,k,(xor(z,z>>30)*13787848793156543929::UHUGEINT)&mask AS z FROM expansion CROSS JOIN params),
mix2 AS (SELECT rep,k,(xor(z,z>>27)*10723151780598845931::UHUGEINT)&mask AS z FROM mix1 CROSS JOIN params),
seeds AS (SELECT rep,MAX(xor(z,z>>31)) FILTER(WHERE k=1) AS s0,MAX(xor(z,z>>31)) FILTER(WHERE k=2) AS s1,MAX(xor(z,z>>31)) FILTER(WHERE k=3) AS s2,MAX(xor(z,z>>31)) FILTER(WHERE k=4) AS s3 FROM mix2 GROUP BY rep),
-- Accumulate complete copies in draw order; rejected RNG words do not alter the moments.
rng(rep,step,accepted,s0,s1,s2,s3,v,commune,acc) AS (
 SELECT rep,0::BIGINT,0::BIGINT,s0,s1,s2,s3,NULL::UHUGEINT,NULL::BIGINT,LIST_TRANSFORM(range(1,51),i->0.0::DOUBLE) FROM seeds
 UNION ALL
 SELECT rep,step+1,accepted+CASE WHEN v_next<zone THEN 1 ELSE 0 END,
 xor(s0,xor(s3,s1)),xor(s1,xor(s2,s0)),xor(xor(s2,s0),(s1<<17)&mask),((xor(s3,s1)<<45)|(xor(s3,s1)>>19))&mask,v_next,
 CASE WHEN v_next<zone THEN groups.commune ELSE NULL END,
 CASE WHEN v_next<zone THEN LIST_TRANSFORM(range(1,51),i->acc[i]+groups.moments[i]) ELSE acc END
 FROM rng CROSS JOIN params CROSS JOIN LATERAL (SELECT mask-(mask%g::UHUGEINT) AS zone,(s0+s3)&mask AS a) q
 CROSS JOIN LATERAL (SELECT ((((a<<23)|(a>>41))&mask)+s0)&mask AS v_next) v
 JOIN groups ON groups.index0=(v_next%g::UHUGEINT)::BIGINT
 WHERE accepted<g AND step<g+1024
),
rng_health AS (SELECT CASE WHEN COUNT(*)!=500 OR MIN(n)!=(SELECT g FROM params) THEN error('FE bootstrap RNG bound exhausted') ELSE COUNT(*) END AS valid FROM (SELECT rep,MAX(accepted) AS n FROM rng GROUP BY rep)),
draws AS MATERIALIZED (SELECT rep,accepted AS position,step,commune FROM rng CROSS JOIN rng_health WHERE commune IS NOT NULL AND valid=500),
original AS MATERIALIZED (
 SELECT LIST_TRANSFORM(range(1,51),i -> LIST_SUM(LIST_TRANSFORM(gs,g -> g[i]))) AS acc
 FROM (SELECT LIST(moments ORDER BY commune) AS gs FROM groups)
),
matrices AS MATERIALIZED (
 SELECT rep,acc[1:36] AS mat,acc[37:42] AS rhs,acc[43] AS sample_n,acc[44]/acc[43] AS my,LIST_TRANSFORM(range(1,7),i -> acc[44+i]/acc[43]) AS mx
 FROM (SELECT rep,acc FROM rng CROSS JOIN rng_health CROSS JOIN params WHERE accepted=g AND valid=500 UNION ALL SELECT 0 AS rep,acc FROM original)
),
-- Scale the normal equations to unit diagonal. Reject small/nonpositive Schur
-- pivots rather than returning a potentially unstable or rank-deficient fit.
scaling AS MATERIALIZED (
 SELECT rep,mat,rhs,LIST_TRANSFORM(range(1,7),i -> CASE WHEN mat[(i-1)*6+i] IS NULL OR NOT isfinite(mat[(i-1)*6+i]) OR mat[(i-1)*6+i]<=0 THEN error('FE bootstrap zero/nonfinite within variation') ELSE sqrt(mat[(i-1)*6+i]) END) AS scale FROM matrices
),
start AS MATERIALIZED (
 SELECT rep,scale,LIST_TRANSFORM(range(1,43),k -> CASE WHEN (k-1)%7+1=7 THEN rhs[(k-1)//7+1]/scale[(k-1)//7+1] ELSE mat[((k-1)//7)*6+(k-1)%7+1]/(scale[(k-1)//7+1]*scale[(k-1)%7+1]) END) AS a FROM scaling
),
solver(rep,step,scale,a) AS (
 SELECT rep,1,scale,a FROM start
 UNION ALL
 SELECT rep,step+1,scale,LIST_TRANSFORM(range(1,43),k -> CASE WHEN (k-1)//7+1=step THEN a[k]/denom ELSE a[k]-a[((k-1)//7)*7+step]*a[(step-1)*7+(k-1)%7+1]/denom END)
 FROM solver CROSS JOIN LATERAL (SELECT CASE WHEN a[(step-1)*7+step] IS NULL OR NOT isfinite(a[(step-1)*7+step]) OR a[(step-1)*7+step]<=1e-3 THEN error('FE bootstrap unidentified or ill-conditioned within design') ELSE a[(step-1)*7+step] END AS denom) p
 WHERE step<=6
),
slopes AS MATERIALIZED (SELECT rep,LIST_TRANSFORM(range(1,7),i -> a[(i-1)*7+7]/scale[i]) AS b FROM solver WHERE step=7),
fits AS MATERIALIZED (
 SELECT rep,sample_n::BIGINT AS sample_n,(SELECT g FROM params) AS group_copies,my-list_sum(LIST_TRANSFORM(range(1,7),i -> mx[i]*b[i])) AS intercept,b[1] AS lnhhexp,b[2] AS age,b[3] AS sex,b[4] AS hhsize,b[5] AS farm,b[6] AS educ FROM slopes JOIN matrices USING(rep)
),
native AS MATERIALIZED (
 SELECT MAX(ABS(fe_xb-(f.intercept+f.lnhhexp*source.lnhhexp+f.age*source.age+f.sex*source.sex+f.hhsize*source.hhsize+f.farm*source.farm+f.educ*source.educ))) AS xb_error,
 COUNT(*) FILTER(WHERE fe_xb IS NULL OR fe_xbu IS NULL OR fe_u IS NULL OR fe_residual IS NULL OR NOT isfinite(fe_xb) OR NOT isfinite(fe_xbu) OR NOT isfinite(fe_u) OR NOT isfinite(fe_residual)) AS bad
 FROM source CROSS JOIN (SELECT * FROM fits WHERE rep=0) f
),
health AS MATERIALIZED (
 SELECT CASE WHEN COUNT(*)!=501 OR COUNT(*) FILTER(WHERE sample_n<=group_copies+6 OR group_copies!=(SELECT g FROM params) OR intercept IS NULL OR NOT isfinite(intercept) OR lnhhexp IS NULL OR NOT isfinite(lnhhexp) OR age IS NULL OR NOT isfinite(age) OR sex IS NULL OR NOT isfinite(sex) OR hhsize IS NULL OR NOT isfinite(hhsize) OR farm IS NULL OR NOT isfinite(farm) OR educ IS NULL OR NOT isfinite(educ))>0
 OR MAX(native.bad)>0 OR MAX(native.xb_error) IS NULL OR NOT isfinite(MAX(native.xb_error)) OR MAX(native.xb_error)>1e-8 THEN error('FE bootstrap refit or native original fit failed validation') ELSE COUNT(*) END AS valid
 FROM fits CROSS JOIN native
)
SELECT d.rep AS replicate,d.position AS copy_id,d.step,d.commune,f.sample_n,f.intercept,f.lnhhexp,f.age,f.sex,f.hhsize,f.farm,f.educ,
 o.intercept AS estimate_intercept,o.lnhhexp AS estimate_lnhhexp,o.age AS estimate_age,o.sex AS estimate_sex,o.hhsize AS estimate_hhsize,o.farm AS estimate_farm,o.educ AS estimate_educ,
 (SELECT g FROM params) AS communes_per_draw,10001 AS base_seed
FROM draws d JOIN fits f ON d.rep=f.rep CROSS JOIN (SELECT * FROM fits WHERE rep=0) o CROSS JOIN health
WHERE health.valid=501 ORDER BY replicate,copy_id
""" into household_fe_bootstrap_all
sql """SELECT DISTINCT replicate,sample_n,intercept,lnhhexp,age,sex,hhsize,farm,educ,estimate_intercept,estimate_lnhhexp,estimate_age,estimate_sex,estimate_hhsize,estimate_farm,estimate_educ,communes_per_draw,base_seed FROM active ORDER BY replicate""" into household_fe_bootstrap_fits
sql """
WITH vals AS MATERIALIZED (
 SELECT replicate,communes_per_draw,base_seed,
 ['_cons','lnhhexp','age','sex','hhsize','farm','educ'] AS names,
 [intercept,lnhhexp,age,sex,hhsize,farm,educ] AS values,
 [estimate_intercept,estimate_lnhhexp,estimate_age,estimate_sex,estimate_hhsize,estimate_farm,estimate_educ] AS estimates
 FROM active
), long AS MATERIALIZED (
 SELECT replicate,i,names[i] AS parameter,values[i] AS value,estimates[i] AS estimate,communes_per_draw,base_seed FROM vals CROSS JOIN range(1,8) t(i)
), report AS MATERIALIZED (
 SELECT a.i AS parameter_index,b.i AS covariance_index,a.parameter,b.parameter AS covariance_parameter,
 MAX(a.estimate) AS estimate,AVG(a.value ORDER BY a.replicate) AS bootstrap_mean,AVG(a.value ORDER BY a.replicate)-MAX(a.estimate) AS bias,
 STDDEV_SAMP(a.value ORDER BY a.replicate) AS standard_error,QUANTILE_CONT(a.value,0.025 ORDER BY a.replicate) AS ci_lower,QUANTILE_CONT(a.value,0.975 ORDER BY a.replicate) AS ci_upper,
 COVAR_SAMP(a.value,b.value ORDER BY a.replicate) AS covariance,COUNT(*) AS replications,MAX(a.communes_per_draw) AS communes_per_draw,MAX(a.base_seed) AS base_seed,
 'xoshiro256++/splitmix64/per_replicate_seed_plus_index/u64_rejection_v1' AS rng,'whole_commune_pairs_refit_normalized_level_fe' AS method,'type7_percentile_95' AS interval_method
 FROM long a JOIN long b ON a.replicate=b.replicate GROUP BY a.i,b.i,a.parameter,b.parameter
), summary_health AS (
 SELECT CASE WHEN COUNT(*)!=49 OR COUNT(*) FILTER(WHERE replications!=500 OR estimate IS NULL OR NOT isfinite(estimate) OR bootstrap_mean IS NULL OR NOT isfinite(bootstrap_mean) OR bias IS NULL OR NOT isfinite(bias) OR standard_error IS NULL OR NOT isfinite(standard_error) OR standard_error<0 OR ci_lower IS NULL OR NOT isfinite(ci_lower) OR ci_upper IS NULL OR NOT isfinite(ci_upper) OR covariance IS NULL OR NOT isfinite(covariance))>0 THEN error('FE bootstrap summary has missing or nonfinite uncertainty') ELSE COUNT(*) END AS valid FROM report
)
SELECT report.* FROM report CROSS JOIN summary_health WHERE valid=49 ORDER BY parameter_index,covariance_index
""" into household_fe_bootstrap_summary
sql """SELECT parameter,estimate,bootstrap_mean,bias,standard_error,ci_lower,ci_upper FROM active WHERE covariance_index=1 ORDER BY parameter_index"""
export household_fe_bootstrap_summary.csv
use household_fe_bootstrap_fits
export household_fe_bootstrap_fits.csv
use household_fe_bootstrap_all
sql """SELECT replicate,copy_id,step,commune FROM active ORDER BY replicate,copy_id""" into household_fe_bootstrap_draws
export household_fe_bootstrap_draws.csv
use vietnam.csv
sql """SELECT * FROM active WHERE lnexp12m IS NOT NULL ORDER BY source_row""" into household_fe_bootstrap_prediction_cohort
predict fe_xb, xb
predict fe_xbu, xbu
predict fe_u, u
predict fe_residual, residuals
summarize fe_xb fe_xbu fe_u fe_residual
sql """SELECT 'Whole-commune FE pairs; copied groups have distinct identities and observation-weighted effects. B=500 has Monte Carlo error; native stream differs from Stata. Unweighted, not survey-design or causal inference.' AS bootstrap_scope"""
export household_fe_bootstrap_predictions.csv
