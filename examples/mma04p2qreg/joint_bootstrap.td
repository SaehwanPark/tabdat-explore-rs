# IID pairs method counterpart to the joint fits underlying Figure4.1.
# Bands/contrasts and the historical Stata draw stream are not reproduced.
use qreg.csv
isid source_row
sql """SELECT * FROM active WHERE isfinite(lhhexp1) AND isfinite(lhhex12m) ORDER BY source_row""" into qreg_cohort
generate lntotal = lhhexp1
generate lnmed = lhhex12m
qreg lnmed lntotal
bootstrap lnmed lntotal, quantiles(.05 .1 .15 .2 .25 .3 .35 .4 .45 .5 .55 .6 .65 .7 .75 .8 .85 .9 .95) seed(10101) reps(200)
predict median
predict median_residual, residuals
select source_row lntotal lnmed median median_residual
export quantile_bootstrap_cohort.csv
