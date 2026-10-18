# Figure14.1 data: the exact §14.2 cohort and six validated fits/predictions.
# Use a fresh output directory: neither the prediction CSV nor SVG is replaced.
run comparison.td
label variable charter "Charter choice / fitted value"
label variable lnrelp "Log relative price: ln(pcharter/ppier)"
label variable logit_classical_pr "Logit probability"
label variable probit_classical_pr "Probit probability"
label variable ols_classical_xb "OLS fitted value (unbounded)"
# Jitter is deterministic y-only display perturbation, not Stata's RNG jitter.
scatter charter lnrelp, lines(logit_classical_pr probit_classical_pr ols_classical_xb) jitter(0.03) title("Predicted probabilities and OLS fitted values") saving("fishing_binary_comparison.svg") noopen
