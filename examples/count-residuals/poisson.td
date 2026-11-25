use count_residuals.csv
poisson y x z, offset(log_time) cluster(person)
predict fitted_mean, n
predict response_residual, residuals
predict pearson_residual, pearson
predict deviance_residual, deviance
head 8
export poisson_residuals.csv, replace
