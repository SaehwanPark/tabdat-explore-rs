Robust joint Anderson–Rubin tests are planned for retained unweighted 2SLS fits. The auxiliary regression is y − X_endog b0 on included exogenous variables and all excluded instruments. HC1 uses N/(N−K_aux); one-way clustering uses G/(G−1)·(N−1)/(N−K_aux). The full excluded-instrument Wald statistic uses asymptotic chi-square(q), with no finite-cluster guarantee.

Acceptance: independent base-R comparisons for HC1 and clustered tests, multiple endogenous coefficients, intercept/no-intercept designs and nonzero nulls; retained-sample identity after current-data edits; rank/cluster failures preserve state; CLI/script text and JSON. Existing homoskedastic F output and scalar confidence sets remain unchanged. Robust confidence sets, endogenous-subvector inference, HAC and survey inference are excluded.

Implementation and verification are in progress; this initial commit records the bounded contract.
