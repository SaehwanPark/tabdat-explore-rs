# Chapter 23 exponential-mean NLS — WIP contract

## Exact source and scope

Reproduce the author's §23.3/Table 23.1 NLS / independent Gaussian GEE mean
illustration using the pinned `patr7079.asc` (SHA-256
`87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8`): all
346 original-order firms, 1,730 1975–79 observations, including 22 zero-total
firms; outcome PAT and `mu=exp(a+b*LOGR)`, no year effects, offset, weights or
extra regressors. Existing bounded native `nl` accepts the explicit formula
with starts `(2.5,0.5)` and supports owned fit, mean and residual prediction.
Compose its predictions with SQL for the exact author comparison; do not
introduce a general NLS cluster/GEE command or claim state for the composition.

Deliver working and firm-clustered Gauss–Newton/GEE covariance as separate
reported conventions. The author uses `RSS/N * inverse(J'J)` as working GEE
scale and `G/(G−1) * inverse(J'J) sum_i(s_i s_i') inverse(J'J)` for cluster
scores `s_i=sum_t j_it*(y_it−mu_it)`, where `j=(mu,mu*x)`. Report full
covariances, slopes/intercepts, standard errors, z and normal95 intervals;
explicitly identify author GEE working N denominator versus native NLS's
RSS/(N−2) and approximate-t inference. State the conditional-mean and
independent-firm assumptions; neither covariance establishes causal validity or
correctness of the mean. Retain all source rows, predictions and residuals in an
ordered no-clobber export and expose diagnostics in text and CLI-v1 JSON.

## Acceptance and stop conditions

1. Independent base R profiles the exponential amplitude and solves its slope
   score, separately checks an R nonlinear fit, evaluates both full covariance
   matrices and all 1,730 source/design/mean/residual rows, and compares author
   printed coefficients, RSS and both SE conventions. Numerical tolerances must
   reflect solver/root precision, not be broadened to force parity. Original
   float64 versus historical Stata storage/optimizer differences are explicit.
2. Synthetic fixture checks full covariance, independent firm scores, including
   all-zero firms and uneven outcome/score profiles, and validates JSON/text,
   reversed input order, no-clobber and input preservation. Test missing/invalid
   keys and zero/rank-deficient curvature, too few clusters and nonfinite input;
   no derived export on failed fit/covariance and no changed pre-existing file.
3. Derive the two-parameter covariance using a weighted-centered Jacobian and
   firm scores, guard rank/finite arithmetic and invalid/zero covariance; verify
   independently. If extracting coefficients from prediction columns, check the
   recovered log-mean linear relationship against the retained fitted means; do
   not silently substitute it for a failed native fit.
4. Update SPEC, roadmap and the ledger only after executable independent
   evidence; retain all other Chapter 23 estimators/bootstraps/figures as open.

Not a general GEE or native clustered NLS covariance, a Gaussian likelihood,
full NLS bootstrap, inference for arbitrary contrasts/margins, or Chapter 23
completion. The native `nl` fit remains the current model state; SQL summaries
are auxiliary and cannot restore their own inference. GitHub Actions are disabled.
