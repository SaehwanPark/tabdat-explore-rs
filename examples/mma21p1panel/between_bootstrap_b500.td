# First-program Between pairs bootstrap: 500 equal-person mean-record resamples.
# B200 manual-program workflow remains in between_bootstrap.td, unchanged.
use MOM.csv
isid id year
isid source_row
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(*)>5320 OR COUNT(*) FILTER (WHERE id IS NULL OR year IS NULL OR source_row IS NULL OR lnhr IS NULL OR lnwg IS NULL OR NOT isfinite(id) OR NOT isfinite(year) OR NOT isfinite(source_row) OR NOT isfinite(lnhr) OR NOT isfinite(lnwg) OR id!=floor(id) OR id<1 OR id>532 OR year!=floor(year) OR year<1979 OR year>1988 OR source_row!=floor(source_row) OR source_row<1 OR source_row>5320 OR ABS(lnhr)>20 OR ABS(lnwg)>20)>0 THEN error('B500 Between requires finite bounded MOM model fields and exact keys') ELSE COUNT(*) END AS observations FROM active"""
sql """SELECT CASE WHEN COUNT(*)<3 OR COUNT(*)>532 OR MIN(t)!=10 OR MAX(t)!=10 THEN error('B500 Between requires complete ten-year people') ELSE COUNT(*) END AS people FROM (SELECT id,COUNT(*) AS t FROM active GROUP BY id)"""
sql """SELECT id,COUNT(*) AS model_rows,MIN(year) AS first_year,MAX(year) AS last_year,AVG(lnhr ORDER BY year) AS mean_lnhr,AVG(lnwg ORDER BY year) AS mean_lnwg FROM active GROUP BY id ORDER BY id""" into between_means
isid id
regress mean_lnhr mean_lnwg
bootstrap mean_lnhr mean_lnwg, reps(500) seed(10001)
predict between_fitted, xb
predict between_residual, residuals
summarize between_fitted between_residual
export panel_between_bootstrap_b500_predictions.csv
