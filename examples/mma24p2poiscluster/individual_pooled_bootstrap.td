# Chapter24 individual pooled whole-commune Poisson bootstrap; source_row is not time.
use vietnam_individual.csv
isid source_row
sql """SELECT CASE WHEN COUNT(*)=0 OR COUNT(*)>27766 OR COUNT(*) FILTER (WHERE typeof(source_row)!='BIGINT' OR source_row IS NULL OR source_row<1 OR source_row>27766 OR CASE WHEN pharvis IS NULL AND lnhhexp IS NULL AND age IS NULL AND sex IS NULL AND married IS NULL AND educ IS NULL AND illness IS NULL AND injury IS NULL AND illdays IS NULL AND actdays IS NULL AND insurance IS NULL AND commune IS NULL THEN FALSE ELSE typeof(pharvis)!='BIGINT' OR pharvis IS NULL OR pharvis<0 OR pharvis>100000 OR typeof(commune)!='BIGINT' OR commune IS NULL OR commune<1 OR commune>194 OR lnhhexp IS NULL OR NOT isfinite(lnhhexp) OR abs(lnhhexp)>1000000 OR age IS NULL OR NOT isfinite(age) OR abs(age)>1000000 OR sex IS NULL OR NOT isfinite(sex) OR abs(sex)>1000000 OR married IS NULL OR NOT isfinite(married) OR abs(married)>1000000 OR educ IS NULL OR NOT isfinite(educ) OR abs(educ)>1000000 OR illness IS NULL OR NOT isfinite(illness) OR abs(illness)>1000000 OR injury IS NULL OR NOT isfinite(injury) OR abs(injury)>1000000 OR illdays IS NULL OR NOT isfinite(illdays) OR abs(illdays)>1000000 OR actdays IS NULL OR NOT isfinite(actdays) OR abs(actdays)>1000000 OR insurance IS NULL OR NOT isfinite(insurance) OR abs(insurance)>1000000 END)>0 THEN error('invalid individual source, count, commune key or partial record') ELSE COUNT(*) END AS source_rows, COUNT(*) FILTER(WHERE pharvis IS NULL) AS missing_rows FROM active"""
sql """SELECT * FROM active WHERE pharvis IS NOT NULL ORDER BY source_row""" into individual_pooled_bootstrap_cohort
sql """SELECT CASE WHEN COUNT(*)<2 OR COUNT(*)>194 OR MAX(n)>206 THEN error('individual bootstrap requires 2..194 communes of size at most 206') ELSE COUNT(*) END AS communes, SUM(n) AS observations, MIN(n) AS minimum_size, MAX(n) AS maximum_size FROM (SELECT commune,COUNT(*) AS n FROM active GROUP BY commune)"""
poisson pharvis lnhhexp insurance sex age married illdays actdays injury illness educ, cluster(commune)
bootstrap pharvis lnhhexp insurance sex age married illdays actdays injury illness educ, model(poisson) cluster(commune) reps(500) seed(10001)
predict pooled_mu, n
predict pooled_xb, xb
summarize pooled_mu pooled_xb
sql """SELECT 'Whole-commune pooled Poisson pairs; B=500 has Monte Carlo error. This is unweighted, not FE/RE, survey-design or causal inference. Native draws differ from Stata.' AS bootstrap_scope"""
export individual_pooled_bootstrap_predictions.csv
