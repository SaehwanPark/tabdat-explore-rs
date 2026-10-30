# Figure 21.4: consecutive annual first differences; raw lags before selection.
use MOM.csv
isid id year
isid source_row
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(*) FILTER(WHERE source_row IS NULL OR id IS NULL OR year IS NULL OR NOT isfinite(source_row) OR NOT isfinite(id) OR NOT isfinite(year) OR source_row!=floor(source_row) OR source_row<1 OR source_row>9007199254740991 OR id!=floor(id) OR year!=floor(year) OR id<1 OR id>532 OR year<1979 OR year>1988)>0 THEN error('invalid MOM source/person/year keys') ELSE COUNT(*) END AS source_rows,COUNT(DISTINCT id) AS source_people FROM active"""
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(*) FILTER(WHERE lower(column_name) IN ('previous_source_row','previous_year','dlnhr','dlnwg','figure_linear','figure_lowess'))>0 THEN error('first-difference figure output column already exists') ELSE COUNT(*) END AS source_columns FROM information_schema.columns WHERE table_schema='main' AND table_name='active'"""
sql """WITH adjacent AS (SELECT source_row,id,year,lnhr,lnwg,LAG(source_row) OVER w AS previous_source_row,LAG(year) OVER w AS previous_year,lnhr-LAG(lnhr) OVER w AS dlnhr,lnwg-LAG(lnwg) OVER w AS dlnwg FROM active WINDOW w AS (PARTITION BY id ORDER BY year)) SELECT * FROM adjacent WHERE year=previous_year+1 AND isfinite(dlnhr) AND isfinite(dlnwg) ORDER BY id,year""" into fd_figure_data
isid id year
summarize dlnhr dlnwg
regress dlnhr dlnwg, cluster(id)
lincom dlnwg
predict figure_linear, xb
lowess dlnhr dlnwg, gen(figure_lowess) bandwidth=0.8
summarize figure_linear figure_lowess
export fd_figure.csv
label variable dlnwg "Change in log hourly wage"
label variable dlnhr "Change in log annual hours"
label variable figure_lowess "Nonparametric fit (span 0.8)"
label variable figure_linear "First-difference linear fit"
scatter dlnhr dlnwg, lines(figure_lowess figure_linear) title("First Differences Regression") saving("fd_figure.svg") noopen
