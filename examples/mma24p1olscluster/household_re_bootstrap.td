# Chapter24 unbalanced commune RE bootstrap. source_row orders cross-sectional households; it is not time.
use vietnam.csv
isid source_row
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(*)>5999 OR COUNT(*) FILTER (WHERE typeof(source_row)!='BIGINT' OR source_row IS NULL OR source_row<1 OR source_row>5999 OR typeof(commune)!='BIGINT' OR commune IS NULL OR commune<1 OR commune>194 OR age IS NULL OR NOT isfinite(age) OR abs(age)>1000000 OR sex IS NULL OR NOT isfinite(sex) OR abs(sex)>1000000 OR hhsize IS NULL OR NOT isfinite(hhsize) OR abs(hhsize)>1000000 OR farm IS NULL OR NOT isfinite(farm) OR abs(farm)>1000000 OR educ IS NULL OR NOT isfinite(educ) OR abs(educ)>1000000 OR lnhhexp IS NULL OR NOT isfinite(lnhhexp) OR abs(lnhhexp)>1000000 OR (lnexp12m IS NOT NULL AND (NOT isfinite(lnexp12m) OR abs(lnexp12m)>1000000)))>0 THEN error('invalid household source, model values or commune key') ELSE COUNT(*) END AS source_rows, COUNT(*) FILTER (WHERE lnexp12m IS NULL) AS missing_outcomes FROM active"""
sql """SELECT * FROM active WHERE lnexp12m IS NOT NULL ORDER BY source_row""" into household_re_bootstrap_cohort
sql """SELECT CASE WHEN COUNT(*)<8 OR COUNT(*)>194 OR MAX(n)>39 OR SUM(n)<=COUNT(*)+6 THEN error('RE bootstrap requires 8..194 communes, each size<=39, and positive original residual DF') ELSE COUNT(*) END AS communes, SUM(n) AS observations, MIN(n) AS minimum_size, MAX(n) AS maximum_size FROM (SELECT commune,COUNT(*) AS n FROM active GROUP BY commune)"""
xtreg lnexp12m lnhhexp age sex hhsize farm educ, re id(commune) time(source_row) vcmethod(equal_group_between)
predict re_xb, xb
predict re_residual, residuals
sql """
WITH RECURSIVE
source AS MATERIALIZED (SELECT * FROM active),
-- Within and Between moments are kept separate: each repeated source commune
-- represents a distinct copied unit in every variance-component denominator.
means AS MATERIALIZED (
 SELECT commune,COUNT(*) AS n,AVG(lnexp12m ORDER BY source_row) AS my,
 [AVG(lnhhexp ORDER BY source_row),AVG(age ORDER BY source_row),AVG(sex ORDER BY source_row),AVG(hhsize ORDER BY source_row),AVG(farm ORDER BY source_row),AVG(educ ORDER BY source_row)] AS mx
 FROM source GROUP BY commune
),
vector_source AS MATERIALIZED (SELECT commune,source_row,lnexp12m,[lnhhexp::DOUBLE,age::DOUBLE,sex::DOUBLE,hhsize::DOUBLE,farm::DOUBLE,educ::DOUBLE] AS x FROM source),
units AS MATERIALIZED (
 SELECT commune,i,j,SUM((x[i]-mx[i])*(x[j]-mx[j]) ORDER BY source_row) AS wxx,SUM((x[i]-mx[i])*(lnexp12m-my) ORDER BY source_row) AS wxy,SUM(POWER(lnexp12m-my,2) ORDER BY source_row) AS wyy
 FROM vector_source JOIN means USING(commune) CROSS JOIN range(1,7) ri(i) CROSS JOIN range(1,7) rj(j) GROUP BY commune,i,j
),
unit_arrays AS MATERIALIZED (
 SELECT commune,LIST(wxx ORDER BY i,j) AS mat,LIST(wxy ORDER BY i) FILTER(WHERE j=1) AS rhs,MAX(wyy) AS wyy FROM units GROUP BY commune
),
groups AS MATERIALIZED (
 SELECT commune,ROW_NUMBER() OVER (ORDER BY commune)-1 AS index0, n,my,mx,mat,rhs,wyy
 FROM unit_arrays JOIN means USING(commune)
),
params AS MATERIALIZED (SELECT CASE WHEN COUNT(*)<8 OR COUNT(*)>194 THEN error('RE bootstrap requires 8..194 retained communes') ELSE COUNT(*)::BIGINT END AS g,18446744073709551615::UHUGEINT AS mask FROM groups),
expansion AS (SELECT rep,k,((10001::UHUGEINT+rep::UHUGEINT-1+k::UHUGEINT*11400714819323198485::UHUGEINT)&mask) AS z FROM params CROSS JOIN range(1,501) t(rep) CROSS JOIN range(1,5) q(k)),
mix1 AS (SELECT rep,k,(xor(z,z>>30)*13787848793156543929::UHUGEINT)&mask AS z FROM expansion CROSS JOIN params),
mix2 AS (SELECT rep,k,(xor(z,z>>27)*10723151780598845931::UHUGEINT)&mask AS z FROM mix1 CROSS JOIN params),
seeds AS (SELECT rep,MAX(xor(z,z>>31)) FILTER(WHERE k=1) AS s0,MAX(xor(z,z>>31)) FILTER(WHERE k=2) AS s1,MAX(xor(z,z>>31)) FILTER(WHERE k=3) AS s2,MAX(xor(z,z>>31)) FILTER(WHERE k=4) AS s3 FROM mix2 GROUP BY rep),
rng(rep,step,accepted,s0,s1,s2,s3,v) AS (
 SELECT rep,0::BIGINT,0::BIGINT,s0,s1,s2,s3,NULL::UHUGEINT FROM seeds
 UNION ALL
 SELECT rep,step+1,accepted+CASE WHEN v_next<zone THEN 1 ELSE 0 END,
 xor(s0,xor(s3,s1)),xor(s1,xor(s2,s0)),xor(xor(s2,s0),(s1<<17)&mask),((xor(s3,s1)<<45)|(xor(s3,s1)>>19))&mask,v_next
 FROM rng CROSS JOIN params CROSS JOIN LATERAL (SELECT mask-(mask%g::UHUGEINT) AS zone,(s0+s3)&mask AS a) q
 CROSS JOIN LATERAL (SELECT ((((a<<23)|(a>>41))&mask)+s0)&mask AS v_next) v WHERE accepted<g AND step<g+1024
),
rng_health AS (SELECT CASE WHEN COUNT(*)!=500 OR MIN(n)!=(SELECT g FROM params) THEN error('RE bootstrap RNG bound exhausted') ELSE COUNT(*) END AS valid FROM (SELECT rep,MAX(accepted) AS n FROM rng GROUP BY rep)),
draws AS MATERIALIZED (SELECT rep,accepted AS position,step,(v%g::UHUGEINT)::BIGINT+1 AS idx FROM rng CROSS JOIN params CROSS JOIN rng_health WHERE v IS NOT NULL AND v<mask-(mask%g::UHUGEINT) AND valid=500),
all_groups AS MATERIALIZED (
 SELECT LIST(STRUCT_PACK(n:=n,my:=my,mx:=mx,mat:=mat,rhs:=rhs,wyy:=wyy) ORDER BY commune) AS gs FROM groups
),
rep_indices AS MATERIALIZED (SELECT rep,LIST(idx ORDER BY position) AS ids FROM draws GROUP BY rep),
samples AS MATERIALIZED (SELECT rep,ids,gs FROM rep_indices CROSS JOIN all_groups UNION ALL SELECT 0 AS rep,range(1,(SELECT g FROM params)+1) AS ids,gs FROM all_groups),
within_moments AS MATERIALIZED (
 SELECT rep,ids,gs,
 LIST_TRANSFORM(range(1,37),i -> LIST_SUM(LIST_TRANSFORM(ids,j -> gs[j].mat[i]))) AS mat,
 LIST_TRANSFORM(range(1,7),i -> LIST_SUM(LIST_TRANSFORM(ids,j -> gs[j].rhs[i]))) AS rhs,
 LIST_SUM(LIST_TRANSFORM(ids,j -> gs[j].wyy)) AS wyy,
 LIST_SUM(LIST_TRANSFORM(ids,j -> gs[j].n)) AS sample_n,
 LIST_SUM(LIST_TRANSFORM(ids,j -> gs[j].my))/len(ids) AS my,
 LIST_TRANSFORM(range(1,7),i -> LIST_SUM(LIST_TRANSFORM(ids,j -> gs[j].mx[i]))/len(ids)) AS mx
 FROM samples
),
between_moments AS MATERIALIZED (
 SELECT rep,LIST_TRANSFORM(range(1,37),k -> LIST_SUM(LIST_TRANSFORM(ids,j -> (gs[j].mx[(k-1)//6+1]-mx[(k-1)//6+1])*(gs[j].mx[(k-1)%6+1]-mx[(k-1)%6+1])))) AS mat,
 LIST_TRANSFORM(range(1,7),i -> LIST_SUM(LIST_TRANSFORM(ids,j -> (gs[j].mx[i]-mx[i])*(gs[j].my-my)))) AS rhs
 FROM within_moments
),
first_matrices AS MATERIALIZED (
 SELECT rep,'within' AS kind,mat,rhs FROM within_moments UNION ALL SELECT rep,'between' AS kind,mat,rhs FROM between_moments
),
-- Standardize both Gram systems to unit diagonal. Reject weak Schur pivots;
-- this conservative guard is not a general condition-number certificate.
first_scaling AS MATERIALIZED (
 SELECT rep,kind,mat,rhs,LIST_TRANSFORM(range(1,7),i -> CASE WHEN mat[(i-1)*6+i] IS NULL OR NOT isfinite(mat[(i-1)*6+i]) OR mat[(i-1)*6+i]<=0 THEN error('RE bootstrap zero/nonfinite variation') ELSE sqrt(mat[(i-1)*6+i]) END) AS scale FROM first_matrices
),
first_start AS MATERIALIZED (
 SELECT rep,kind,scale,LIST_TRANSFORM(range(1,43),k -> CASE WHEN (k-1)%7+1=7 THEN rhs[(k-1)//7+1]/scale[(k-1)//7+1] ELSE mat[((k-1)//7)*6+(k-1)%7+1]/(scale[(k-1)//7+1]*scale[(k-1)%7+1]) END) AS a FROM first_scaling
),
first_solver(rep,kind,step,scale,a) AS (
 SELECT rep,kind,1,scale,a FROM first_start UNION ALL
 SELECT rep,kind,step+1,scale,LIST_TRANSFORM(range(1,43),k -> CASE WHEN (k-1)//7+1=step THEN a[k]/denom ELSE a[k]-a[((k-1)//7)*7+step]*a[(step-1)*7+(k-1)%7+1]/denom END)
 FROM first_solver CROSS JOIN LATERAL (SELECT CASE WHEN a[(step-1)*7+step] IS NULL OR NOT isfinite(a[(step-1)*7+step]) OR a[(step-1)*7+step]<=1e-3 THEN error('RE bootstrap unidentified or ill-conditioned design') ELSE a[(step-1)*7+step] END AS denom) p WHERE step<=6
),
first_slopes AS MATERIALIZED (SELECT rep,kind,LIST_TRANSFORM(range(1,7),i -> a[(i-1)*7+7]/scale[i]) AS b FROM first_solver WHERE step=7),
slopes AS MATERIALIZED (
 SELECT a.rep,a.b AS bw,b.b AS bb FROM first_slopes a JOIN first_slopes b ON a.rep=b.rep WHERE a.kind='within' AND b.kind='between'
),
rss AS NOT MATERIALIZED (
 SELECT w.*,s.bw,s.bb,
 -- Reject strong cancellation in the residual quadratic below; no clipping.
 w.wyy-2*LIST_SUM(LIST_TRANSFORM(range(1,7),i -> s.bw[i]*w.rhs[i]))+
 LIST_SUM(LIST_TRANSFORM(range(1,37),k -> s.bw[(k-1)//6+1]*w.mat[k]*s.bw[(k-1)%6+1])) AS within_rss,
 LIST_SUM(LIST_TRANSFORM(ids,j -> POWER(gs[j].my-my-LIST_SUM(LIST_TRANSFORM(range(1,7),i -> (gs[j].mx[i]-mx[i])*s.bb[i])),2))) AS between_rss,
 LIST_SUM(LIST_TRANSFORM(ids,j -> 1.0/gs[j].n))/len(ids) AS mean_inverse_n
 FROM within_moments w JOIN slopes s USING(rep)
),
variances AS NOT MATERIALIZED (
 SELECT *,CASE WHEN within_rss IS NULL OR NOT isfinite(within_rss) OR within_rss<=1e-4*wyy OR sample_n<=(SELECT g FROM params)+6 OR between_rss IS NULL OR NOT isfinite(between_rss) OR between_rss<0 THEN error('RE bootstrap invalid or unstable residual moments') ELSE within_rss/(sample_n-(SELECT g FROM params)-6) END AS e FROM rss
),
raw_component AS NOT MATERIALIZED (SELECT *,between_rss/((SELECT g FROM params)-7)-e*mean_inverse_n AS raw_u FROM variances),
components AS NOT MATERIALIZED (SELECT *,CASE WHEN raw_u IS NULL OR NOT isfinite(raw_u) OR e IS NULL OR NOT isfinite(e) OR e<=0 THEN error('RE bootstrap invalid variance components') ELSE GREATEST(raw_u,0) END AS u FROM raw_component),
weights AS NOT MATERIALIZED (
 SELECT *,LIST_TRANSFORM(ids,j -> CASE WHEN e/(e+gs[j].n*u) IS NULL OR NOT isfinite(e/(e+gs[j].n*u)) OR e/(e+gs[j].n*u)<=1e-24 THEN error('RE bootstrap loses transformed intercept precision') ELSE gs[j].n*e/(e+gs[j].n*u) END) AS weights FROM components
),
weighted_means AS MATERIALIZED (
 SELECT *,LIST_SUM(weights) AS total_weight,
 LIST_SUM(LIST_TRANSFORM(range(1,len(ids)+1),k -> weights[k]*gs[ids[k]].my))/LIST_SUM(weights) AS wmy,
 LIST_TRANSFORM(range(1,7),i -> LIST_SUM(LIST_TRANSFORM(range(1,len(ids)+1),k -> weights[k]*gs[ids[k]].mx[i]))/LIST_SUM(weights)) AS wmx
 FROM weights
),
gls_matrices AS MATERIALIZED (
 SELECT rep,LIST_TRANSFORM(range(1,37),z -> mat[z]+LIST_SUM(LIST_TRANSFORM(range(1,len(ids)+1),k -> weights[k]*(gs[ids[k]].mx[(z-1)//6+1]-wmx[(z-1)//6+1])*(gs[ids[k]].mx[(z-1)%6+1]-wmx[(z-1)%6+1])))) AS mat,
 LIST_TRANSFORM(range(1,7),i -> rhs[i]+LIST_SUM(LIST_TRANSFORM(range(1,len(ids)+1),k -> weights[k]*(gs[ids[k]].mx[i]-wmx[i])*(gs[ids[k]].my-wmy)))) AS rhs
 FROM weighted_means
),
gls_scaling AS MATERIALIZED (
 SELECT rep,mat,rhs,LIST_TRANSFORM(range(1,7),i -> CASE WHEN mat[(i-1)*6+i] IS NULL OR NOT isfinite(mat[(i-1)*6+i]) OR mat[(i-1)*6+i]<=0 THEN error('RE bootstrap zero/nonfinite variation') ELSE sqrt(mat[(i-1)*6+i]) END) AS scale FROM gls_matrices
),
gls_start AS MATERIALIZED (
 SELECT rep,scale,LIST_TRANSFORM(range(1,43),k -> CASE WHEN (k-1)%7+1=7 THEN rhs[(k-1)//7+1]/scale[(k-1)//7+1] ELSE mat[((k-1)//7)*6+(k-1)%7+1]/(scale[(k-1)//7+1]*scale[(k-1)%7+1]) END) AS a FROM gls_scaling
),
gls_solver(rep,step,scale,a) AS (
 SELECT rep,1,scale,a FROM gls_start UNION ALL
 SELECT rep,step+1,scale,LIST_TRANSFORM(range(1,43),k -> CASE WHEN (k-1)//7+1=step THEN a[k]/denom ELSE a[k]-a[((k-1)//7)*7+step]*a[(step-1)*7+(k-1)%7+1]/denom END)
 FROM gls_solver CROSS JOIN LATERAL (SELECT CASE WHEN a[(step-1)*7+step] IS NULL OR NOT isfinite(a[(step-1)*7+step]) OR a[(step-1)*7+step]<=1e-3 THEN error('RE bootstrap unidentified or ill-conditioned design') ELSE a[(step-1)*7+step] END AS denom) p WHERE step<=6
),
gls_slopes AS MATERIALIZED (SELECT rep,LIST_TRANSFORM(range(1,7),i -> a[(i-1)*7+7]/scale[i]) AS b FROM gls_solver WHERE step=7),
fits AS MATERIALIZED (
 SELECT rep,sample_n::BIGINT AS sample_n,e,raw_u,u,within_rss,between_rss,mean_inverse_n,
 1-sqrt(LIST_MAX(LIST_TRANSFORM(range(1,len(ids)+1),k -> weights[k]/gs[ids[k]].n))) AS theta_min,
 1-sqrt(LIST_MIN(LIST_TRANSFORM(range(1,len(ids)+1),k -> weights[k]/gs[ids[k]].n))) AS theta_max,
 wmy-LIST_SUM(LIST_TRANSFORM(range(1,7),i -> wmx[i]*b[i])) AS intercept,
 b[1] AS lnhhexp,b[2] AS age,b[3] AS sex,b[4] AS hhsize,b[5] AS farm,b[6] AS educ,
 bw[1] AS within_lnhhexp,bw[2] AS within_age,bw[3] AS within_sex,bw[4] AS within_hhsize,bw[5] AS within_farm,bw[6] AS within_educ,
 bb[1] AS between_lnhhexp,bb[2] AS between_age,bb[3] AS between_sex,bb[4] AS between_hhsize,bb[5] AS between_farm,bb[6] AS between_educ
 FROM weighted_means JOIN gls_slopes USING(rep)
),
native AS MATERIALIZED (
 SELECT MAX(ABS(re_xb-(f.intercept+f.lnhhexp*source.lnhhexp+f.age*source.age+f.sex*source.sex+f.hhsize*source.hhsize+f.farm*source.farm+f.educ*source.educ))) AS xb_error,
 COUNT(*) FILTER(WHERE re_xb IS NULL OR re_residual IS NULL OR NOT isfinite(re_xb) OR NOT isfinite(re_residual)) AS bad
 FROM source CROSS JOIN (SELECT * FROM fits WHERE rep=0) f
),
health AS MATERIALIZED (
 SELECT CASE WHEN COUNT(*)!=501 OR COUNT(*) FILTER(WHERE sample_n<=(SELECT g FROM params)+6 OR e IS NULL OR NOT isfinite(e) OR e<=0 OR raw_u IS NULL OR NOT isfinite(raw_u) OR u IS NULL OR NOT isfinite(u) OR u<0 OR within_rss IS NULL OR NOT isfinite(within_rss) OR between_rss IS NULL OR NOT isfinite(between_rss) OR mean_inverse_n IS NULL OR NOT isfinite(mean_inverse_n) OR theta_min IS NULL OR NOT isfinite(theta_min) OR theta_min<0 OR theta_max IS NULL OR NOT isfinite(theta_max) OR theta_max>=1 OR intercept IS NULL OR NOT isfinite(intercept) OR lnhhexp IS NULL OR NOT isfinite(lnhhexp) OR age IS NULL OR NOT isfinite(age) OR sex IS NULL OR NOT isfinite(sex) OR hhsize IS NULL OR NOT isfinite(hhsize) OR farm IS NULL OR NOT isfinite(farm) OR educ IS NULL OR NOT isfinite(educ))>0
 OR MAX(native.bad)>0 OR MAX(native.xb_error) IS NULL OR NOT isfinite(MAX(native.xb_error)) OR MAX(native.xb_error)>1e-8 THEN error('RE bootstrap refit or native original fit failed validation') ELSE COUNT(*) END AS valid
 FROM fits CROSS JOIN native
)
SELECT d.rep AS replicate,d.position AS copy_id,d.step,groups.commune,f.* EXCLUDE(rep),
 o.intercept AS estimate_intercept,o.lnhhexp AS estimate_lnhhexp,o.age AS estimate_age,o.sex AS estimate_sex,o.hhsize AS estimate_hhsize,o.farm AS estimate_farm,o.educ AS estimate_educ,
 (SELECT g FROM params) AS communes_per_draw,10001 AS base_seed
FROM draws d JOIN groups ON groups.index0=d.idx-1 JOIN fits f ON d.rep=f.rep CROSS JOIN (SELECT * FROM fits WHERE rep=0) o CROSS JOIN health
WHERE health.valid=501 ORDER BY replicate,copy_id
""" into household_re_bootstrap_all
sql """SELECT DISTINCT replicate,sample_n,e,raw_u,u,within_rss,between_rss,mean_inverse_n,theta_min,theta_max,within_lnhhexp,within_age,within_sex,within_hhsize,within_farm,within_educ,between_lnhhexp,between_age,between_sex,between_hhsize,between_farm,between_educ,intercept,lnhhexp,age,sex,hhsize,farm,educ,estimate_intercept,estimate_lnhhexp,estimate_age,estimate_sex,estimate_hhsize,estimate_farm,estimate_educ,communes_per_draw,base_seed FROM active ORDER BY replicate""" into household_re_bootstrap_fits
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
 'xoshiro256++/splitmix64/per_replicate_seed_plus_index/u64_rejection_v1' AS rng,'whole_commune_pairs_refit_equal_group_re_gls' AS method,'type7_percentile_95' AS interval_method
 FROM long a JOIN long b ON a.replicate=b.replicate GROUP BY a.i,b.i,a.parameter,b.parameter
), summary_health AS (
 SELECT CASE WHEN COUNT(*)!=49 OR COUNT(*) FILTER(WHERE replications!=500 OR estimate IS NULL OR NOT isfinite(estimate) OR bootstrap_mean IS NULL OR NOT isfinite(bootstrap_mean) OR bias IS NULL OR NOT isfinite(bias) OR standard_error IS NULL OR NOT isfinite(standard_error) OR standard_error<0 OR ci_lower IS NULL OR NOT isfinite(ci_lower) OR ci_upper IS NULL OR NOT isfinite(ci_upper) OR covariance IS NULL OR NOT isfinite(covariance))>0 THEN error('RE bootstrap summary has missing or nonfinite uncertainty') ELSE COUNT(*) END AS valid FROM report
)
SELECT report.* FROM report CROSS JOIN summary_health WHERE valid=49 ORDER BY parameter_index,covariance_index
""" into household_re_bootstrap_summary
sql """SELECT parameter,estimate,bootstrap_mean,bias,standard_error,ci_lower,ci_upper FROM active WHERE covariance_index=1 ORDER BY parameter_index"""
export household_re_bootstrap_summary.csv
use household_re_bootstrap_fits
export household_re_bootstrap_fits.csv
use household_re_bootstrap_all
sql """SELECT replicate,copy_id,step,commune FROM active ORDER BY replicate,copy_id""" into household_re_bootstrap_draws
export household_re_bootstrap_draws.csv
use vietnam.csv
sql """SELECT * FROM active WHERE lnexp12m IS NOT NULL ORDER BY source_row""" into household_re_bootstrap_prediction_cohort
predict re_xb, xb
predict re_residual, residuals
summarize re_xb re_residual
sql """SELECT 'Whole-commune RE pairs; every distinct copied group refits equal-group Between components and quasi-GLS. B=500 has Monte Carlo error; native stream differs from Stata. Unweighted, not survey-design or causal inference.' AS bootstrap_scope"""
export household_re_bootstrap_predictions.csv
