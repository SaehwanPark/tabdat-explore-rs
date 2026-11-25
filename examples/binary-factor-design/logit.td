use binary_factor_data.csv
generate treated = 0
replace treated = 1 if x > 0
logit y treated x i.region
lincom treated + region_North - region_West + 0.5
estat gof, group(10)
estat classification
estat margins
estat margins, discrete(treated)
estat margins, factor(region)
predict index, xb
predict probability, pr
estimates store logit_factor
estimates table logit_factor
export logit_factor_predictions.csv, replace
