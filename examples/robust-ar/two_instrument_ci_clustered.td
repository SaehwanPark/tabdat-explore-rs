use robust_ar_multi_ci_data.csv
ivregress 2sls y w, endog(x) iv(z1 z2) cluster(group)
estat ar, ci
replace y = 999
drop z1
estat ar, ci
