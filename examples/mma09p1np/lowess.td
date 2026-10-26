# Figure9.3 source-data adaptation: explicit nonrobust centred-rank LOWESS.
# Stable within-education input ties, not certified historical Stata curve order.
run prepare.td
# The complete three-span comparison needs selected N>=5 for N*0.1>=0.5.
# The native command errors rather than silently clipping a too-small span.
lowess lnhwage educatn, gen(lowess_08) bandwidth=0.8
lowess lnhwage educatn, gen(lowess_04) bandwidth=0.4
lowess lnhwage educatn, gen(lowess_01) bandwidth=0.1
# Diagnose ties explicitly: within-X fit ranges are NOT confidence intervals.
sql """
SELECT educatn,COUNT(*) AS n,AVG(lnhwage) AS observed_mean,
  MIN(lowess_01) AS span01_min,MAX(lowess_01) AS span01_max
FROM active GROUP BY educatn ORDER BY educatn
"""
export psid_lowess.csv
label variable educatn "Years of schooling"
label variable lnhwage "Log hourly wage (observations)"
label variable lowess_08 "LOWESS span 0.8"
label variable lowess_04 "LOWESS span 0.4"
label variable lowess_01 "LOWESS span 0.1"
# Actual raw observations, not evaluation-grid marks or confidence bands.
# Equal X can have distinct fits: these are explicitly ordered row smoothers.
scatter lnhwage educatn, lines(lowess_08 lowess_04 lowess_01) title("Centered-rank LOWESS as Span Varies: PSID age 36") saving("psid_lowess.svg") noopen
