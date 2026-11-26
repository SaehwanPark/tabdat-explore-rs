# Balanced Gaussian identity GEE with a retained unstructured working R.
use MOM.csv
isid id year
isid source_row
xtgee lnhr lnwg, family(gaussian) link(identity) corr(unstructured) id(id) time(year)
predict unstructured_mean, xb
predict unstructured_residual, residuals
estimates store unstructured_working, covariance(working)
estimates store unstructured_sandwich, covariance(sandwich)
estimates table unstructured_working unstructured_sandwich
export unstructured_gee_predictions.csv
