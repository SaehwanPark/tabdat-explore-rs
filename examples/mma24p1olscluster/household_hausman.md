# Household classical joint FE–RE Hausman comparison

## Bounded preimplementation contract

Deliver H-HAUSMAN in the [source inventory](inventory.md), original May2005
household program lines123–127. Use the original seven-parameter numeric design
and5,006 complete cases/194 communes; no factors, weights, causal identification
or survey design. Source-row is an ordering key, not time. Compare all six
commune-FE slopes against the opt-in equal-group-between RE-GLS slopes, excluding
the common intercept. Retain each fit's **separate native covariance scale**:
FE within RSS/(N−G−6), RE quasi RSS/(N−7). No common-scale substitution.

For d=b_FE−b_RE and D=V_FE−V_RE report d'D^−1d and asymptotic chi-square6
upper-tail probability, both coefficient vectors, both full slope covariances
and their difference, counts/DF and assumptions. Require a safely positive-
definite covariance difference after scaling by parent covariance diagonals;
reject rank deficiency, indefiniteness, cancellation-dominated differences and
nonfinite statistics. No pseudoinverse, ridge, absolute value or silent dropped
restrictions. A failure leaves data and retained model unchanged. A non-rejection
does not establish exogeneity; validity requires RE efficiency/classical IID
idiosyncratic errors under the null, not cluster-robust inference.

Extend `estat hausman` only after model-based equal-group-between RE-GLS fits,
1..16 numeric slopes/intercept, N<=40,000 and N*K²<=20million with K slopes.
Balanced/unbalanced groups and singletons follow the installed estimator sample.
Retain the original owned design and refit/compare the stored RE result before
post-estimation, independent of current data. Existing balanced one-slope
Swamy–Arora scalar behavior/result wire type stays unchanged. ML, clustered RE,
general named-fit comparisons and unbalanced/multivariate default Swamy–Arora
remain unsupported. The new joint output gets a separate typed JSON result.

Acceptance cases:

1. ASCII/precise-DTA original cohort, both full slope covariances, statistic/p,
   all original source and RE prediction rows agree with independent R QR.
   DTA separate-scale probe17.8864248463 / p.00652238245 matches author17.89/.0065;
   common FE/RE scaling gives different results and is not the contract.
2. Balanced and unbalanced/singleton multi-slope controls match independent R;
   predictor permutation/rescaling leaves the joint statistic invariant.
3. Negative/indefinite, singular and cancellation-dominated covariance
   differences fail rather than producing a generalized or positive statistic.
4. Retained-sample semantics survive current-data edits; mutated/legacy missing
   owned designs and mismatched retained fits fail atomically. Predictions and
   reporting preserve the original RE model; export collisions preserve owners.
5. Existing scalar wire/numerical contracts and unsupported variants regress
   unchanged; new JSON/text/script diagnostics expose assumptions and limits.

Implementation/reference/integration evidence is in progress. Independent
R/Python are optional development tools, not runtime dependencies. This bounded
comparison is not whole-Table24.4 or whole-chapter completion.
