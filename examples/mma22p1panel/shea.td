# Chapter 22 author wage Shea relevance diagnostic; all auxiliary fits include intercept.
use MOMprecise.csv
isid id year
sql """
WITH panels AS (SELECT id,count(*) AS n,min(year) AS first_year,max(year) AS last_year FROM active GROUP BY id)
SELECT CASE WHEN count(*)=0 OR count(*) FILTER (WHERE n<>10 OR first_year<>1979 OR last_year<>1988)>0
THEN error('panel IV requires all ten years 1979-1988 for every person') ELSE count(*) END AS people FROM panels
"""
sql """
SELECT CASE WHEN count(*) FILTER (WHERE NOT coalesce(isfinite(id) AND id=floor(id) AND id>=1
  AND isfinite(year) AND year=floor(year) AND year BETWEEN 1979 AND 1988
  AND isfinite(lnhr) AND isfinite(lnwg) AND isfinite(kids) AND isfinite(ageh)
  AND isfinite(agesq) AND isfinite(disab),FALSE))>0
THEN error('panel IV requires finite outcomes, covariates and exact integer panel keys')
ELSE count(*) END AS observations FROM active
"""
sql """
WITH lagged AS (
 SELECT id,year,lnhr-lag(lnhr) OVER w AS dlnhr,lnwg-lag(lnwg) OVER w AS dlnwg,
   kids-lag(kids) OVER w AS dkids,ageh-lag(ageh) OVER w AS dageh,
   agesq-lag(agesq) OVER w AS dagesq,disab-lag(disab) OVER w AS ddisab,
   lag(kids) OVER w AS kidsl1,lag(ageh) OVER w AS agehl1,lag(agesq) OVER w AS agesql1,
   lag(disab) OVER w AS disabl1,lag(ageh,2) OVER w AS agehl2,lag(kids,2) OVER w AS kidsl2,
   lag(agesq,2) OVER w AS agesql2,lag(disab,2) OVER w AS disabl2,lag(lnwg,2) OVER w AS lnwgl2
 FROM active WINDOW w AS (PARTITION BY id ORDER BY year)
)
SELECT * FROM lagged WHERE year BETWEEN 1981 AND 1988 ORDER BY id,year
""" into panel_iv_design
isid id year
regress dlnwg dkids dageh dagesq ddisab
predict wage_residual, residuals
regress dlnwg kidsl1 agehl1 agesql1 disabl1 agehl2 kidsl2 agesql2 disabl2 lnwgl2
predict dlnwg_hat, xb
regress dkids kidsl1 agehl1 agesql1 disabl1 agehl2 kidsl2 agesql2 disabl2 lnwgl2
predict dkids_hat, xb
regress dageh kidsl1 agehl1 agesql1 disabl1 agehl2 kidsl2 agesql2 disabl2 lnwgl2
predict dageh_hat, xb
regress dagesq kidsl1 agehl1 agesql1 disabl1 agehl2 kidsl2 agesql2 disabl2 lnwgl2
predict dagesq_hat, xb
regress ddisab kidsl1 agehl1 agesql1 disabl1 agehl2 kidsl2 agesql2 disabl2 lnwgl2
predict ddisab_hat, xb
regress dlnwg_hat dkids_hat dageh_hat dagesq_hat ddisab_hat
predict projected_wage_residual, residuals
sql """
WITH diagnostic AS (
 SELECT count(*) AS observations, corr(wage_residual,projected_wage_residual) AS rho FROM active
)
SELECT observations,
 CASE WHEN rho IS NULL OR NOT isfinite(rho) THEN error('Shea relevance requires finite nonzero residual variation') ELSE rho END AS residual_correlation,
 CASE WHEN rho IS NULL OR NOT isfinite(rho) THEN error('Shea relevance requires finite nonzero residual variation') ELSE rho*rho END AS shea_partial_r_squared,
 'Descriptive relevance only; not a joint weak-IV test or evidence of instrument validity' AS interpretation
FROM diagnostic
"""
export shea_design.csv
