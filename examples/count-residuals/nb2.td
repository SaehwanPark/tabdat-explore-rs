use count_residuals.csv
nbreg y x z, exposure(exposure) robust
predict fitted_mean, n
predict response_residual, residuals
predict pearson_residual, pearson
predict deviance_residual, deviance
head 8
export nb2_residuals.csv, replace
