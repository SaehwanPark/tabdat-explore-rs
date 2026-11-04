# Figure 21.1 source-data adaptation: joint-finite pooled person-years.
use MOM.csv
isid id year
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(*) FILTER(WHERE id IS NULL OR year IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR id!=floor(id) OR year!=floor(year) OR id<1 OR id>532 OR year<1979 OR year>1988)>0 THEN error('invalid MOM person/year keys') ELSE COUNT(*) END AS source_rows,COUNT(DISTINCT id) AS source_people FROM active"""
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(*) FILTER(WHERE lower(column_name) IN ('figure_linear','figure_lowess'))>0 THEN error('pooled figure output column already exists') ELSE COUNT(*) END AS source_columns FROM information_schema.columns WHERE table_schema='main' AND table_name='active'"""
# Fix candidate order before the ordered-row smoother; preserve source columns.
sql """SELECT * FROM active WHERE isfinite(lnhr) AND isfinite(lnwg) ORDER BY id,year""" into pooled_figure_data
summarize lnhr lnwg
regress lnhr lnwg, cluster(id)
lincom lnwg
predict figure_linear, xb
lowess lnhr lnwg, gen(figure_lowess) bandwidth=0.8
summarize figure_linear figure_lowess
export pooled_figure.csv
label variable lnwg "Log hourly wage"
label variable lnhr "Log annual hours"
label variable figure_lowess "Nonparametric fit (span 0.8)"
label variable figure_linear "Pooled linear fit"
scatter lnhr lnwg, lines(figure_lowess figure_linear) title("Pooled (Overall) Regression") saving("pooled_figure.svg") noopen
