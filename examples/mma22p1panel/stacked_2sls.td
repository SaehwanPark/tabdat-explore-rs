# Table 22.2 year-stacked instruments, first-differenced panel 2SLS; no intercept.
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
sql """
SELECT *,
  CASE WHEN year=1981 THEN agehl1 ELSE 0 END AS z1y1,
  CASE WHEN year=1981 THEN agesql1 ELSE 0 END AS z2y1,
  CASE WHEN year=1981 THEN kidsl1 ELSE 0 END AS z3y1,
  CASE WHEN year=1981 THEN disabl1 ELSE 0 END AS z4y1,
  CASE WHEN year=1981 THEN agehl2 ELSE 0 END AS z5y1,
  CASE WHEN year=1981 THEN agesql2 ELSE 0 END AS z6y1,
  CASE WHEN year=1981 THEN kidsl2 ELSE 0 END AS z7y1,
  CASE WHEN year=1981 THEN disabl2 ELSE 0 END AS z8y1,
  CASE WHEN year=1981 THEN lnwgl2 ELSE 0 END AS z9y1,
  CASE WHEN year=1982 THEN agehl1 ELSE 0 END AS z1y2,
  CASE WHEN year=1982 THEN agesql1 ELSE 0 END AS z2y2,
  CASE WHEN year=1982 THEN kidsl1 ELSE 0 END AS z3y2,
  CASE WHEN year=1982 THEN disabl1 ELSE 0 END AS z4y2,
  CASE WHEN year=1982 THEN agehl2 ELSE 0 END AS z5y2,
  CASE WHEN year=1982 THEN agesql2 ELSE 0 END AS z6y2,
  CASE WHEN year=1982 THEN kidsl2 ELSE 0 END AS z7y2,
  CASE WHEN year=1982 THEN disabl2 ELSE 0 END AS z8y2,
  CASE WHEN year=1982 THEN lnwgl2 ELSE 0 END AS z9y2,
  CASE WHEN year=1983 THEN agehl1 ELSE 0 END AS z1y3,
  CASE WHEN year=1983 THEN agesql1 ELSE 0 END AS z2y3,
  CASE WHEN year=1983 THEN kidsl1 ELSE 0 END AS z3y3,
  CASE WHEN year=1983 THEN disabl1 ELSE 0 END AS z4y3,
  CASE WHEN year=1983 THEN agehl2 ELSE 0 END AS z5y3,
  CASE WHEN year=1983 THEN agesql2 ELSE 0 END AS z6y3,
  CASE WHEN year=1983 THEN kidsl2 ELSE 0 END AS z7y3,
  CASE WHEN year=1983 THEN disabl2 ELSE 0 END AS z8y3,
  CASE WHEN year=1983 THEN lnwgl2 ELSE 0 END AS z9y3,
  CASE WHEN year=1984 THEN agehl1 ELSE 0 END AS z1y4,
  CASE WHEN year=1984 THEN agesql1 ELSE 0 END AS z2y4,
  CASE WHEN year=1984 THEN kidsl1 ELSE 0 END AS z3y4,
  CASE WHEN year=1984 THEN disabl1 ELSE 0 END AS z4y4,
  CASE WHEN year=1984 THEN agehl2 ELSE 0 END AS z5y4,
  CASE WHEN year=1984 THEN agesql2 ELSE 0 END AS z6y4,
  CASE WHEN year=1984 THEN kidsl2 ELSE 0 END AS z7y4,
  CASE WHEN year=1984 THEN disabl2 ELSE 0 END AS z8y4,
  CASE WHEN year=1984 THEN lnwgl2 ELSE 0 END AS z9y4,
  CASE WHEN year=1985 THEN agehl1 ELSE 0 END AS z1y5,
  CASE WHEN year=1985 THEN agesql1 ELSE 0 END AS z2y5,
  CASE WHEN year=1985 THEN kidsl1 ELSE 0 END AS z3y5,
  CASE WHEN year=1985 THEN disabl1 ELSE 0 END AS z4y5,
  CASE WHEN year=1985 THEN agehl2 ELSE 0 END AS z5y5,
  CASE WHEN year=1985 THEN agesql2 ELSE 0 END AS z6y5,
  CASE WHEN year=1985 THEN kidsl2 ELSE 0 END AS z7y5,
  CASE WHEN year=1985 THEN disabl2 ELSE 0 END AS z8y5,
  CASE WHEN year=1985 THEN lnwgl2 ELSE 0 END AS z9y5,
  CASE WHEN year=1986 THEN agehl1 ELSE 0 END AS z1y6,
  CASE WHEN year=1986 THEN agesql1 ELSE 0 END AS z2y6,
  CASE WHEN year=1986 THEN kidsl1 ELSE 0 END AS z3y6,
  CASE WHEN year=1986 THEN disabl1 ELSE 0 END AS z4y6,
  CASE WHEN year=1986 THEN agehl2 ELSE 0 END AS z5y6,
  CASE WHEN year=1986 THEN agesql2 ELSE 0 END AS z6y6,
  CASE WHEN year=1986 THEN kidsl2 ELSE 0 END AS z7y6,
  CASE WHEN year=1986 THEN disabl2 ELSE 0 END AS z8y6,
  CASE WHEN year=1986 THEN lnwgl2 ELSE 0 END AS z9y6,
  CASE WHEN year=1987 THEN agehl1 ELSE 0 END AS z1y7,
  CASE WHEN year=1987 THEN agesql1 ELSE 0 END AS z2y7,
  CASE WHEN year=1987 THEN kidsl1 ELSE 0 END AS z3y7,
  CASE WHEN year=1987 THEN disabl1 ELSE 0 END AS z4y7,
  CASE WHEN year=1987 THEN agehl2 ELSE 0 END AS z5y7,
  CASE WHEN year=1987 THEN agesql2 ELSE 0 END AS z6y7,
  CASE WHEN year=1987 THEN kidsl2 ELSE 0 END AS z7y7,
  CASE WHEN year=1987 THEN disabl2 ELSE 0 END AS z8y7,
  CASE WHEN year=1987 THEN lnwgl2 ELSE 0 END AS z9y7,
  CASE WHEN year=1988 THEN agehl1 ELSE 0 END AS z1y8,
  CASE WHEN year=1988 THEN agesql1 ELSE 0 END AS z2y8,
  CASE WHEN year=1988 THEN kidsl1 ELSE 0 END AS z3y8,
  CASE WHEN year=1988 THEN disabl1 ELSE 0 END AS z4y8,
  CASE WHEN year=1988 THEN agehl2 ELSE 0 END AS z5y8,
  CASE WHEN year=1988 THEN agesql2 ELSE 0 END AS z6y8,
  CASE WHEN year=1988 THEN kidsl2 ELSE 0 END AS z7y8,
  CASE WHEN year=1988 THEN disabl2 ELSE 0 END AS z8y8,
  CASE WHEN year=1988 THEN lnwgl2 ELSE 0 END AS z9y8
FROM active ORDER BY id,year
""" into stacked_iv_design
ivregress 2sls dlnhr, endog(dlnwg dkids dageh dagesq ddisab) iv(z1y1 z2y1 z3y1 z4y1 z5y1 z6y1 z7y1 z8y1 z9y1 z1y2 z2y2 z3y2 z4y2 z5y2 z6y2 z7y2 z8y2 z9y2 z1y3 z2y3 z3y3 z4y3 z5y3 z6y3 z7y3 z8y3 z9y3 z1y4 z2y4 z3y4 z4y4 z5y4 z6y4 z7y4 z8y4 z9y4 z1y5 z2y5 z3y5 z4y5 z5y5 z6y5 z7y5 z8y5 z9y5 z1y6 z2y6 z3y6 z4y6 z5y6 z6y6 z7y6 z8y6 z9y6 z1y7 z2y7 z3y7 z4y7 z5y7 z6y7 z7y7 z8y7 z9y7 z1y8 z2y8 z3y8 z4y8 z5y8 z6y8 z7y8 z8y8 z9y8) noconstant
ivregress 2sls dlnhr, endog(dlnwg dkids dageh dagesq ddisab) iv(z1y1 z2y1 z3y1 z4y1 z5y1 z6y1 z7y1 z8y1 z9y1 z1y2 z2y2 z3y2 z4y2 z5y2 z6y2 z7y2 z8y2 z9y2 z1y3 z2y3 z3y3 z4y3 z5y3 z6y3 z7y3 z8y3 z9y3 z1y4 z2y4 z3y4 z4y4 z5y4 z6y4 z7y4 z8y4 z9y4 z1y5 z2y5 z3y5 z4y5 z5y5 z6y5 z7y5 z8y5 z9y5 z1y6 z2y6 z3y6 z4y6 z5y6 z6y6 z7y6 z8y6 z9y6 z1y7 z2y7 z3y7 z4y7 z5y7 z6y7 z7y7 z8y7 z9y7 z1y8 z2y8 z3y8 z4y8 z5y8 z6y8 z7y8 z8y8 z9y8) noconstant cluster(id)
predict panel_iv_fitted, xb
predict panel_iv_residual, residuals
export stacked_iv_design.csv
