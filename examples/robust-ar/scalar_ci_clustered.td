use robust_ar_ci_data.csv
ivregress 2sls y w, endog(x) iv(z) cluster(g)
estat ar, ci
replace y = 999
drop z
estat ar, ci
