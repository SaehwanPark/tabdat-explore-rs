# Native independent-working Gaussian identity-link GEE on source MOM rows.
use MOM.csv
isid id year
isid source_row
xtgee lnhr lnwg, family(gaussian) link(identity) corr(independent) id(id)
predict native_independent_mean, xb
predict native_independent_residual, residuals
estimates store p4_independent_working, covariance(working)
estimates store p4_independent_sandwich, covariance(sandwich)
estimates table p4_independent_working p4_independent_sandwich
sql """SELECT source_row,lnhr,lnwg,id,year,native_independent_mean,native_independent_residual FROM active ORDER BY source_row"""
export native_independent_gee_predictions.csv
