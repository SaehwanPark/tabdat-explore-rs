use Nldata.csv
sql """SELECT source_row, mode, CASE WHEN mode = 4 THEN 1 ELSE 0 END AS charter, pcharter, ppier, ln(pcharter/ppier) AS lnrelp FROM active WHERE mode IN (2,4) ORDER BY source_row""" into fishing_binary
logit charter lnrelp
estat margins
predict phat, pr
predict log_odds, xb
export fishing_logit_predictions.csv
