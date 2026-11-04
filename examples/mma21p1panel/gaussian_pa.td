# Balanced Gaussian identity-link exchangeable GEE; native retained state is pooled OLS.
use MOM.csv
isid id year
isid source_row
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(*)>5320 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR source_row IS NULL OR lnhr IS NULL OR lnwg IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR NOT isfinite(source_row) OR NOT isfinite(lnhr) OR NOT isfinite(lnwg) OR id!=floor(id) OR id<1 OR id>532 OR year!=floor(year) OR year<1979 OR year>1988 OR source_row!=floor(source_row) OR source_row<1 OR source_row>5320 OR ABS(lnhr)>20 OR ABS(lnwg)>20)>0 THEN error('Gaussian PA requires finite bounded MOM model fields and exact keys') ELSE COUNT(*) END AS observations FROM active"""
sql """SELECT CASE WHEN COUNT(*)<3 OR COUNT(*)>532 OR MIN(t)!=10 OR MAX(t)!=10 THEN error('Gaussian PA requires 3..532 people with all ten years 1979–1988') ELSE COUNT(*) END AS people FROM (SELECT id,COUNT(*) AS t FROM active GROUP BY id)"""
regress lnhr lnwg, cluster(id)
predict pols_mean, xb
predict pols_residual, residuals
sql """SELECT * FROM active ORDER BY source_row""" into gaussian_pa_source
sql """
WITH RECURSIVE
source AS MATERIALIZED (SELECT * FROM active),
means AS MATERIALIZED (SELECT id,AVG(lnhr ORDER BY year) AS my,AVG(lnwg ORDER BY year) AS mx FROM source GROUP BY id),
units AS MATERIALIZED (
 SELECT id,MAX(my) AS my,MAX(mx) AS mx,SUM(POWER(lnwg-mx,2) ORDER BY year) AS wxx,
 SUM((lnwg-mx)*(lnhr-my) ORDER BY year) AS wxy FROM source JOIN means USING(id) GROUP BY id
), centers AS (SELECT AVG(my ORDER BY id) AS my,AVG(mx ORDER BY id) AS mx FROM units),
statistics AS MATERIALIZED (
 SELECT COUNT(*) AS g,10*COUNT(*) AS n,MAX(c.my) AS my,MAX(c.mx) AS mx,SUM(wxx ORDER BY id) AS wxx,
 SUM(wxy ORDER BY id) AS wxy,SUM(POWER(u.mx-c.mx,2) ORDER BY id) AS bxx,
 SUM((u.mx-c.mx)*(u.my-c.my) ORDER BY id) AS bxy FROM units u CROSS JOIN centers c
), checked AS MATERIALIZED (
 SELECT * EXCLUDE(wxx),CASE WHEN wxx IS NULL OR bxx IS NULL OR wxy IS NULL OR bxy IS NULL
 OR NOT isfinite(wxx) OR NOT isfinite(bxx) OR NOT isfinite(wxy) OR NOT isfinite(bxy)
 OR wxx<=1e-12 OR bxx<=1e-12 THEN error('Gaussian PA requires identified within and between wage designs') ELSE wxx END AS wxx
 FROM statistics
), start AS (
 SELECT CASE WHEN b IS NULL OR NOT isfinite(b) OR ABS(b)>100 OR ABS(a-(s.my-b*s.mx))>1e-9
 THEN error('invalid native pooled OLS start for Gaussian PA') ELSE b END AS b
 FROM (SELECT REGR_SLOPE(pols_mean,lnwg) AS b,REGR_INTERCEPT(pols_mean,lnwg) AS a FROM source) p CROSS JOIN checked s
), iterate(iteration,b,finished) AS (
 SELECT 0,b,FALSE FROM start
 UNION ALL
 SELECT iteration+1,b_next,ABS(b_next-i.b)<=1e-12*(1+ABS(i.b))
 FROM iterate i CROSS JOIN checked s
 CROSS JOIN LATERAL (
 SELECT SUM(POWER((p.lnhr-u.my)-i.b*(p.lnwg-u.mx),2) ORDER BY p.id,p.year) AS sw
 FROM source p JOIN means u USING(id)
 ) w
 CROSS JOIN LATERAL (
 SELECT 10*SUM(POWER((u.my-s.my)-i.b*(u.mx-s.mx),2) ORDER BY id) AS sb FROM means u
 ) v
 CROSS JOIN LATERAL (
 SELECT CASE WHEN sw IS NULL OR sb IS NULL OR NOT isfinite(sw+sb)
 OR sw<=2.842170943040401e-14*(sw+sb) OR sb<=2.842170943040401e-14*(sw+sb) OR sw+sb<=1e-12
 THEN error('Gaussian PA working correlation is singular or unresolved') ELSE sw/(9*sb) END AS q
 ) correlation
 CROSS JOIN LATERAL (SELECT (wxy+10*q*bxy)/(wxx+10*q*bxx) AS raw_b) step
 CROSS JOIN LATERAL (SELECT CASE WHEN raw_b IS NULL OR NOT isfinite(raw_b) OR ABS(raw_b)>100
 THEN error('Gaussian PA coefficient step is unrepresentable') ELSE raw_b END AS b_next) valid_step
 WHERE NOT finished AND iteration<100
), final AS MATERIALIZED (SELECT * FROM iterate ORDER BY iteration DESC LIMIT 1),
residual_moments AS MATERIALIZED (
 SELECT f.*,s.*,s.my-f.b*s.mx AS a,
 (SELECT SUM(POWER((p.lnhr-u.my)-f.b*(p.lnwg-u.mx),2) ORDER BY p.id,p.year) FROM source p JOIN means u USING(id)) AS sw,
 (SELECT 10*SUM(POWER((u.my-s.my)-f.b*(u.mx-s.mx),2) ORDER BY id) FROM means u) AS sb
 FROM final f CROSS JOIN checked s
), variances AS MATERIALIZED (
 SELECT * EXCLUDE(sw),CASE WHEN NOT finished OR iteration>=100 OR NOT isfinite(a) OR ABS(a)>100
 OR NOT isfinite(sw+sb) OR sw+sb<=1e-12 OR sw<=2.842170943040401e-14*(sw+sb) OR sb<=2.842170943040401e-14*(sw+sb)
 THEN error('Gaussian PA failed convergence or final positive-definiteness') ELSE sw END AS sw
 FROM residual_moments
), parameters AS MATERIALIZED (
 SELECT *,sw/(9*g) AS e,sb/g AS v,(sw+sb)/n AS phi,(9*sb-sw)/(9*(sw+sb)) AS rho,
 wxx/(sw/(9*g))+10*bxx/(sb/g) AS d FROM variances
), influences AS MATERIALIZED (
 SELECT u.id,10*(u.my-p.my-p.b*(u.mx-p.mx))/v AS score_intercept,
 (u.wxy-p.b*u.wxx)/e+10*(u.mx-p.mx)*(u.my-p.my-p.b*(u.mx-p.mx))/v AS score_centered_slope,
 p.* FROM units u CROSS JOIN parameters p
), aggregate_scores AS (
 SELECT MAX(n) AS n,MAX(g) AS g,MAX(iteration) AS iterations,MAX(a) AS a,MAX(b) AS b,MAX(mx) AS mx,
 MAX(e) AS e,MAX(v) AS v,MAX(phi) AS phi,MAX(rho) AS rho,MAX(d) AS d,
 SUM(POWER((v/n)*score_intercept-mx*score_centered_slope/d,2) ORDER BY id) AS c00,
 SUM(((v/n)*score_intercept-mx*score_centered_slope/d)*(score_centered_slope/d) ORDER BY id) AS c01,
 SUM(POWER(score_centered_slope/d,2) ORDER BY id) AS c11,
 SUM(score_intercept ORDER BY id) AS u0,SUM(score_centered_slope ORDER BY id) AS u1 FROM influences
), result AS (
 SELECT *,v/n+mx*mx/d AS w00,-mx/d AS w01,1/d AS w11,
 (v/n)*u0-mx*u1/d AS intercept_step,u1/d AS slope_step FROM aggregate_scores
)
SELECT 1 AS gee_join_key,n AS observations,g AS people,10 AS years_per_person,iterations,a AS intercept,b AS slope,mx AS covariate_mean,
 phi AS scale,rho AS working_correlation,e AS within_eigenvariance,v AS between_eigenvariance,
 w00 AS working_covariance_aa,w01 AS working_covariance_ab,w11 AS working_covariance_bb,
 c00 AS unit_covariance_aa,c01 AS unit_covariance_ab,c11 AS unit_covariance_bb,
 CASE WHEN NOT isfinite(d) OR d<=1e-12 OR NOT isfinite(w00) OR w00<=0 OR NOT isfinite(w01) OR NOT isfinite(w11) OR w11<=0
 OR NOT isfinite(c00) OR c00<=0 OR NOT isfinite(c01) OR NOT isfinite(c11) OR c11<=0
 OR NOT isfinite(intercept_step) OR NOT isfinite(slope_step) OR ABS(intercept_step)>1e-10*(1+ABS(a)) OR ABS(slope_step)>1e-10*(1+ABS(b))
 OR NOT isfinite(phi) OR NOT isfinite(rho) OR rho<=-1.0/9 OR rho>=1
 THEN error('Gaussian PA failed bread, covariance or estimating-score certificate') ELSE intercept_step END AS intercept_step,
 slope_step,'legacy_pearson_exchangeable_rss_over_n' AS convention,
 'uncorrected_unit_score_sandwich' AS unit_covariance,'normal_95_no_pvalues' AS inference
