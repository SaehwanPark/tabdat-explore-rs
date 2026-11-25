use robust_ar_seven_ci_multiple_data.csv
ivregress 2sls y w, endog(x) iv(z1 z2 z3 z4 z5 z6 z7) robust
estat ar, ci
