use Nldata.csv
sql """SELECT source_row, mode, CASE WHEN mode = 4 THEN 1 ELSE 0 END AS charter, pcharter, ppier, ln(pcharter/ppier) AS lnrelp FROM active WHERE mode IN (2,4) ORDER BY source_row""" into fishing_binary
isid source_row
logit charter lnrelp
predict logit_classical_pr, pr
predict logit_classical_xb, xb
probit charter lnrelp
predict probit_classical_pr, pr
predict probit_classical_xb, xb
regress charter lnrelp
predict ols_classical_xb, xb
logit charter lnrelp, robust
predict logit_robust_pr, pr
predict logit_robust_xb, xb
probit charter lnrelp, robust
predict probit_robust_pr, pr
predict probit_robust_xb, xb
regress charter lnrelp, robust
predict ols_robust_xb, xb
export fishing_comparison_predictions.csv
