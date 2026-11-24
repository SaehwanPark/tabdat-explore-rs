use projected_ar_data.csv
ivregress 2sls y w, endog(x1 x2) iv(z1 z2) cluster(group)
estat ar, ci coefficient(x1) level(95)
