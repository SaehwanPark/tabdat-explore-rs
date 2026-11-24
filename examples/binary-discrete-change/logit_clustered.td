use binary_discrete_change.csv
logit y x treated, cluster(group)
estat margins, discrete(treated)
