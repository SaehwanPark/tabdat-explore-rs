# Shared gamma firm effects, bounded 2-coefficient/one-dispersion likelihood.
use patents.csv
isid id year
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(DISTINCT id)<2 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR pat IS NULL OR logr IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR NOT isfinite(pat) OR NOT isfinite(logr) OR id!=floor(id) OR id<1 OR id>346 OR year!=floor(year) OR year<1975 OR year>1979 OR pat<0 OR pat!=floor(pat) OR pat>1000 OR ABS(logr)>20)>0 THEN error('invalid gamma RE panel data') ELSE COUNT(*) END AS observations,COUNT(DISTINCT id) AS firms FROM active"""
sql """SELECT CASE WHEN COUNT(*)=0 OR MIN(n)!=5 OR MAX(n)!=5 THEN error('each firm requires all five years 1975–79') ELSE COUNT(*) END AS firms FROM (SELECT id,COUNT(*) AS n FROM active GROUP BY id)"""
poisson pat logr
predict pooled_xb, xb
sql """
WITH RECURSIVE
source AS (SELECT * FROM active),
firm AS (
 SELECT id,SUM(pat ORDER BY year) AS y_total,SUM(pat*logr ORDER BY year) AS yx,
   SUM(ABS(pat*logr) ORDER BY year) AS yx_scale,SUM(lgamma(pat+1) ORDER BY year) AS log_factorial
 FROM source GROUP BY id
), start_regression AS (
 SELECT REGR_INTERCEPT(pooled_xb,logr) AS a,REGR_SLOPE(pooled_xb,logr) AS b FROM source
), start AS (
 SELECT CASE WHEN a IS NULL OR b IS NULL OR NOT isfinite(a) OR NOT isfinite(b)
   OR COUNT(*) FILTER (WHERE pooled_xb IS NULL OR NOT isfinite(pooled_xb))>0
   OR MAX(ABS(pooled_xb-(a+b*logr)))>1e-12*(1+MAX(ABS(pooled_xb)))
   THEN error('gamma RE pooled start cannot be reconstructed') ELSE a END AS a,b,COUNT(DISTINCT id) AS g,COUNT(*) AS n
 FROM source CROSS JOIN start_regression GROUP BY a,b
), search(iter,a,b,t,g,n,old_a,old_b,old_t,u0,u1,u2,s0,s1,s2,i00,i01,i02,i11,i12,i22,ll,last_step) AS (
 SELECT 0::BIGINT,a,b,0.0::DOUBLE,g,n,NULL::DOUBLE,NULL::DOUBLE,NULL::DOUBLE,
   NULL::DOUBLE,NULL::DOUBLE,NULL::DOUBLE,NULL::DOUBLE,NULL::DOUBLE,NULL::DOUBLE,
   NULL::DOUBLE,NULL::DOUBLE,NULL::DOUBLE,NULL::DOUBLE,NULL::DOUBLE,NULL::DOUBLE,
   NULL::DOUBLE,1e100::DOUBLE FROM start
 UNION ALL
 SELECT iter+1,a+da,b+db,t+dt,g,n,a,b,t,step.u0,step.u1,step.u2,step.s0,step.s1,step.s2,step.i00,step.i01,step.i02,step.i11,step.i12,step.i22,step.ll,
   GREATEST(ABS(da)/(1+ABS(a)),ABS(db)/(1+ABS(b)),ABS(dt)/(1+ABS(t)))
 FROM search CROSS JOIN LATERAL (
   WITH observation AS (
     SELECT id,year,pat,logr,
       CASE WHEN a IS NULL OR b IS NULL OR t IS NULL OR NOT isfinite(a) OR NOT isfinite(b) OR NOT isfinite(t)
         OR ABS(a)>20 OR ABS(b)>20 OR ABS(t)>=8 OR NOT isfinite(a+b*logr) OR ABS(a+b*logr)>40
         THEN error('gamma RE parameters or predictor outside numerical bounds') ELSE EXP(a+b*logr) END AS mu
     FROM source
   ), panel AS (
     SELECT id,COUNT(*) AS panel_n,SUM(pat ORDER BY year) AS y_total,SUM(pat*logr ORDER BY year) AS yx,
       SUM(ABS(pat*logr) ORDER BY year) AS yx_scale,SUM(lgamma(pat+1) ORDER BY year) AS log_factorial,
       SUM(mu ORDER BY year) AS m,SUM(mu*logr ORDER BY year) AS h,
       SUM(mu*logr*logr ORDER BY year) AS j,EXP(-t) AS r FROM observation GROUP BY id,t
   ), gamma_terms AS (
     -- Integer recurrences avoid subtracting nearly equal digamma/trigamma values.
     SELECT *,COALESCE(LIST_SUM(LIST_TRANSFORM(RANGE(0,y_total::BIGINT),k -> 1/(r+k))),0.0) AS d,
       COALESCE(LIST_SUM(LIST_TRANSFORM(RANGE(0,y_total::BIGINT),k -> 1/POWER(r+k,2))),0.0) AS d2 FROM panel
   ), system AS (
     SELECT COUNT(*) AS group_count,SUM(panel_n) AS row_count,MIN(panel_n) AS min_panel_n,MAX(panel_n) AS max_panel_n,
       SUM(y_total-(r+y_total)*m/(r+m) ORDER BY id) AS u0,
       SUM(yx-(r+y_total)*h/(r+m) ORDER BY id) AS u1,
       -MAX(r)*(SUM(d ORDER BY id)+COUNT(*)*(LN(MAX(r))+1)-SUM(LN(r+m)+(r+y_total)/(r+m) ORDER BY id)) AS u2,
       SUM(ABS(y_total)+ABS((r+y_total)*m/(r+m)) ORDER BY id) AS s0,
       SUM(yx_scale+ABS((r+y_total)*h/(r+m)) ORDER BY id) AS s1,
       MAX(r)*(SUM(ABS(d) ORDER BY id)+COUNT(*)*(ABS(LN(MAX(r)))+1)+SUM(ABS(LN(r+m))+ABS((r+y_total)/(r+m)) ORDER BY id)) AS s2,
       SUM((r+y_total)/(r+m)*m*r/(r+m) ORDER BY id) AS i00,
       SUM((r+y_total)/(r+m)*h*r/(r+m) ORDER BY id) AS i01,
       -SUM(r*(m-y_total)*m/POWER(r+m,2) ORDER BY id) AS i02,
       SUM((r+y_total)/(r+m)*(j-h*h/(r+m)) ORDER BY id) AS i11,
       SUM((r+y_total)/(r+m)*(ABS(j)+h*h/(r+m)) ORDER BY id) AS i11_scale,
       -SUM(r*(m-y_total)*h/POWER(r+m,2) ORDER BY id) AS i12,
       -MAX(r)*(SUM(d ORDER BY id)+COUNT(*)*(LN(MAX(r))+1)-SUM(LN(r+m)+(r+y_total)/(r+m) ORDER BY id))
         -POWER(MAX(r),2)*(-SUM(d2 ORDER BY id)+COUNT(*)/MAX(r)-SUM(1/(r+m) ORDER BY id)+SUM((y_total-m)/POWER(r+m,2) ORDER BY id)) AS i22,
       MAX(r)*(SUM(ABS(d) ORDER BY id)+COUNT(*)*(ABS(LN(MAX(r)))+1)+SUM(ABS(LN(r+m))+ABS((r+y_total)/(r+m)) ORDER BY id))
         +POWER(MAX(r),2)*(SUM(ABS(d2) ORDER BY id)+COUNT(*)/MAX(r)+SUM(1/(r+m) ORDER BY id)+SUM((ABS(y_total)+ABS(m))/POWER(r+m,2) ORDER BY id)) AS i22_scale,
       SUM(lgamma(r+y_total)-lgamma(r)+r*LN(r)-(r+y_total)*LN(r+m)+a*y_total+b*yx-log_factorial ORDER BY id) AS ll
     FROM gamma_terms
   ), first_pivot AS (
     SELECT *,CASE WHEN group_count!=search.g OR row_count!=search.n OR min_panel_n!=5 OR max_panel_n!=5
       OR NOT isfinite(u0) OR NOT isfinite(u1) OR NOT isfinite(u2) OR NOT isfinite(s0) OR NOT isfinite(s1) OR NOT isfinite(s2)
       OR NOT isfinite(i00) OR i00<=0 OR NOT isfinite(i01) OR NOT isfinite(i02) OR NOT isfinite(i11) OR NOT isfinite(i11_scale)
       OR NOT isfinite(i12) OR NOT isfinite(i22) OR NOT isfinite(i22_scale)
       OR NOT isfinite(ll) OR (iter>0 AND ll<search.ll-1e-10*(1+ABS(search.ll)))
       THEN error('gamma RE nonfinite or nonascending Newton iterate') ELSE i00 END AS p00 FROM system
   ), second_pivot AS (
     SELECT *,CASE WHEN NOT isfinite(i11-i01*i01/p00)
       OR i11-i01*i01/p00<=2.842170943040401e-14*(1+i11_scale+ABS(i01*i01/p00))
       THEN error('gamma RE working information is singular') ELSE i11-i01*i01/p00 END AS p11 FROM first_pivot
   ), third_pivot AS (
     SELECT *, (i12-i01*i02/p00)/p11 AS l21,
       CASE WHEN NOT isfinite(i22-i02*i02/p00-POWER(i12-i01*i02/p00,2)/p11)
       OR i22-i02*i02/p00-POWER(i12-i01*i02/p00,2)/p11<=2.842170943040401e-14*(1+i22_scale+ABS(i02*i02/p00)+ABS(POWER(i12-i01*i02/p00,2)/p11))
       THEN error('gamma RE working information is singular') ELSE i22-i02*i02/p00-POWER(i12-i01*i02/p00,2)/p11 END AS p22
     FROM second_pivot
   ), steps AS (
     SELECT *, (u2-(i02/p00)*u0-l21*(u1-(i01/p00)*u0))/p22 AS dt FROM third_pivot
   ), steps2 AS (
     SELECT *, (u1-(i01/p00)*u0)/p11-l21*dt AS db FROM steps
   )
   SELECT *, (u0-i01*db-i02*dt)/p00 AS da FROM steps2
 ) step
 WHERE iter<100 AND (iter=0 OR last_step>1e-12)
), last_iteration AS MATERIALIZED (
 SELECT * FROM search ORDER BY iter DESC LIMIT 1
), final_checked AS (
 SELECT *,CASE WHEN iter=0 OR last_step IS NULL OR NOT isfinite(last_step) OR last_step>1e-12
   OR ABS(u0)>1e-10*(1+s0) OR ABS(u1)>1e-10*(1+s1) OR ABS(u2)>1e-10*(1+s2)
   THEN error('gamma RE failed convergence or final score check') ELSE old_a END AS intercept
 FROM last_iteration
), inverse_base AS (
 SELECT *,i11-i01*i01/i00 AS p11,(i12-i01*i02/i00)/(i11-i01*i01/i00) AS l21,
   i22-i02*i02/i00-POWER(i12-i01*i02/i00,2)/(i11-i01*i01/i00) AS p22 FROM final_checked
), inverse_info AS (
 SELECT *,1/p22 AS c22,-l21/p22 AS c12,
   1/p11+l21*l21/p22 AS c11,
   -(i02/i00-l21*i01/i00)/p22 AS c02,
   -(i01/i00)/p11+l21*(i02/i00-l21*i01/i00)/p22 AS c01,
   1/i00+POWER(i01/i00,2)/p11+POWER(i02/i00-l21*i01/i00,2)/p22 AS c00
 FROM inverse_base
), guarded_fit AS MATERIALIZED (
 SELECT *,CASE WHEN NOT isfinite(c00) OR NOT isfinite(c01) OR NOT isfinite(c02) OR NOT isfinite(c11) OR NOT isfinite(c12) OR NOT isfinite(c22)
   OR c00<=0 OR c11<=0 OR c22<=0
   OR 2.842170943040401e-14*(ABS(c00)*s0+ABS(c01)*s1+ABS(c02)*s2)>=1e-9+1e-10*ABS(intercept)
   OR 2.842170943040401e-14*(ABS(c01)*s0+ABS(c11)*s1+ABS(c12)*s2)>=1e-9+1e-10*ABS(old_b)
   OR 2.842170943040401e-14*(ABS(c02)*s0+ABS(c12)*s1+ABS(c22)*s2)>=1e-9+1e-10*ABS(old_t)
   THEN error('gamma RE covariance or coefficient is numerically unreliable') ELSE old_b END AS slope
 FROM inverse_info
), native AS (
 SELECT SUM(pat*pooled_xb-EXP(pooled_xb)-lgamma(pat+1) ORDER BY id,year) AS poisson_log_likelihood FROM source
)
SELECT s.*,
   EXP(f.intercept+f.slope*s.logr) AS marginal_mu,s.pat-EXP(f.intercept+f.slope*s.logr) AS marginal_residual,
   f.intercept AS estimate_intercept,f.slope AS estimate_slope,f.old_t AS estimate_lnalpha,EXP(f.old_t) AS alpha,
   f.c00 AS covariance_aa,f.c01 AS covariance_ab,f.c02 AS covariance_at,f.c11 AS covariance_bb,f.c12 AS covariance_bt,f.c22 AS covariance_tt,
   f.ll AS log_likelihood,n.poisson_log_likelihood,
   CASE WHEN NOT isfinite(n.poisson_log_likelihood) OR NOT isfinite(2*(f.ll-n.poisson_log_likelihood))
     OR f.ll-n.poisson_log_likelihood<=2.842170943040401e-14*(1+ABS(f.ll)+ABS(n.poisson_log_likelihood))
     THEN error('gamma RE likelihood is indistinguishable from its Poisson boundary')
     ELSE 2*(f.ll-n.poisson_log_likelihood) END AS lr_alpha_zero,
   f.iter AS iterations,f.last_step,(SELECT COUNT(*) FROM source) AS candidate_rows,(SELECT COUNT(*) FROM firm) AS firms,
   (SELECT COUNT(*) FROM firm WHERE y_total=0) AS zero_total_firms
