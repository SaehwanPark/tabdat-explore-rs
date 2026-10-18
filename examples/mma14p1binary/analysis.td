# Complete §14.2 author-output workflow under the README's native conventions.
# Use a fresh output directory/session: no file replacement or report overwrite.
use Nldata.csv
isid source_row
generate ydiv1000 = income/1000
summarize mode price crate dbeach dpier dprivate dcharter pbeach ppier pprivate pcharter qbeach qpier qprivate qcharter income ydiv1000
export fishing_source_sample.csv
# Keep original source-row identities/order, not the author's in-place mode sort.
sql """SELECT *, CASE WHEN mode = 4 THEN 1 ELSE 0 END AS charter, 100*ln(pcharter/ppier) AS pratio, ln(pcharter/ppier) AS lnrelp FROM active WHERE mode IN (2,4) ORDER BY source_row""" into fishing_prepared
isid source_row
summarize mode price crate dbeach dpier dprivate dcharter pbeach ppier pprivate pcharter qbeach qpier qprivate qcharter income ydiv1000 charter pratio lnrelp
summarize mode price crate dbeach dpier dprivate dcharter pbeach ppier pprivate pcharter qbeach qpier qprivate qcharter income ydiv1000 charter pratio lnrelp if mode == 2
summarize mode price crate dbeach dpier dprivate dcharter pbeach ppier pprivate pcharter qbeach qpier qprivate qcharter income ydiv1000 charter pratio lnrelp if mode == 4
export fishing_prepared_sample.csv
# Explicit Table14.1 blocks, in the author's overall/pier/charter order.
summarize charter pcharter ppier lnrelp
summarize charter pcharter ppier lnrelp if mode == 2
summarize charter pcharter ppier lnrelp if mode == 4
# The leaf workflow reloads/reprepares the identical cohort, stores six reports,
# produces native Table14.2 and exports the independently checked predictions.
run stored_comparison.td
# Reuse those classical fitted values rather than quietly refitting three models.
summarize charter logit_classical_pr probit_classical_pr ols_classical_xb
label variable charter "Charter choice / fitted value"
label variable lnrelp "Log relative price: ln(pcharter/ppier)"
label variable logit_classical_pr "Logit probability"
label variable probit_classical_pr "Probit probability"
label variable ols_classical_xb "OLS fitted value (unbounded)"
# Renderer sorts lines privately; jitter is display-only, not Stata RNG parity.
scatter charter lnrelp, lines(logit_classical_pr probit_classical_pr ols_classical_xb) jitter(0.03) title("Predicted probabilities and OLS fitted values") saving("fishing_binary_comparison.svg") noopen
