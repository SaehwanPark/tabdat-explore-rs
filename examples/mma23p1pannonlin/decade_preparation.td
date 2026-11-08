# Full-decade history first; five lags must precede the analysis restriction.
use patents_decade.csv
isid id year
sql """SELECT CASE WHEN COUNT(*)<20 OR COUNT(*)>3460 OR COUNT(*) FILTER (WHERE typeof(id)!='BIGINT' OR id IS NULL OR id<1 OR id>346 OR typeof(year)!='BIGINT' OR year IS NULL OR year<1970 OR year>1979 OR pat IS NULL OR NOT isfinite(pat) OR pat<0 OR pat>1000000000 OR pat!=floor(pat) OR logr IS NULL OR NOT isfinite(logr) OR abs(logr)>40 OR cusip IS NULL OR NOT isfinite(cusip) OR scisect IS NULL OR NOT isfinite(scisect) OR logk IS NULL OR NOT isfinite(logk) OR sumpat IS NULL OR NOT isfinite(sumpat) OR (ardssic IS NOT NULL AND (TRY_CAST(ardssic AS DOUBLE) IS NULL OR NOT isfinite(TRY_CAST(ardssic AS DOUBLE)))))>0 THEN error('requires canonical finite ten-year patents source and exact keys/counts') ELSE COUNT(*) END AS source_rows,COUNT(DISTINCT id) AS source_firms FROM active"""
sql """SELECT CASE WHEN EXISTS(SELECT id FROM active GROUP BY id HAVING COUNT(*)!=10) THEN error('every firm must have ten unique years1970..1979 before lagging') ELSE COUNT(DISTINCT id) END AS ten_year_firms FROM active"""
sql """SELECT id,year,cusip,CAST(ardssic AS DOUBLE) AS ardssic,scisect,logk,sumpat,logr,pat,LN(CASE WHEN pat=0 THEN 0.5 ELSE pat END) AS lpat,CASE WHEN pat>0 THEN 1 ELSE 0 END AS dpat,EXP(logr) AS randd,LAG(logr,1) OVER (PARTITION BY id ORDER BY year) AS logrl1,LAG(logr,2) OVER (PARTITION BY id ORDER BY year) AS logrl2,LAG(logr,3) OVER (PARTITION BY id ORDER BY year) AS logrl3,LAG(logr,4) OVER (PARTITION BY id ORDER BY year) AS logrl4,LAG(logr,5) OVER (PARTITION BY id ORDER BY year) AS logrl5,CASE WHEN year=1976 THEN 1 ELSE 0 END AS dyear2,CASE WHEN year=1977 THEN 1 ELSE 0 END AS dyear3,CASE WHEN year=1978 THEN 1 ELSE 0 END AS dyear4,CASE WHEN year=1979 THEN 1 ELSE 0 END AS dyear5 FROM active ORDER BY id,year""" into patents_decade_prepared
label variable id "id"
label variable pat "Patents"
label variable logr "Ln(R&D)"
label variable lpat "Ln(Patents)"
label variable dpat "Patent Indicator"
label variable randd "R&D"
label variable logrl1 "Ln(R&D) lagged once"
label variable logrl2 "Ln(R&D) lagged twice"
label variable logrl3 "Ln(R&D) lagged three times"
label variable logrl4 "Ln(R&D) lagged four times"
label variable logrl5 "Ln(R&D) lagged five times"
summarize id year cusip ardssic scisect logk sumpat logr pat lpat dpat randd logrl1 logrl2 logrl3 logrl4 logrl5 dyear2 dyear3 dyear4 dyear5
export patents_decade_prepared.csv
keep if year>=1975
summarize id year cusip ardssic scisect logk sumpat logr pat lpat dpat randd logrl1 logrl2 logrl3 logrl4 logrl5 dyear2 dyear3 dyear4 dyear5
export patents_decade_analysis.csv
