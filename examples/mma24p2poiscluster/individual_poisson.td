# MMA §24.7 pooled Poisson only; not FE/RE or survey design.
use vietnam_individual.csv
isid source_row
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(*) FILTER (WHERE typeof(source_row)!='BIGINT' OR source_row IS NULL OR source_row<1 OR CASE WHEN pharvis IS NULL AND lnhhexp IS NULL AND age IS NULL AND sex IS NULL AND married IS NULL AND educ IS NULL AND illness IS NULL AND injury IS NULL AND illdays IS NULL AND actdays IS NULL AND insurance IS NULL AND commune IS NULL THEN FALSE ELSE typeof(pharvis)!='BIGINT' OR pharvis IS NULL OR pharvis<0 OR typeof(commune)!='BIGINT' OR commune IS NULL OR commune<1 OR lnhhexp IS NULL OR NOT isfinite(lnhhexp) OR age IS NULL OR NOT isfinite(age) OR sex IS NULL OR NOT isfinite(sex) OR married IS NULL OR NOT isfinite(married) OR educ IS NULL OR NOT isfinite(educ) OR illness IS NULL OR NOT isfinite(illness) OR injury IS NULL OR NOT isfinite(injury) OR illdays IS NULL OR NOT isfinite(illdays) OR actdays IS NULL OR NOT isfinite(actdays) OR insurance IS NULL OR NOT isfinite(insurance) END)>0 THEN error('invalid count, commune key or partial individual record') ELSE COUNT(*) END AS source_rows,COUNT(*) FILTER (WHERE pharvis IS NULL) AS entirely_missing_rows,COUNT(DISTINCT commune) AS source_communes FROM active"""
sql """SELECT * FROM active WHERE pharvis IS NOT NULL ORDER BY source_row""" into individual_cohort
summarize pharvis lnhhexp age
poisson pharvis lnhhexp insurance sex age married illdays actdays injury illness educ
estat margins
poisson pharvis lnhhexp insurance sex age married illdays actdays injury illness educ, robust
estat margins
poisson pharvis lnhhexp insurance sex age married illdays actdays injury illness educ, cluster(commune)
estat margins
predict pooled_mu, n
predict pooled_xb, xb
summarize pooled_mu pooled_xb
sql """SELECT 'Pooled Poisson with alternative model, independent-row, and commune-clustered covariance; not a survey design, FE/RE, or causal analysis. The retained model is commune-clustered Poisson.' AS scope"""
export individual_poisson_predictions.csv
