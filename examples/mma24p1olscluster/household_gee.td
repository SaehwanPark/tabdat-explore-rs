# Gaussian population-averaged household model: source_row is a key, not time.
use vietnam.csv
isid source_row
sql """SELECT * FROM active ORDER BY source_row""" into household_gee_source
xtgee lnexp12m lnhhexp age sex hhsize farm educ, family(gaussian) link(identity) corr(exchangeable) id(commune)
predict gee_mean
predict gee_xb, xb
predict gee_residual, residuals
summarize gee_mean gee_residual
sql """SELECT *, (lnexp12m IS NOT NULL AND isfinite(lnexp12m) AND lnhhexp IS NOT NULL AND isfinite(lnhhexp) AND age IS NOT NULL AND isfinite(age) AND sex IS NOT NULL AND isfinite(sex) AND hhsize IS NOT NULL AND isfinite(hhsize) AND farm IS NOT NULL AND isfinite(farm) AND educ IS NOT NULL AND isfinite(educ) AND commune IS NOT NULL AND isfinite(commune)) AS included_in_gee FROM active ORDER BY source_row""" into household_gee_predictions
sql """SELECT 'Gaussian identity-link exchangeable GEE with RSS/N and pair-weighted correlation. Working and uncorrected commune sandwich are distinct; not a survey-design or causal analysis.' AS scope"""
export household_gee_predictions.csv
