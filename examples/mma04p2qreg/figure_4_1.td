# MMA §4.6.4 Figure 4.1: pointwise quantile-slope bands from a common row bootstrap.
use qreg.csv
isid source_row
generate lntotal = lhhexp1
generate lnmed = lhhex12m
label variable lntotal "Log total household expenditure"
label variable lnmed "Log medical expenditure"
qregplot lnmed lntotal, coefficient(lntotal) quantiles(.05 .1 .15 .2 .25 .3 .35 .4 .45 .5 .55 .6 .65 .7 .75 .8 .85 .9 .95) seed(10101) reps(200) saving("quantile_slope_bands.svg") noopen title("Slope Estimates as Quantile Varies")
