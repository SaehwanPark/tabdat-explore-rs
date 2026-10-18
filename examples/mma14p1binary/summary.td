# Table14.1: overall, pier, then charter; SD uses n-1, not n.
# Run in a fresh output directory containing the converted Nldata.csv.
use Nldata.csv
sql """SELECT source_row, mode, CASE WHEN mode = 4 THEN 1 ELSE 0 END AS charter, pcharter, ppier, ln(pcharter/ppier) AS lnrelp FROM active WHERE mode IN (2,4) ORDER BY source_row""" into fishing_summary
isid source_row
summarize charter pcharter ppier lnrelp
summarize charter pcharter ppier lnrelp if mode == 2
summarize charter pcharter ppier lnrelp if mode == 4
export fishing_summary_sample.csv
