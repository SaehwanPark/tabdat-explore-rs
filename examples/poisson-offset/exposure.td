use poisson_offset.csv
sql """SELECT *, exp(log_time) AS exposure FROM active""" into exposure_design
poisson y x z, exposure(exposure) robust
estat overdispersion
estat margins
predict poisson_mean, n
predict poisson_xb, xb
estimates store poisson_exposure
estimates table poisson_exposure
nbreg y x z, exposure(exposure) robust
estat margins
predict nb2_mean, n
predict nb2_xb, xb
estimates store nb2_exposure
estimates table nb2_exposure
