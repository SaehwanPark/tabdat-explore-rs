use score_reference.csv
poisson y_clustered x, offset(log_exposure) cluster(group)
estat overdispersion
