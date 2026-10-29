# Figure 21.2 adaptation: equally weighted joint-finite person means.
use MOM.csv
isid id year
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(*) FILTER(WHERE id IS NULL OR year IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR id!=floor(id) OR year!=floor(year) OR id<1 OR id>532 OR year<1979 OR year>1988)>0 THEN error('invalid MOM person/year keys') ELSE COUNT(*) END AS source_rows,COUNT(DISTINCT id) AS source_people FROM active"""
sql """SELECT id,COUNT(*) AS model_rows,MIN(year) AS first_year,MAX(year) AS last_year,AVG(lnhr ORDER BY year) AS mean_lnhr,AVG(lnwg ORDER BY year) AS mean_lnwg FROM active WHERE isfinite(lnhr) AND isfinite(lnwg) GROUP BY id ORDER BY id""" into between_figure_data
isid id
summarize mean_lnhr mean_lnwg
regress mean_lnhr mean_lnwg
lincom mean_lnwg
predict figure_linear, xb
lowess mean_lnhr mean_lnwg, gen(figure_lowess) bandwidth=0.8
summarize figure_linear figure_lowess
export between_figure.csv
label variable mean_lnwg "Mean log hourly wage"
label variable mean_lnhr "Mean log annual hours"
label variable figure_lowess "Nonparametric fit (span 0.8)"
label variable figure_linear "Between linear fit"
scatter mean_lnhr mean_lnwg, lines(figure_lowess figure_linear) title("Between Regression") saving("between_figure.svg") noopen
