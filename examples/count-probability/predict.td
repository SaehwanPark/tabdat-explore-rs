# Copy examples/nb2-offset/nb2_offset.csv to the working directory first.
use nb2_offset.csv
poisson y x z, offset(log_time) robust
generate cutoff = id - 1
predict poisson_p0, pmf(0)
predict poisson_p0_se, pmf(0) se
predict poisson_p0_interval, pmf(0) interval
predict poisson_future, n interval level(90)
predict poisson_response_residual, residuals
predict poisson_cdf2, cdf(2)
predict poisson_cdf2_se, cdf(2) se
predict poisson_cdf2_interval, cdf(2) interval level(90)
predict poisson_row_pmf, pmf(cutoff)
predict poisson_row_pmf_se, pmf(cutoff) se
predict poisson_row_cdf, cdf(cutoff)
predict poisson_row_cdf_se, cdf(cutoff) se
predict poisson_row_cdf_interval, cdf(cutoff) interval
export poisson_probabilities.csv
nbreg y x z, offset(log_time) cluster(person)
# Retained fit with changed current outcome, predictors and offset.
replace y = y + 1
replace x = x + .2
replace log_time = log_time - .3
gsort -id
predict nb2_mean, n
predict nb2_response_residual, residuals
# Probability predictions need no current outcome or group ID.
drop y person
predict nb2_p0, pmf(0)
predict nb2_p2, pmf(2)
predict nb2_p2_se, pmf(2) se
predict nb2_p2_interval, pmf(2) interval
predict nb2_future, n interval
predict nb2_cdf2, cdf(2)
predict nb2_cdf2_se, cdf(2) se
predict nb2_cdf2_interval, cdf(2) interval
predict nb2_row_pmf, pmf(cutoff)
predict nb2_row_pmf_se, pmf(cutoff) se
predict nb2_row_cdf, cdf(cutoff)
predict nb2_row_cdf_se, cdf(cutoff) se
predict nb2_row_cdf_interval, cdf(cutoff) interval
export nb2_probabilities.csv
