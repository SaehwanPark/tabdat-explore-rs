use robust_ar_thirteen_ci_multiple_data.csv
ivregress 2sls y w, endog(x) iv(z1 z2 z3 z4 z5 z6 z7 z8 z9 z10 z11 z12 z13) cluster(group)
estat ar, ci
