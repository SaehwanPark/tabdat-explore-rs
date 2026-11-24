use robust_ar_four_ci_all_real_data.csv
ivregress 2sls y w, endog(x) iv(z1 z2 z3 z4) robust
estat ar, ci
