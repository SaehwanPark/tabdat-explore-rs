use robust_ar_data.csv
ivregress 2sls y w, endog(x1 x2) iv(z1 z2 z3) robust
estat ar, null(1.7 -0.6)
ivregress 2sls y w, endog(x1 x2) iv(z1 z2 z3) cluster(g)
estat ar, null(1.7 -0.6)
