# First two Table 5.7 columns; the Poisson primitive has a count-pmf likelihood.
# For the exponential density subtract sum(log(y)); see comparison.td.
use nls.csv
isid source_row
sql """SELECT 'The Poisson columns fit exponential rates through event/exposure algebra; their likelihood includes the log(y) offset' AS interpretation FROM active LIMIT 1"""
regress y x
estimates store ols_working
regress y x, robust
estimates store ols_hc1
sql """SELECT *,1 AS event,ln(y) AS log_exposure FROM active ORDER BY source_row""" into exponential_design
poisson event x, offset(log_exposure)
estimates store exp_working
poisson event x, offset(log_exposure) robust
estimates store exp_robust
estimates table ols_working ols_hc1 exp_working exp_robust
