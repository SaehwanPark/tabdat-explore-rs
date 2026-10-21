# MMA section 4.6.4: log medical expenditure on log total household expenditure.
# Preserve zeros in the log outcome; only joint nonfinite/missing cases are removed.
use qreg.csv
isid source_row
sql """SELECT * FROM active WHERE isfinite(lhhexp1) AND isfinite(lhhex12m) ORDER BY source_row""" into qreg_cohort
generate lntotal = lhhexp1
generate lnmed = lhhex12m
label variable lntotal "Log total household expenditure"
label variable lnmed "Log household medical expenditure"
qreg lnmed lntotal, quantile(0.1)
predict q10
qreg lnmed lntotal
predict q50
predict median_residual, residuals
qreg lnmed lntotal, quantile(0.9)
predict q90
export quantile_predictions.csv
