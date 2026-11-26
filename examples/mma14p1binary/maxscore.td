# Maximum-score point fit for the author’s charter-versus-pier cohort.
use Nldata.csv
sql """SELECT source_row, mode, CASE WHEN mode = 4 THEN 1 ELSE 0 END AS charter,
  ln(pcharter/ppier) AS lnrelp FROM active WHERE mode IN (2,4)
  ORDER BY source_row""" into fishing_maxscore
isid source_row
maxscore charter lnrelp
predict predicted_charter, class
export fishing_maxscore_predictions.csv
