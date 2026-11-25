use binary_factor_data.csv
generate treated = 0
replace treated = 1 if x > 0
probit y treated x i.region
lincom treated + region_North - region_West + 0.5
estat gof, group(10)
estat margins
estat margins, discrete(treated)
estat margins, factor(region)
predict index, xb
predict probability, pr
estimates store probit_factor
estimates table probit_factor
export probit_factor_predictions.csv, replace
