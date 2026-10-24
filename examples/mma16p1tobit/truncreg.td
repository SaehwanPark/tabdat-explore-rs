use mma16p1tobit.csv
truncreg ytrunc lnwage, ll(0)
estat margins
predict latent, xb
predict retention_probability, pr
predict conditional_mean, mean
export truncreg_predictions.csv
