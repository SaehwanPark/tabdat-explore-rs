# Gaussian identity-link AR1 working GEE; retained model is pooled OLS.
use MOM.csv
isid id year
isid source_row
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(*)>5320 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR source_row IS NULL OR lnhr IS NULL OR lnwg IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR NOT isfinite(source_row) OR NOT isfinite(lnhr) OR NOT isfinite(lnwg) OR id!=floor(id) OR id<1 OR id>532 OR year!=floor(year) OR year<1979 OR year>1988 OR source_row!=floor(source_row) OR source_row<1 OR source_row>5320 OR ABS(lnhr)>20 OR ABS(lnwg)>20)>0 THEN error('AR1 GEE requires finite bounded MOM model fields and exact keys') ELSE COUNT(*) END AS observations FROM active"""
sql """SELECT CASE WHEN COUNT(*)<3 OR COUNT(*)>532 OR MIN(t)!=10 OR MAX(t)!=10 THEN error('AR1 GEE requires 3..532 complete ten-year people') ELSE COUNT(*) END AS people FROM (SELECT id,COUNT(*) AS t FROM active GROUP BY id)"""
sql """SELECT CASE WHEN VAR_POP(lnwg) IS NULL OR NOT isfinite(VAR_POP(lnwg)) OR COUNT(*)*VAR_POP(lnwg)<=1e-12 THEN error('AR1 GEE requires identified wage design') ELSE COUNT(*)*VAR_POP(lnwg) END AS wage_centered_ss FROM active"""
sql """SELECT 'AR1 GEE uses the exact keyed annual panel and Pearson rho, with model-working and G/(G-1) independent-person sandwich inference. The retained fit and preceding OLS t/F are not GEE. This marginal association is not causal or a formal serial-correlation test.' AS inference_scope"""
regress lnhr lnwg
predict pols_mean, xb
predict pols_residual, residuals
sql """SELECT * FROM active ORDER BY source_row""" into ar1_gee_source
# Residual-score updates avoid cancellation in the uncentered normal-equation RHS.
sql """
WITH RECURSIVE
source AS MATERIALIZED (
 SELECT *,LEAD(lnhr) OVER (PARTITION BY id ORDER BY year) AS ny,LEAD(lnwg) OVER (PARTITION BY id ORDER BY year) AS nx
 FROM active
), stats AS MATERIALIZED (
 SELECT COUNT(*) AS n,COUNT(DISTINCT id) AS g,
 SUM(lnwg ORDER BY id,year) AS sx,SUM(lnwg*lnwg ORDER BY id,year) AS sxx,
 SUM(lnwg ORDER BY id,year) FILTER (WHERE year>1979 AND year<1988) AS ix,
 SUM(lnwg*lnwg ORDER BY id,year) FILTER (WHERE year>1979 AND year<1988) AS ixx,
 SUM(lnwg+nx ORDER BY id,year) FILTER (WHERE nx IS NOT NULL) AS px,
 SUM(lnwg*nx ORDER BY id,year) FILTER (WHERE nx IS NOT NULL) AS pxx
 FROM source
), start AS (
 SELECT REGR_INTERCEPT(pols_mean,lnwg ORDER BY source_row) AS a,REGR_SLOPE(pols_mean,lnwg ORDER BY source_row) AS b FROM source
), iterate(iteration,a,b,finished) AS (
 SELECT 0,a,b,FALSE FROM start
 UNION ALL
 SELECT iteration+1,an,bn,ABS(an-i.a)<=1e-12*(1+ABS(i.a)) AND ABS(bn-i.b)<=1e-12*(1+ABS(i.b))
 FROM iterate i CROSS JOIN stats s
 CROSS JOIN LATERAL (
 SELECT SUM(POWER(lnhr-i.a-i.b*lnwg,2) ORDER BY id,year) AS rss,
 SUM((lnhr-i.a-i.b*lnwg)*(ny-i.a-i.b*nx) ORDER BY id,year) FILTER (WHERE nx IS NOT NULL) AS lag_cross,
 SUM((lnhr-i.a-i.b*lnwg) ORDER BY id,year) AS e0,
 SUM(lnwg*(lnhr-i.a-i.b*lnwg) ORDER BY id,year) AS e1,
 SUM((lnhr-i.a-i.b*lnwg) ORDER BY id,year) FILTER (WHERE year>1979 AND year<1988) AS ie0,
 SUM(lnwg*(lnhr-i.a-i.b*lnwg) ORDER BY id,year) FILTER (WHERE year>1979 AND year<1988) AS ie1,
 SUM((lnhr-i.a-i.b*lnwg)+(ny-i.a-i.b*nx) ORDER BY id,year) FILTER (WHERE nx IS NOT NULL) AS pe0,
 SUM(lnwg*(ny-i.a-i.b*nx)+nx*(lnhr-i.a-i.b*lnwg) ORDER BY id,year) FILTER (WHERE nx IS NOT NULL) AS pe1 FROM source
 ) residuals
 CROSS JOIN LATERAL (
 SELECT CASE WHEN rss IS NULL OR NOT isfinite(rss) OR rss<=1e-12 OR lag_cross IS NULL OR NOT isfinite(lag_cross)
 THEN error('AR1 GEE requires positive finite residual scale') ELSE lag_cross/(9*s.g*(rss/s.n)) END AS raw_rho
 ) pearson
 CROSS JOIN LATERAL (
 SELECT CASE WHEN NOT isfinite(raw_rho) OR 1-raw_rho*raw_rho<=1e-10 THEN error('AR1 GEE working correlation is singular or unresolved') ELSE raw_rho END AS rho
 ) guarded_rho
 CROSS JOIN LATERAL (
 SELECT s.n+rho*rho*(s.n-2*s.g)-18*s.g*rho AS q00,
 s.sx+rho*rho*s.ix-rho*s.px AS q01,s.sxx+rho*rho*s.ixx-2*rho*s.pxx AS q11,
 e0+rho*rho*ie0-rho*pe0 AS u0,e1+rho*rho*ie1-rho*pe1 AS u1
 ) normal_matrix
 CROSS JOIN LATERAL (
 SELECT CASE WHEN NOT isfinite(q00) OR q00<=1e-12 OR NOT isfinite(q11-q01*q01/q00) OR q11-q01*q01/q00<=1e-12
 THEN error('AR1 GEE requires identified weighted design') ELSE (u1-q01*u0/q00)/(q11-q01*q01/q00) END AS db
 ) coefficient
 CROSS JOIN LATERAL (
 SELECT CASE WHEN NOT isfinite(i.b+db) OR ABS(i.b+db)>100 OR NOT isfinite(i.a+(u0-q01*db)/q00) OR ABS(i.a+(u0-q01*db)/q00)>100
 THEN error('AR1 GEE coefficient step is unrepresentable') ELSE i.b+db END AS bn,i.a+(u0-q01*db)/q00 AS an
 ) guarded_coefficients
 WHERE NOT finished AND iteration<100
), final AS MATERIALIZED (SELECT * FROM iterate ORDER BY iteration DESC LIMIT 1), residuals AS MATERIALIZED (
 SELECT p.*,f.iteration,f.finished,f.a,f.b,p.lnhr-f.a-f.b*p.lnwg AS e,
 LAG(p.lnhr-f.a-f.b*p.lnwg) OVER (PARTITION BY p.id ORDER BY p.year) AS pe,
 LEAD(p.lnhr-f.a-f.b*p.lnwg) OVER (PARTITION BY p.id ORDER BY p.year) AS ne
 FROM source p CROSS JOIN final f
), sums AS MATERIALIZED (
 SELECT COUNT(*) AS n,COUNT(DISTINCT id) AS g,MAX(iteration) AS iteration,BOOL_AND(finished) AS finished,MAX(a) AS a,MAX(b) AS b,
 SUM(e*e ORDER BY id,year) AS rss,SUM(e*ne ORDER BY id,year) FILTER (WHERE ne IS NOT NULL) AS lag_cross FROM residuals
), parameters AS MATERIALIZED (
 SELECT *,rss/n AS phi,lag_cross/(9*g*(rss/n)) AS rho FROM sums
), checked_parameters AS MATERIALIZED (
 SELECT * EXCLUDE(rho),CASE WHEN NOT finished OR iteration>=100 OR NOT isfinite(a) OR NOT isfinite(b)
 OR NOT isfinite(rss) OR rss<=1e-12 OR NOT isfinite(phi) OR phi<=0 OR NOT isfinite(rho) OR 1-rho*rho<=1e-10
 THEN error('AR1 GEE failed convergence or working correlation') ELSE rho END AS rho FROM parameters
), normal_matrix AS MATERIALIZED (
 SELECT p.*,p.n+rho*rho*(p.n-2*p.g)-18*p.g*rho AS q00,
 s.sx+rho*rho*s.ix-rho*s.px AS q01,s.sxx+rho*rho*s.ixx-2*rho*s.pxx AS q11 FROM checked_parameters p CROSS JOIN stats s
), bread AS MATERIALIZED (
 SELECT *,q00*q11-q01*q01 AS det FROM normal_matrix
), unit_raw AS MATERIALIZED (
 SELECT r.id,MAX(bread.n) AS n,MAX(bread.g) AS g,MAX(bread.iteration) AS iteration,MAX(bread.a) AS a,MAX(bread.b) AS b,MAX(rss) AS rss,MAX(phi) AS phi,
 MAX(rho) AS rho,MAX(q00) AS q00,MAX(q01) AS q01,MAX(q11) AS q11,MAX(det) AS det,
 SUM(((CASE WHEN year=1979 OR year=1988 THEN 1 ELSE 1+rho*rho END)*e-rho*(COALESCE(pe,0)+COALESCE(ne,0))) ORDER BY year) AS u0,
 SUM(lnwg*((CASE WHEN year=1979 OR year=1988 THEN 1 ELSE 1+rho*rho END)*e-rho*(COALESCE(pe,0)+COALESCE(ne,0))) ORDER BY year) AS u1
 FROM residuals r CROSS JOIN bread GROUP BY r.id
), influences AS MATERIALIZED (
 SELECT *,CASE WHEN NOT isfinite(det) OR det<=1e-12 OR NOT isfinite(q00) OR q00<=0 THEN error('AR1 GEE bread is singular') ELSE (q11*u0-q01*u1)/det END AS h0,
 (q00*u1-q01*u0)/det AS h1 FROM unit_raw
), aggregate AS (
 SELECT MAX(n) AS n,MAX(g) AS g,MAX(iteration) AS iterations,MAX(a) AS a,MAX(b) AS b,MAX(rss) AS rss,MAX(phi) AS phi,MAX(rho) AS rho,
 MAX(q00) AS q00,MAX(q01) AS q01,MAX(q11) AS q11,MAX(det) AS det,
 SUM(h0*h0 ORDER BY id) AS c00,SUM(h0*h1 ORDER BY id) AS c01,SUM(h1*h1 ORDER BY id) AS c11,
 SUM(h0 ORDER BY id) AS step0,SUM(h1 ORDER BY id) AS step1 FROM influences
), result AS (
 SELECT *,phi*(1-rho*rho)*q11/det AS w00,-phi*(1-rho*rho)*q01/det AS w01,phi*(1-rho*rho)*q00/det AS w11,
 c00*g/(g-1.0) AS v00,c01*g/(g-1.0) AS v01,c11*g/(g-1.0) AS v11 FROM aggregate
)
SELECT 1 AS gee_join_key,n AS observations,g AS people,10 AS years_per_person,iterations,a AS intercept,b AS slope,rss,phi AS scale,rho AS working_correlation,
 w00 AS working_covariance_aa,w01 AS working_covariance_ab,w11 AS working_covariance_bb,g/(g-1.0) AS unit_multiplier,
 v00 AS unit_covariance_aa,v01 AS unit_covariance_ab,v11 AS unit_covariance_bb,
 CASE WHEN NOT isfinite(w00) OR w00<=0 OR NOT isfinite(w01) OR NOT isfinite(w11) OR w11<=0
 OR NOT isfinite(v00) OR v00<=2.842170943040401e-14*w00 OR NOT isfinite(v01) OR NOT isfinite(v11) OR v11<=2.842170943040401e-14*w11
 OR NOT isfinite(step0) OR NOT isfinite(step1) OR ABS(step0)>1e-10*(1+ABS(a)) OR ABS(step1)>1e-10*(1+ABS(b))
 THEN error('AR1 GEE failed covariance or estimating-score certificate') ELSE step0 END AS intercept_step,
 step1 AS slope_step,'pearson_ar1_rss_over_n' AS convention,'g_over_g_minus_1_unit_sandwich' AS unit_covariance,'normal_z_chi2_1' AS inference
