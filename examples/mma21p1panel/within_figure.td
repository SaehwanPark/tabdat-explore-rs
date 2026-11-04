# Figure 21.3: within-person deviations plus observation-weighted overall means.
use MOM.csv
isid id year
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(*) FILTER(WHERE id IS NULL OR year IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR id!=floor(id) OR year!=floor(year) OR id<1 OR id>532 OR year<1979 OR year>1988)>0 THEN error('invalid MOM person/year keys') ELSE COUNT(*) END AS source_rows,COUNT(DISTINCT id) AS source_people FROM active"""
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(*) FILTER(WHERE lower(column_name) IN ('unit_lnhr','unit_lnwg','overall_lnhr','overall_lnwg','within_lnhr','within_lnwg','figure_linear','figure_lowess'))>0 THEN error('within figure output column already exists') ELSE COUNT(*) END AS source_columns FROM information_schema.columns WHERE table_schema='main' AND table_name='active'"""
sql """SELECT CASE WHEN COUNT(*) FILTER(WHERE isfinite(lnhr) AND isfinite(lnwg) AND (ABS(lnhr)>1000000 OR ABS(lnwg)>1000000 OR lnhr!=round(lnhr,2) OR lnwg!=round(lnwg,2)))>0 THEN error('within figure requires two-decimal finite model values') ELSE COUNT(*) END AS source_rows FROM active"""
sql """SELECT * FROM active WHERE isfinite(lnhr) AND isfinite(lnwg) ORDER BY id,year""" into within_eligible
# Report the actual native FE cluster inference before changing active data.
xtreg lnhr lnwg, fe id(id) time(year) cluster(id)
lincom lnwg
# Exact source-scale sums; float64 division, then (raw - unit mean) + grand mean.
sql """WITH person AS (SELECT id,SUM(CAST(lnhr AS DECIMAL(18,2)))/COUNT(*) AS unit_lnhr,SUM(CAST(lnwg AS DECIMAL(18,2)))/COUNT(*) AS unit_lnwg FROM active GROUP BY id),overall AS (SELECT SUM(CAST(lnhr AS DECIMAL(18,2)))/COUNT(*) AS overall_lnhr,SUM(CAST(lnwg AS DECIMAL(18,2)))/COUNT(*) AS overall_lnwg FROM active) SELECT s.*,p.unit_lnhr,p.unit_lnwg,o.overall_lnhr,o.overall_lnwg,(s.lnhr-p.unit_lnhr)+o.overall_lnhr AS within_lnhr,(s.lnwg-p.unit_lnwg)+o.overall_lnwg AS within_lnwg FROM active s JOIN person p USING(id) CROSS JOIN overall o ORDER BY s.id,s.year""" into within_figure_data
summarize within_lnhr within_lnwg
# This auxiliary OLS has its own two-parameter covariance correction, not FE's.
regress within_lnhr within_lnwg, cluster(id)
lincom within_lnwg
predict figure_linear, xb
lowess within_lnhr within_lnwg, gen(figure_lowess) bandwidth=0.8
summarize figure_linear figure_lowess
export within_figure.csv
label variable within_lnwg "Within log wage (overall mean restored)"
label variable within_lnhr "Within log hours (overall mean restored)"
label variable figure_lowess "Nonparametric fit (span 0.8)"
label variable figure_linear "Within linear fit"
scatter within_lnhr within_lnwg, lines(figure_lowess figure_linear) title("Within (Fixed Effects) Regression") saving("within_figure.svg") noopen
