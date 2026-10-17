use Nldata.csv
sql """SELECT source_row, mode, CASE WHEN mode = 4 THEN 1 ELSE 0 END AS charter, pcharter, ppier, ln(pcharter/ppier) AS lnrelp FROM active WHERE mode IN (2,4) ORDER BY source_row""" into fishing_binary
probit charter lnrelp
estat margins
predict phat, pr
predict pindex, xb
export fishing_probit_predictions.csv
