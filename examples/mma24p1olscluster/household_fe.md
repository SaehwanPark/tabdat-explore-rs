# Chapter 24 household fixed-effects OLS — in progress

Bounded target: Cameron–Trivedi (2005) §24.7/Table 24.4 household
commune-fixed-effects columns 7–8, from the pinned `mma24p1olscluster.do`
program and `vietnam_ex1.asc` source described in [README](README.md). The
outcome and six numeric predictors are identical to the pooled comparison; use
5,006 jointly complete households, 194 communes (including singleton communes)
and the author's classical commune-FE slope/common-intercept covariance.
This is unweighted, not a complex-survey design or causal identification, and
does not complete random effects, GEE, bootstrap or the chapter.

The source has cross-sectional households, not time series. Use the synthetic
`source_row` **solely as the unique ordering/key required by `xtreg`**, never
as time, lag, duration or serial dependence. Fixed effects absorb commune
intercepts; the common intercept uses the observation-weighted zero-sum
normalization of unit effects. State this normalization and source/key meaning.

Acceptance contract: prepare the pinned source with existing converter; audit
all source/cohort rows and native FE point estimates, full normalized-intercept
and slope covariance, t/F inference and xb/xbu/unit-effect/residual predictions
against independent full-dummy R QR on the same ASCII data. Investigate author
DTA/log precision separately. Test nontrivial small unbalanced groups with a
singleton, missing outcome/zero outcome, duplicate/invalid keys, unidentified
within design and output collision; retain and identify FE state. Text/JSON
outputs must agree. No tolerance widening or full author/chapter parity claim.

**Status:** contract only; implementation and verification remain.
