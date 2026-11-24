use binary_discrete_change.csv
probit y x treated, robust
estat margins, discrete(treated)
