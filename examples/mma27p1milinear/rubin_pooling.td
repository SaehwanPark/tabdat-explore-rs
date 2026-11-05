# Marginal pooling of supplied summaries, not imputation or per-dataset fitting.
use estimates.csv
isid imputation parameter
sql """SELECT CASE WHEN COUNT(*)<6 OR COUNT(*)>300 OR COUNT(*) FILTER (WHERE imputation IS NULL OR NOT isfinite(imputation) OR imputation!=floor(imputation) OR imputation<1 OR imputation>100 OR parameter IS NULL OR parameter NOT IN ('intercept','x1missing','x2missing') OR estimate IS NULL OR NOT isfinite(estimate) OR ABS(estimate)>1000000 OR standard_error IS NULL OR NOT isfinite(standard_error) OR standard_error<1e-12 OR standard_error>1000000)>0 THEN error('Rubin pooling requires finite bounded estimate/SE and exact parameter/imputation keys') ELSE COUNT(*) END AS admitted_rows FROM active"""
sql """SELECT CASE WHEN COUNT(*)<2 OR COUNT(*)>100 OR MIN(imputation)!=1 OR MAX(imputation)!=COUNT(*) OR MIN(parameters)!=3 OR MAX(parameters)!=3 THEN error('Rubin pooling requires the same contiguous 2..100 imputations for all three parameters') ELSE COUNT(*) END AS imputations FROM (SELECT imputation,COUNT(*) AS parameters FROM active GROUP BY imputation)"""
sql """SELECT 'Marginal Rubin pooling assumes supplied estimates come from proper imputations and compatible complete-data analyses; no imputation, regression fit, joint covariance, finite-complete-data DF or causal claim is supplied.' AS inference_scope"""
sql """
WITH summary AS (
 SELECT parameter,COUNT(*) AS m,AVG(estimate ORDER BY imputation) AS qbar,AVG(standard_error*standard_error ORDER BY imputation) AS within_variance,
 VAR_SAMP(estimate ORDER BY imputation) AS between_variance,MIN(estimate) AS minimum,MAX(estimate) AS maximum
 FROM active GROUP BY parameter
), totals AS (
 SELECT *,within_variance+(1+1.0/m)*between_variance AS total_variance,
 (1+1.0/m)*between_variance/within_variance AS relative_increase FROM summary
), calculated AS (
 SELECT *,CASE WHEN between_variance=0 THEN NULL ELSE (m-1)*POWER(1+1/relative_increase,2) END AS degrees_freedom FROM totals
), checked AS MATERIALIZED (
 SELECT *,CASE WHEN NOT isfinite(qbar) OR NOT isfinite(within_variance) OR within_variance<=0 OR NOT isfinite(between_variance) OR between_variance<0 OR NOT isfinite(total_variance) OR total_variance<=0 OR NOT isfinite(relative_increase) OR (between_variance>0 AND (degrees_freedom IS NULL OR NOT isfinite(degrees_freedom) OR degrees_freedom<1 OR degrees_freedom>1000000)) OR NOT isfinite(qbar/SQRT(total_variance)) OR ABS(qbar/SQRT(total_variance))>1000000
 THEN error('Rubin pooling requires resolved finite variance, t and degrees of freedom') ELSE qbar/SQRT(total_variance) END AS statistic FROM calculated
), inference AS (
 SELECT *,CASE WHEN between_variance=0 THEN 2*tabdat_gauss_phi(-ABS(statistic)) ELSE tabdat_t_pvalue(statistic,degrees_freedom) END AS p_value,
 CASE WHEN between_variance=0 THEN 1.959963984540054 ELSE tabdat_t_quantile(.975,degrees_freedom) END AS critical,
 CASE WHEN between_variance=0 THEN 0 ELSE (relative_increase+2/(degrees_freedom+3))/(1+relative_increase) END AS missing_information
 FROM checked
)
SELECT parameter,m AS imputations,qbar AS estimate,within_variance,between_variance,total_variance,SQRT(total_variance) AS standard_error,
 relative_increase,degrees_freedom,statistic,
 CASE WHEN p_value IS NULL OR critical IS NULL OR NOT isfinite(p_value) OR NOT isfinite(critical) THEN error('Rubin pooling Student-t inference is unresolved') ELSE p_value END AS p_value,
 qbar-critical*SQRT(total_variance) AS normal_or_t95_lower,qbar+critical*SQRT(total_variance) AS normal_or_t95_upper,
 missing_information,1/(1+missing_information/m) AS relative_efficiency,minimum,maximum,
 CASE WHEN between_variance=0 THEN 'normal_infinite_df_limit' ELSE 'rubin_student_t_large_complete_df' END AS reference
FROM inference ORDER BY parameter
""" into rubin_marginal_report
sql """SELECT * FROM active"""
export rubin_marginal_report.csv
status
