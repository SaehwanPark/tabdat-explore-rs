# Conditional Poisson commune FE on individuals: no temporal interpretation.
use vietnam_individual.csv
isid source_row
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(*) FILTER (WHERE typeof(source_row)!='BIGINT' OR source_row IS NULL OR source_row<1 OR CASE WHEN pharvis IS NULL AND lnhhexp IS NULL AND age IS NULL AND sex IS NULL AND married IS NULL AND educ IS NULL AND illness IS NULL AND injury IS NULL AND illdays IS NULL AND actdays IS NULL AND insurance IS NULL AND commune IS NULL THEN FALSE ELSE typeof(pharvis)!='BIGINT' OR pharvis IS NULL OR pharvis<0 OR pharvis>100000 OR typeof(commune)!='BIGINT' OR commune IS NULL OR commune<1 OR lnhhexp IS NULL OR NOT isfinite(lnhhexp) OR age IS NULL OR NOT isfinite(age) OR sex IS NULL OR NOT isfinite(sex) OR married IS NULL OR NOT isfinite(married) OR educ IS NULL OR NOT isfinite(educ) OR illness IS NULL OR NOT isfinite(illness) OR injury IS NULL OR NOT isfinite(injury) OR illdays IS NULL OR NOT isfinite(illdays) OR actdays IS NULL OR NOT isfinite(actdays) OR insurance IS NULL OR NOT isfinite(insurance) END)>0 THEN error('invalid count, commune key or partial individual record') ELSE COUNT(*) END AS source_rows,COUNT(*) FILTER (WHERE pharvis IS NULL) AS entirely_missing_rows,COUNT(DISTINCT commune) AS source_communes FROM active"""
sql """WITH cohort AS (SELECT *,SUM(pharvis ORDER BY source_row) OVER (PARTITION BY commune) AS selected_commune_total FROM active WHERE pharvis IS NOT NULL) SELECT *,selected_commune_total>0 AS included_in_fe FROM cohort ORDER BY source_row""" into individual_fe_cohort
xtpoisson pharvis lnhhexp insurance sex age married illdays actdays injury illness educ, fe id(commune)
predict fe_mean, n
predict fe_xb, xb
predict fe_residual, residuals
summarize fe_mean fe_residual
sql """SELECT 'Conditional Poisson FE given selected commune count totals; model-based normal inference, not cluster-robust, survey-design or causal analysis. Cross-sectional source_row is not time. Means use retained positive-commune normalizations; excluded zero-total communes have NULL means/residuals.' AS scope"""
export individual_conditional_fe_predictions.csv
