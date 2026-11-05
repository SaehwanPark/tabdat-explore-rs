# Independent-working Gaussian PA inference composed from a retained pooled OLS fit.
use MOM.csv
isid id year
isid source_row
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(*)>5320 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR source_row IS NULL OR lnhr IS NULL OR lnwg IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR NOT isfinite(source_row) OR NOT isfinite(lnhr) OR NOT isfinite(lnwg) OR id!=floor(id) OR id<1 OR id>532 OR year!=floor(year) OR year<1979 OR year>1988 OR source_row!=floor(source_row) OR source_row<1 OR source_row>5320 OR ABS(lnhr)>20 OR ABS(lnwg)>20)>0 THEN error('independent GEE requires finite bounded MOM model fields and exact keys') ELSE COUNT(*) END AS observations FROM active"""
sql """SELECT CASE WHEN COUNT(*)<3 OR COUNT(*)>532 OR MIN(t)!=10 OR MAX(t)!=10 THEN error('independent GEE requires 3..532 complete ten-year people') ELSE COUNT(*) END AS people FROM (SELECT id,COUNT(*) AS t FROM active GROUP BY id)"""
sql """SELECT CASE WHEN VAR_POP(lnwg) IS NULL OR NOT isfinite(VAR_POP(lnwg)) OR COUNT(*)*VAR_POP(lnwg)<=1e-12 THEN error('independent GEE requires identified wage design') ELSE COUNT(*)*VAR_POP(lnwg) END AS wage_centered_ss FROM active"""
sql """SELECT 'The retained model is pooled OLS; its printed t/F/CR1 inference is not the independent GEE report below. GEE working covariance assumes independent errors; its G/(G-1) unit sandwich assumes independent people. These marginal associations are not causal.' AS inference_scope"""
regress lnhr lnwg
predict pols_mean, xb
predict pols_residual, residuals
sql """SELECT * FROM active ORDER BY source_row""" into independent_gee_source
sql """
WITH moments AS MATERIALIZED (
 SELECT COUNT(*) AS n,COUNT(DISTINCT id) AS g,AVG(lnwg ORDER BY source_row) AS mx,
 REGR_SLOPE(pols_mean,lnwg ORDER BY source_row) AS b,REGR_INTERCEPT(pols_mean,lnwg ORDER BY source_row) AS a,
 SUM(pols_residual*pols_residual ORDER BY source_row) AS rss,
 COUNT(*)*VAR_POP(lnwg ORDER BY source_row) AS sxx FROM active
), checked AS MATERIALIZED (
 SELECT * EXCLUDE(rss),CASE WHEN a IS NULL OR b IS NULL OR NOT isfinite(a) OR NOT isfinite(b) OR ABS(a)>100 OR ABS(b)>100
 OR rss IS NULL OR NOT isfinite(rss) OR rss<=1e-12 OR sxx IS NULL OR NOT isfinite(sxx) OR sxx<=1e-12
 THEN error('independent GEE requires finite identified fit and positive residual scale') ELSE rss END AS rss FROM moments
), scores AS MATERIALIZED (
 SELECT id,SUM(pols_residual ORDER BY year) AS u0,SUM((lnwg-mx)*pols_residual ORDER BY year) AS u1,
 MAX(n) AS n,MAX(g) AS g,MAX(mx) AS mx,MAX(a) AS a,MAX(b) AS b,MAX(rss) AS rss,MAX(sxx) AS sxx
 FROM active CROSS JOIN checked GROUP BY id
), influences AS MATERIALIZED (
 SELECT *,u0/n-mx*u1/sxx AS h0,u1/sxx AS h1 FROM scores
), aggregate AS (
 SELECT MAX(n) AS n,MAX(g) AS g,MAX(mx) AS mx,MAX(a) AS a,MAX(b) AS b,MAX(rss) AS rss,MAX(sxx) AS sxx,
 SUM(h0*h0 ORDER BY id) AS c00,SUM(h0*h1 ORDER BY id) AS c01,SUM(h1*h1 ORDER BY id) AS c11,
 SUM(h0 ORDER BY id) AS step0,SUM(h1 ORDER BY id) AS step1 FROM influences
), result AS (
 SELECT *,rss/n AS phi,(rss/n)*(1.0/n+mx*mx/sxx) AS w00,-(rss/n)*mx/sxx AS w01,(rss/n)/sxx AS w11,
 c00*g/(g-1.0) AS v00,c01*g/(g-1.0) AS v01,c11*g/(g-1.0) AS v11 FROM aggregate
)
SELECT 1 AS gee_join_key,n AS observations,g AS people,10 AS years_per_person,a AS intercept,b AS slope,mx AS covariate_mean,
 rss,sxx AS wage_centered_ss,phi AS scale,0.0 AS working_correlation,
 w00 AS working_covariance_aa,w01 AS working_covariance_ab,w11 AS working_covariance_bb,
 v00 AS unit_covariance_aa,v01 AS unit_covariance_ab,v11 AS unit_covariance_bb,
 CASE WHEN NOT isfinite(w00) OR w00<=0 OR NOT isfinite(w01) OR NOT isfinite(w11) OR w11<=0
 OR NOT isfinite(v00) OR v00<=2.842170943040401e-14*w00 OR NOT isfinite(v01) OR NOT isfinite(v11) OR v11<=2.842170943040401e-14*w11
 OR NOT isfinite(step0) OR NOT isfinite(step1) OR ABS(step0)>1e-10*(1+ABS(a)) OR ABS(step1)>1e-10*(1+ABS(b))
 THEN error('independent GEE failed covariance or estimating-score certificate') ELSE step0 END AS intercept_step,
 step1 AS slope_step,'rss_over_n_independent' AS convention,'g_over_g_minus_1_unit_sandwich' AS unit_covariance,
 'normal_z_chi2_1' AS inference FROM result
