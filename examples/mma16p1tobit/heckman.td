# Author200-row two-step comparison; no exclusion restriction, not a published table replay.
use mma16p1tobit.csv
heckman ytrunc lnwage, selectdep(dy) select(lnwage) twostep
predict latent, xb
predict selection_probability, pr
predict inverse_mills, mills
predict conditional_selected_mean, ycond
predict observed_contribution_mean, yexpected
export heckman_predictions.csv
