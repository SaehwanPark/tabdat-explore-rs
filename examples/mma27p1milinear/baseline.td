# Source-scoped single-mean-fill comparison; this is NOT multiple imputation.
use missing.csv
isid source_row
sql """SELECT CASE WHEN COUNT(*)<4 OR COUNT(*)>10000 OR COUNT(*) FILTER (WHERE source_row IS NULL OR NOT isfinite(source_row) OR source_row!=floor(source_row) OR source_row<1 OR source_row>10000 OR y IS NULL OR NOT isfinite(y) OR ABS(y)>1000000 OR (x1 IS NOT NULL AND (NOT isfinite(x1) OR ABS(x1)>1000000)) OR (x2 IS NOT NULL AND (NOT isfinite(x2) OR ABS(x2)>1000000)))>0 OR COUNT(x1)=0 OR COUNT(x2)=0 THEN error('missing-data baseline requires 4..10000 rows, finite bounded y and observed predictors') ELSE COUNT(*) END AS admitted_rows FROM active"""
# Preflight both designs before printing inference: reject numerically unresolved rank or residual variance.
sql """
WITH filled AS (
 SELECT y,x1,x2,COALESCE(x1,AVG(x1 ORDER BY source_row) OVER ()) AS a,COALESCE(x2,AVG(x2 ORDER BY source_row) OVER ()) AS b FROM active
), samples AS (
 SELECT 'complete' AS sample,y,x1 AS a,x2 AS b FROM filled WHERE x1 IS NOT NULL AND x2 IS NOT NULL
 UNION ALL SELECT 'filled',y,a,b FROM filled
), moments AS (
 SELECT sample,COUNT(*) AS n,VAR_POP(y) AS yy,VAR_POP(a) AS aa,VAR_POP(b) AS bb,
 COVAR_POP(a,b) AS ab,COVAR_POP(a,y) AS ay,COVAR_POP(b,y) AS byy FROM samples GROUP BY sample
), quality AS (
 SELECT *,aa*bb-ab*ab AS determinant FROM moments
), residual AS (
 SELECT *,yy-(bb*ay*ay-2*ab*ay*byy+aa*byy*byy)/determinant AS residual_variance FROM quality
)
SELECT CASE WHEN COUNT(*)!=2 OR COUNT(*) FILTER (WHERE n<4 OR yy IS NULL OR aa IS NULL OR bb IS NULL OR determinant IS NULL OR residual_variance IS NULL OR NOT isfinite(yy) OR NOT isfinite(aa) OR NOT isfinite(bb) OR NOT isfinite(determinant) OR NOT isfinite(residual_variance) OR yy<=0 OR aa<=0 OR bb<=0 OR determinant<=2.842170943040401e-14*aa*bb OR residual_variance<=2.842170943040401e-14*yy)>0
 THEN error('missing-data baseline needs resolved full-rank designs and positive residual variance in both samples') ELSE COUNT(*) END AS resolved_designs FROM residual
"""
sql """SELECT COUNT(*) AS rows_total,COUNT(x1) AS observed_x1,COUNT(x2) AS observed_x2,COUNT(*)-COUNT(x1) AS missing_x1,COUNT(*)-COUNT(x2) AS missing_x2,COUNT(*) FILTER (WHERE x1 IS NOT NULL AND x2 IS NOT NULL) AS complete_cases,COUNT(*) FILTER (WHERE x1 IS NULL AND x2 IS NULL) AS both_missing,AVG(x1 ORDER BY source_row) AS mean_x1_observed,AVG(x2 ORDER BY source_row) AS mean_x2_observed FROM active"""
sql """SELECT 'Comparison only: complete-case OLS uses jointly observed x; single mean filling uses each predictor observed mean. Classical t/SE after filling treat those means as fixed, omit missing-data uncertainty, and are not multiple imputation, a causal estimate, or a guarantee of unbiasedness.' AS interpretation"""
regress y x1 x2
lincom intercept
lincom x1
lincom x2
predict complete_mean, xb
predict complete_residual, residuals
# Each fill uses all observed rows of its own predictor, not only complete cases.
sql """SELECT source_row,y,x1,x2,CASE WHEN x1 IS NULL THEN 1 ELSE 0 END AS x1_missing,CASE WHEN x2 IS NULL THEN 1 ELSE 0 END AS x2_missing,COALESCE(x1,AVG(x1 ORDER BY source_row) OVER ()) AS x1_filled,COALESCE(x2,AVG(x2 ORDER BY source_row) OVER ()) AS x2_filled,complete_mean,complete_residual FROM active ORDER BY source_row""" into missing_filled
regress y x1_filled x2_filled
lincom intercept
lincom x1_filled
lincom x2_filled
predict filled_mean, xb
predict filled_residual, residuals
sql """SELECT * FROM active ORDER BY source_row""" into missing_comparison_predictions
export missing_comparison_predictions.csv
status
