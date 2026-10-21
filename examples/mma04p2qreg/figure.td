# Figure 4.2: fit the source cohort and overlay its three conditional quantiles.
run quantiles.td
label variable q10 "10th percentile"
label variable q50 "Median"
label variable q90 "90th percentile"
scatter lnmed lntotal, lines(q90 q50 q10) title("Regression Lines as Quantile Varies") saving("quantile_lines.svg") noopen
