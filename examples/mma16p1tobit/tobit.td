# Author200-row export; lower-censored IID Gaussian program extension, not a table replay.
use mma16p1tobit.csv
tobit ycens lnwage, ll(0)
estat margins
predict latent, xb
predict uncensored_probability, pr
predict observed_mean, mean
export tobit_predictions.csv