FROM source s CROSS JOIN guarded_fit f CROSS JOIN native n
ORDER BY s.id,s.year
""" into gamma_re_poisson_predictions
sql """WITH RECURSIVE
z_values AS (
 SELECT 'intercept' AS name,ABS(estimate_intercept)/SQRT(covariance_aa) AS z FROM (SELECT * FROM active LIMIT 1)
 UNION ALL SELECT 'slope',ABS(estimate_slope)/SQRT(covariance_bb) FROM (SELECT * FROM active LIMIT 1)
 UNION ALL SELECT 'lnalpha',ABS(estimate_lnalpha)/SQRT(covariance_tt) FROM (SELECT * FROM active LIMIT 1)
 UNION ALL SELECT 'alpha_boundary',SQRT(lr_alpha_zero) FROM (SELECT * FROM active LIMIT 1)
), small_tail(name,z,iter,term,partial) AS (
 SELECT name,z,0,z/SQRT(2),z/SQRT(2) FROM z_values WHERE z<=3
 UNION ALL SELECT name,z,iter+1,term*(-(z*z/2))*(2*iter+1)/((iter+1)*(2*iter+3)),
   partial+term*(-(z*z/2))*(2*iter+1)/((iter+1)*(2*iter+3)) FROM small_tail WHERE iter<80
), large_tail(name,z,iter,denominator) AS (
 SELECT name,z,100,z FROM z_values WHERE z>3
 UNION ALL SELECT name,z,iter-1,z+iter/denominator FROM large_tail WHERE iter>0
), pvalues AS (
 SELECT name,1-2/SQRT(PI())*partial AS p FROM small_tail WHERE iter=80
 UNION ALL SELECT name,2*EXP(-z*z/2)/SQRT(2*PI())/denominator AS p FROM large_tail WHERE iter=0
)
SELECT MAX(candidate_rows) AS candidate_rows,MAX(firms) AS firms,MAX(zero_total_firms) AS zero_total_firms,
 MAX(estimate_intercept) AS estimate_intercept,MAX(estimate_slope) AS estimate_slope,MAX(estimate_lnalpha) AS estimate_lnalpha,MAX(alpha) AS alpha,
 SQRT(MAX(covariance_aa)) AS se_intercept,SQRT(MAX(covariance_bb)) AS se_slope,SQRT(MAX(covariance_tt)) AS se_lnalpha,
 MAX(alpha)*SQRT(MAX(covariance_tt)) AS se_alpha,
 EXP(MAX(estimate_lnalpha)-1.959963984540054*SQRT(MAX(covariance_tt))) AS lower_alpha,
 EXP(MAX(estimate_lnalpha)+1.959963984540054*SQRT(MAX(covariance_tt))) AS upper_alpha,
 MAX(estimate_intercept)/SQRT(MAX(covariance_aa)) AS z_intercept,MAX(estimate_slope)/SQRT(MAX(covariance_bb)) AS z_slope,MAX(estimate_lnalpha)/SQRT(MAX(covariance_tt)) AS z_lnalpha,
 (SELECT p FROM pvalues WHERE name='intercept') AS p_intercept,(SELECT p FROM pvalues WHERE name='slope') AS p_slope,(SELECT p FROM pvalues WHERE name='lnalpha') AS p_lnalpha,
 MAX(estimate_intercept)-1.959963984540054*SQRT(MAX(covariance_aa)) AS lower_intercept,MAX(estimate_intercept)+1.959963984540054*SQRT(MAX(covariance_aa)) AS upper_intercept,
 MAX(estimate_slope)-1.959963984540054*SQRT(MAX(covariance_bb)) AS lower_slope,MAX(estimate_slope)+1.959963984540054*SQRT(MAX(covariance_bb)) AS upper_slope,
 MAX(estimate_lnalpha)-1.959963984540054*SQRT(MAX(covariance_tt)) AS lower_lnalpha,MAX(estimate_lnalpha)+1.959963984540054*SQRT(MAX(covariance_tt)) AS upper_lnalpha,
 MAX(covariance_aa) AS covariance_aa,MAX(covariance_ab) AS covariance_ab,MAX(covariance_at) AS covariance_at,MAX(covariance_bb) AS covariance_bb,MAX(covariance_bt) AS covariance_bt,MAX(covariance_tt) AS covariance_tt,
 MAX(log_likelihood) AS log_likelihood,MAX(poisson_log_likelihood) AS poisson_log_likelihood,MAX(lr_alpha_zero) AS lr_alpha_zero,
 (SELECT p/2 FROM pvalues WHERE name='alpha_boundary') AS p_lr_alpha_zero,
 AVG(marginal_mu ORDER BY id,year) AS mean_prediction,AVG(marginal_residual ORDER BY id,year) AS mean_residual,
 'normal_asymptotic' AS inference,'chi_bar_square_0_1_equal_mixture' AS lr_reference FROM active"""
export gamma_re_poisson_predictions.csv
