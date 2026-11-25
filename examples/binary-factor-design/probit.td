use binary_factor_data.csv
probit y x i.region
estat gof, group(10)
estat margins, factor(region)
predict index, xb
predict probability, pr
estimates store probit_factor
estimates table probit_factor
export probit_factor_predictions.csv, replace
