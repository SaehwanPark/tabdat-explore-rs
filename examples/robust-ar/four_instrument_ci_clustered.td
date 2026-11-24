use robust_ar_four_ci_four_roots_data.csv
ivregress 2sls y w, endog(x) iv(z1 z2 z3 z4) cluster(group)
estat ar, ci
