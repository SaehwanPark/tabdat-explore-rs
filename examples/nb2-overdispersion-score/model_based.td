use score_reference.csv
poisson y_independent x, offset(log_exposure)
estat overdispersion