FROM result
""" into ar1_gee_fit
export panel_ar1_gee_fit.csv
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
""" into ar1_gee_coefficients
sql """SELECT * FROM active"""
export panel_ar1_gee_coefficients.csv
use ar1_gee_source
generate gee_join_key = 1
join ar1_gee_fit on gee_join_key
sql """SELECT * FROM active""" into ar1_gee_joined
sql """SELECT source_row,lnhr,lnwg,kids,ageh,agesq,disab,id,year,pols_mean,pols_residual,intercept+slope*lnwg AS gee_mean,lnhr-intercept-slope*lnwg AS gee_residual FROM active ORDER BY source_row""" into ar1_gee_predictions
summarize gee_mean gee_residual
export panel_ar1_gee_predictions.csv
use ar1_gee_joined
sql """
WITH r AS (
 SELECT *,lnhr-intercept-slope*lnwg AS e,LAG(lnhr-intercept-slope*lnwg) OVER (PARTITION BY id ORDER BY year) AS pe,
 LEAD(lnhr-intercept-slope*lnwg) OVER (PARTITION BY id ORDER BY year) AS ne FROM active
), weighted AS (
 SELECT *,((CASE WHEN year=1979 OR year=1988 THEN 1 ELSE 1+working_correlation*working_correlation END)*e
 -working_correlation*(COALESCE(pe,0)+COALESCE(ne,0)))/(scale*(1-working_correlation*working_correlation)) AS we FROM r
), scores AS (
 SELECT id,COUNT(*) AS observations,MAX(working_covariance_aa) AS w00,MAX(working_covariance_ab) AS w01,MAX(working_covariance_bb) AS w11,
 SUM(we ORDER BY year) AS u0,SUM(lnwg*we ORDER BY year) AS u1 FROM weighted GROUP BY id
)
SELECT id,observations,u0 AS score_intercept,u1 AS score_slope,w00*u0+w01*u1 AS influence_intercept,w01*u0+w11*u1 AS influence_slope FROM scores ORDER BY id
""" into ar1_gee_unit_scores
export panel_ar1_gee_unit_scores.csv
use ar1_gee_predictions
status
