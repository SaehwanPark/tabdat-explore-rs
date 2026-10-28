# Chapter 21 classical FE–RE Hausman comparison — work in progress

## Bounded contract

Implement `estat hausman` after a successful **model-based**, balanced
Swamy–Arora RE-GLS fit with explicit id/time, one slope and an owned joint-finite
estimation sample. Refit the homoskedastic within-FE slope on exactly that
retained sample, without reading current data or mutating the retained RE fit.
For slope difference d=b_FE−b_RE and model-based covariance difference
v=V_FE−V_RE, report d²/v and chi-square(1) upper tail only when v is finite,
positive and numerically distinguishable from cancellation. No pseudoinverse,
ridge, absolute value, or substitution of clustered covariances. Reject ML,
clustered/robust RE, unbalanced data, multiple slopes, absent retained design,
undefined FE or nonpositive/nearly-zero covariance difference; preserve model
and current data on diagnostic failure. Old serialized RE states lacking a new
owned design must remain readable and fail this diagnostic explicitly.

Illustration: §21.4.3, original 5,320-row/532-person MOM wage panel from
`mma21p1panfeandre.do`, outcome lnhr, lnwg slope and intercept. This is the
conventional covariance-difference comparison under RE efficiency, not the
already implemented cluster-robust auxiliary Hausman comparison. The author
warns the test is invalid under remaining within-person correlation.

Before completion:

1. Text, script and CLI-v1 JSON expose FE/RE estimates, both variances, difference,
   statistic, df, tail, sample/group provenance and assumptions. Reproduce
   original model/prediction/export and result against independent R/plm with
   exact sample, tolerances and selected historical-log facts.
2. Independent synthetic balanced cases check both fits and diagnostic; negative,
   zero or degenerate covariance difference fails, including boundary-component
   cases. Verify post-estimation uses owned sample after data changes; failed
   diagnostics preserve prior fit/data and old wire state remains compatible.
3. Runtime, parser/JSON admission, text, script, rank/covariance failure and
   affected reverse-consumer tests pass. Update SPEC, roadmap and ledger.

Exclude multivariate/rank-reduced comparisons, covariance rescaling variants,
clustered robust comparisons, unbalanced/ML RE, named model pairs and arbitrary
sample/design mismatches. No automatic model selection or claim that failure to
reject establishes exogeneity or that classical inference is cluster robust.