""" into independent_gee_fit
export panel_independent_gee_fit.csv
sql """
WITH methods AS (
 SELECT 'working_model' AS covariance,intercept,slope,working_covariance_aa AS va,working_covariance_bb AS vb FROM active UNION ALL
 SELECT 'unit_sandwich_g_adjusted',intercept,slope,unit_covariance_aa,unit_covariance_bb FROM active
), parameters AS (
 SELECT covariance,'_cons' AS parameter,intercept AS estimate,SQRT(va) AS standard_error FROM methods UNION ALL
 SELECT covariance,'lnwg',slope,SQRT(vb) FROM methods
), statistics AS (SELECT *,estimate/standard_error AS z FROM parameters)
SELECT covariance,parameter,estimate,standard_error,z,2*tabdat_gauss_phi(-ABS(z)) AS p_value,
 estimate-1.959963984540054*standard_error AS normal95_lower,estimate+1.959963984540054*standard_error AS normal95_upper,
 CASE WHEN parameter='lnwg' THEN z*z ELSE NULL END AS slope_wald_chi2_1 FROM statistics ORDER BY covariance,parameter
""" into independent_gee_coefficients
sql """SELECT * FROM active"""
export panel_independent_gee_coefficients.csv
use independent_gee_source
generate gee_join_key = 1
join independent_gee_fit on gee_join_key
sql """SELECT * FROM active""" into independent_gee_joined
sql """SELECT source_row,lnhr,lnwg,kids,ageh,agesq,disab,id,year,pols_mean,pols_residual,intercept+slope*lnwg AS gee_mean,lnhr-intercept-slope*lnwg AS gee_residual FROM active ORDER BY source_row""" into independent_gee_predictions
summarize gee_mean gee_residual
export panel_independent_gee_predictions.csv
use independent_gee_joined
sql """WITH scores AS (SELECT id,COUNT(*) AS observations,MAX(observations) AS n,MAX(covariate_mean) AS mx,MAX(wage_centered_ss) AS sxx,SUM(pols_residual ORDER BY year) AS u0,SUM((lnwg-covariate_mean)*pols_residual ORDER BY year) AS u1 FROM active GROUP BY id) SELECT id,observations,u0 AS score_intercept,u1 AS score_centered_slope,u0/n-mx*u1/sxx AS influence_intercept,u1/sxx AS influence_slope FROM scores ORDER BY id""" into independent_gee_unit_scores
export panel_independent_gee_unit_scores.csv
use independent_gee_predictions
status
