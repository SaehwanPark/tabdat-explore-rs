use Nldata.csv
isid source_row
generate ydiv1000 = income/1000
mlogit mode ydiv1000, baseoutcome(1)
estat margins
predict p1, pr outcome(1)
predict xb1, xb outcome(1)
predict p2, pr outcome(2)
predict xb2, xb outcome(2)
predict p3, pr outcome(3)
predict xb3, xb outcome(3)
predict p4, pr outcome(4)
predict xb4, xb outcome(4)
export fishing_mnl_predictions.csv
