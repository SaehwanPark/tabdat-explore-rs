use nb2_offset.csv
summarize y x z log_time
nbreg y x z, offset(log_time) robust
estat margins
estat countprob, max(5)
predict mu, n
predict eta, xb
head 5
export nb2_offset_predictions.csv
