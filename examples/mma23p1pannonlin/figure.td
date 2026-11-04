# Figure 23.1 source-data adaptation: corrected author-program axes.
use patents.csv
isid id year
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(DISTINCT id)<2 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR pat IS NULL OR logr IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR NOT isfinite(pat) OR NOT isfinite(logr) OR id!=floor(id) OR id<1 OR id>346 OR year!=floor(year) OR year<1975 OR year>1979 OR pat<0 OR pat!=floor(pat) OR pat>1000 OR ABS(logr)>20)>0 THEN error('invalid log-patents panel data') ELSE COUNT(*) END AS observations,COUNT(DISTINCT id) AS firms FROM active"""
sql """SELECT CASE WHEN COUNT(*)=0 OR MIN(n)!=5 OR MAX(n)!=5 THEN error('each firm requires all five years 1975–79') ELSE COUNT(*) END AS firms FROM (SELECT id,COUNT(*) AS n FROM active GROUP BY id)"""
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(*) FILTER (WHERE lower(column_name) IN ('lpat','figure_linear','figure_lowess'))>0 THEN error('figure output column already exists') ELSE COUNT(*) END AS source_columns FROM information_schema.columns WHERE table_schema='main' AND table_name='active'"""
generate lpat = 0
sql """SELECT * EXCLUDE(lpat),LN(CASE WHEN pat=0 THEN 0.5 ELSE pat END) AS lpat FROM active""" into figure_data
# Fix within-tie order before the ordered-row smoother; do not rely on file order.
sort id year
summarize pat logr lpat
regress lpat logr, cluster(id)
lincom logr
predict figure_linear, xb
lowess lpat logr, gen(figure_lowess) bandwidth=0.8
summarize figure_linear figure_lowess
export figure_data.csv
label variable logr "Log R&D spending"
label variable lpat "Log patents (zero mapped to ln(0.5))"
label variable figure_lowess "Nonparametric fit (span 0.8)"
label variable figure_linear "Pooled linear fit"
scatter lpat logr, lines(figure_lowess figure_linear) title("Pooled (Overall) Regression") saving("figure.svg") noopen
