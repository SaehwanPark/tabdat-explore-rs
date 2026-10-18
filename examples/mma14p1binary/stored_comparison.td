# Table14.2 native reporting snapshots; same preparation/predictions as comparison.td.
# Run in a fresh output directory; report names are local to the current session.
use Nldata.csv
sql """SELECT source_row, mode, CASE WHEN mode = 4 THEN 1 ELSE 0 END AS charter, pcharter, ppier, ln(pcharter/ppier) AS lnrelp FROM active WHERE mode IN (2,4) ORDER BY source_row""" into fishing_binary
isid source_row
logit charter lnrelp
estimates store blogit
predict logit_classical_pr, pr
predict logit_classical_xb, xb
probit charter lnrelp
estimates store bprobit
predict probit_classical_pr, pr
predict probit_classical_xb, xb
regress charter lnrelp
estimates store bOLS
predict ols_classical_xb, xb
logit charter lnrelp, robust
estimates store bloghet
predict logit_robust_pr, pr
predict logit_robust_xb, xb
probit charter lnrelp, robust
estimates store bprobhet
predict probit_robust_pr, pr
predict probit_robust_xb, xb
regress charter lnrelp, robust
estimates store bOLShet
predict ols_robust_xb, xb
estimates table blogit bprobit bOLS bloghet bprobhet bOLShet
export fishing_comparison_predictions.csv
