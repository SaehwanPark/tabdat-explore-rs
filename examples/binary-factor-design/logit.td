use binary_factor_data.csv
logit y x i.region
estat gof, group(10)
predict index, xb
predict probability, pr
estimates store logit_factor
estimates table logit_factor
export logit_factor_predictions.csv, replace
