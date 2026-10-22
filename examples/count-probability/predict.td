# Copy examples/nb2-offset/nb2_offset.csv to the working directory first.
use nb2_offset.csv
poisson y x z, offset(log_time) robust
generate cutoff = id - 1
predict poisson_p0, pmf(0)
predict poisson_cdf2, cdf(2)
predict poisson_row_pmf, pmf(cutoff)
predict poisson_row_cdf, cdf(cutoff)
export poisson_probabilities.csv
nbreg y x z, offset(log_time) cluster(person)
# Retained fit, with changed current predictors/offsets and no outcome/group columns.
drop y person
replace x = x + .2
replace log_time = log_time - .3
gsort -id
predict nb2_mean, n
predict nb2_p0, pmf(0)
predict nb2_p2, pmf(2)
predict nb2_cdf2, cdf(2)
predict nb2_row_pmf, pmf(cutoff)
predict nb2_row_cdf, cdf(cutoff)
export nb2_probabilities.csv
