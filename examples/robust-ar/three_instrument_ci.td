use robust_ar_three_ci_data.csv
ivregress 2sls y w, endog(x) iv(z1 z2 z3) robust
estat ar, ci
