# Synthetic whole-cluster pairs bootstrap for joint conditional quantiles.
# See README.md for assumptions, reference evidence and the input fixture.
use data.csv
regress y x z
predict original_fit
bootstrap y x z, quantiles(.25 .5 .75) cluster(group) seed(918273) reps(80)
predict preserved_fit
select row y x z group original_fit preserved_fit
export cluster_quantile_predictions.csv
