use Contraception.csv
sql """SELECT woman, district, CASE WHEN "use"='Y' THEN 1 WHEN "use"='N' THEN 0 ELSE NULL END AS y, age FROM active ORDER BY woman""" into contraception
isid woman
logit y age, cluster(district)
estat margins
predict logit_pr, pr
predict logit_xb, xb
probit y age, cluster(district)
estat margins
predict probit_pr, pr
predict probit_xb, xb
export clustered_predictions.csv
