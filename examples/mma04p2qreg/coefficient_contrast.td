# Joint IID pairs inference for the author's .25/.75 log-expenditure slope test.
use qreg.csv
isid source_row
sql """SELECT * FROM active WHERE isfinite(lhhexp1) AND isfinite(lhhex12m) ORDER BY source_row""" into qreg_cohort
generate lntotal = lhhexp1
generate lnmed = lhhex12m
qreg lnmed lntotal
bootstrap lnmed lntotal, quantiles(.25 .75) contrast(lntotal) seed(10101) reps(200)
predict median
predict median_residual, residuals
select source_row lntotal lnmed median median_residual
export quantile_contrast_cohort.csv
