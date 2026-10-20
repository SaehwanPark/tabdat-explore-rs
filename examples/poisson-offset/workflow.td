use poisson_offset.csv
summarize y x z log_time
poisson y x z, offset(log_time) robust
estat margins
estat countprob, max(5)
predict mu, n
predict eta, xb
head 5
export poisson_offset_predictions.csv
