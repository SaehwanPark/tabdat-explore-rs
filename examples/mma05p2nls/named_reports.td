# Compare the unweighted NLS fit under working and HC1 covariance.
# Reports retain the formula and fit-time sample, but do not restore fits.
use nls.csv
isid source_row
nl y = exp(-a-b*x), params(a b) start(1 0)
estimates store nls_working
nl y = exp(-a-b*x), params(a b) start(1 0) robust
estimates store nls_hc1
estimates table nls_working nls_hc1
