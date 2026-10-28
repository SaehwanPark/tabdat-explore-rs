# Table 22.2 first-differenced pooled OLS; no intercept.
use MOMprecise.csv
isid id year
sql """
WITH panels AS (SELECT id,count(*) AS n,min(year) AS first_year,max(year) AS last_year FROM active GROUP BY id)
SELECT CASE WHEN count(*)=0 OR count(*) FILTER (WHERE n<>10 OR first_year<>1979 OR last_year<>1988)>0
THEN error('panel analysis requires all ten years 1979-1988 for every person') ELSE count(*) END AS people FROM panels
"""
sql """
SELECT CASE WHEN count(*) FILTER (WHERE NOT coalesce(isfinite(id) AND id=floor(id) AND id>=1
  AND isfinite(year) AND year=floor(year) AND year BETWEEN 1979 AND 1988
  AND isfinite(lnhr) AND isfinite(lnwg) AND isfinite(kids) AND isfinite(ageh)
  AND isfinite(agesq) AND isfinite(disab),FALSE))>0
THEN error('panel analysis requires finite outcomes, covariates and exact integer panel keys')
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
""" into fd_ols_design
isid id year
regress dlnhr dlnwg dkids dageh dagesq ddisab, noconstant
regress dlnhr dlnwg dkids dageh dagesq ddisab, noconstant robust
regress dlnhr dlnwg dkids dageh dagesq ddisab, noconstant cluster(id)
predict fd_ols_fitted, xb
predict fd_ols_residual, residuals
export fd_ols_design.csv
