# Chapter24 shared-gamma RE whole-commune pairs bootstrap; source_row is not time.
use vietnam_individual.csv
isid source_row
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(*)>27766 OR COUNT(*) FILTER (WHERE typeof(source_row)!='BIGINT' OR source_row IS NULL OR source_row<1 OR source_row>27766 OR CASE WHEN pharvis IS NULL AND lnhhexp IS NULL AND age IS NULL AND sex IS NULL AND married IS NULL AND educ IS NULL AND illness IS NULL AND injury IS NULL AND illdays IS NULL AND actdays IS NULL AND insurance IS NULL AND commune IS NULL THEN FALSE ELSE typeof(pharvis)!='BIGINT' OR pharvis IS NULL OR pharvis<0 OR pharvis>100000 OR typeof(commune)!='BIGINT' OR commune IS NULL OR commune<1 OR commune>194 OR lnhhexp IS NULL OR NOT isfinite(lnhhexp) OR age IS NULL OR NOT isfinite(age) OR sex IS NULL OR NOT isfinite(sex) OR married IS NULL OR NOT isfinite(married) OR educ IS NULL OR NOT isfinite(educ) OR illness IS NULL OR NOT isfinite(illness) OR injury IS NULL OR NOT isfinite(injury) OR illdays IS NULL OR NOT isfinite(illdays) OR actdays IS NULL OR NOT isfinite(actdays) OR insurance IS NULL OR NOT isfinite(insurance) END)>0 THEN error('invalid count, commune key or partial individual record') ELSE COUNT(*) END AS source_rows,COUNT(*) FILTER (WHERE pharvis IS NULL) AS entirely_missing_rows,COUNT(DISTINCT commune) AS source_communes FROM active"""
sql """SELECT *,SUM(pharvis ORDER BY source_row) OVER (PARTITION BY commune) AS selected_commune_total,TRUE AS included_in_re FROM active WHERE pharvis IS NOT NULL ORDER BY source_row""" into individual_re_cohort
xtpoisson pharvis lnhhexp insurance sex age married illdays actdays injury illness educ, re id(commune)
bootstrap pharvis lnhhexp insurance sex age married illdays actdays injury illness educ, model(gamma_poisson_re) cluster(commune) reps(500) seed(10001)
predict re_mean, n
predict re_xb, xb
predict re_residual, residuals
summarize re_mean re_residual
sql """SELECT 'Shared-gamma RE whole-commune pairs with distinct copied effects; B=500 has Monte Carlo error. All-zero communes retained. Unweighted, not survey-design or causal inference. Means are original-fit marginal predictions, not posterior effects; draws differ from Stata.' AS scope"""
export individual_gamma_re_bootstrap_predictions.csv
