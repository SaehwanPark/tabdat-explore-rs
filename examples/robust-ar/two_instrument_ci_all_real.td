use robust_ar_multi_ci_all_real_data.csv
ivregress 2sls y w, endog(x) iv(z1 z2) robust
estat ar, ci