FROM result
""" into gaussian_pa_fit
export panel_gaussian_pa_fit.csv
sql """WITH r AS (SELECT 'working_model' AS covariance,intercept,slope,working_covariance_aa AS va,working_covariance_ab AS cab,working_covariance_bb AS vb FROM active UNION ALL SELECT 'uncorrected_unit_sandwich',intercept,slope,unit_covariance_aa,unit_covariance_ab,unit_covariance_bb FROM active) SELECT covariance,'_cons' AS parameter,intercept AS estimate,SQRT(va) AS standard_error,intercept/SQRT(va) AS z,intercept-1.959963984540054*SQRT(va) AS normal95_lower,intercept+1.959963984540054*SQRT(va) AS normal95_upper FROM r UNION ALL SELECT covariance,'lnwg',slope,SQRT(vb),slope/SQRT(vb),slope-1.959963984540054*SQRT(vb),slope+1.959963984540054*SQRT(vb) FROM r ORDER BY covariance,parameter""" into gaussian_pa_coefficients
sql """SELECT * FROM active"""
export panel_gaussian_pa_coefficients.csv
use gaussian_pa_source
generate gee_join_key = 1
join gaussian_pa_fit on gee_join_key
sql """SELECT * FROM active""" into gaussian_pa_joined
sql """SELECT source_row,lnhr,lnwg,kids,ageh,agesq,disab,id,year,pols_mean,pols_residual,intercept+slope*lnwg AS gee_mean,lnhr-(intercept+slope*lnwg) AS gee_residual FROM active ORDER BY source_row""" into gaussian_pa_predictions
summarize gee_mean gee_residual
export panel_gaussian_pa_predictions.csv
use gaussian_pa_joined
sql """WITH centered AS (SELECT *,AVG(lnhr) OVER (PARTITION BY id) AS my,AVG(lnwg) OVER (PARTITION BY id) AS mx FROM active), scores AS (SELECT id,COUNT(*) AS observations,MAX(centered.observations) AS n,MAX(between_eigenvariance) AS v,MAX(working_covariance_bb) AS vb,MAX(covariate_mean) AS grand_x,10*AVG(lnhr-intercept-slope*lnwg)/MAX(between_eigenvariance) AS u0,SUM((lnwg-mx)*((lnhr-my)-slope*(lnwg-mx)) ORDER BY year)/MAX(within_eigenvariance)+10*(MAX(mx)-MAX(covariate_mean))*AVG(lnhr-intercept-slope*lnwg)/MAX(between_eigenvariance) AS u1 FROM centered GROUP BY id) SELECT id,observations,u0 AS score_intercept,u1 AS score_centered_slope,(v/n)*u0-grand_x*vb*u1 AS influence_intercept,vb*u1 AS influence_slope FROM scores ORDER BY id""" into gaussian_pa_unit_scores
export panel_gaussian_pa_unit_scores.csv
use gaussian_pa_predictions
status
