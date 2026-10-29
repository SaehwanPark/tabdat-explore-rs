# Chapter 23 pooled Poisson whole-firm bootstrap — WIP contract

## Exact source and intended bounded workflow

Reproduce the author program's §23.3/Table 23.1 **pooled Poisson firm-pairs
bootstrap method**, not Stata's random stream. Pinned `patr7079.asc` SHA-256
`87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8`;
all 346 source-order firms/1,730 observations 1975–79, including 22 zero-total
firms. Model `mu=exp(a+b*LOGR)`, outcome PAT, with intercept and one slope, no
extra regressors/offset. Each of B=500 replicates draws exactly 346 firms with
replacement, retains all five rows per copy, refits both Poisson coefficients,
and fails the entire workflow on any failed replicate. No independent-row
resampling, arbitrary estimator, general native bootstrap command or retained
bootstrap model state is implied.

Reuse the validated explicit SQL xoshiro256++/SplitMix64 stream family: seed
`10001+r−1` for replicate r, u64 rejection, firms ordered by original ID.
This differs from Stata and native single-stream TabDat bootstrap. Export all
173,000 ordered firm draws and both corresponding replicate coefficients,
report full B−1 two-parameter covariance, means, biases, SEs and type7
percentile95 intervals plus sample/seed/method in text and CLI-v1 JSON. Never
redraw or silently discard failures. Compare with native pooled Poisson fit and
prediction workflow; no new estimation state or analytic-sandwich claim.

The author log reports 1,730 observations/346 clusters, B=500, observed slope
`.6929337`, intercept `1.711528`, bootstrap slope SE `.0473006`, intercept SE
`.141745`, but its RNG and potentially percentile convention differ. Report
numerical differences without claiming historical RNG parity. Independent firms
are assumed; this is not a few-cluster guarantee or evidence of correct mean or
causality.

## Acceptance, references, failure cases and exclusions

1. With independently generated Python oracle draws, R refits all 500 slopes and
   intercepts, and separately checks three frequency-weighted Poisson GLMs.
   Compare all original source/design rows, exact draw IDs/steps, every
   replicate coefficient, full covariance and summaries at documented fixed
   tolerances. Compare native pooled Poisson original fit/predictions and author
   printed facts; use R/Python only as development references.
2. Scalar profile uses `a=log(sum(y)/sum(exp(b*x)))` for each weighted resample
   and monotone slope score; sample validation and exponent/score bounds must be
   explicit. Require a scale-distinct bracketed interior root, finite positive
   information, and converged score for original and all 500 replicates. A
   bounded RNG rejection loop must complete; failure never triggers redraw.
3. Small synthetic fixtures compare every draw, fit, covariance and source
   value against independent reference. Test exact/near endpoint roots, zero
   outcome totals, rank deficiency/separation including a failed replicate,
   nonfinite/invalid inputs, bounded RNG exhaustion, prediction/report/export
   failures, source/order invariants and no-clobber. Evidence must show the
   failure occurs before export, not merely that an existing output blocks it.
4. Update SPEC/roadmap/ledger and run focused and affected-crate tests. Existing
   conditional-FE bootstrap high-bit/forced-rejection oracle does not substitute
   for checking all new pooled draws. State limitations and resource costs.

Out of scope: whole Table 23.1/chapter, RE/GEE/NLS bootstraps, cluster sandwich
correction changes, BCa/normal intervals, historical RNG/float parity, or a
fully general Poisson-bootstrap interface. If the proposed profile or SQL work
cannot meet these bounds and comparisons, investigate and stop rather than
widening tolerances or publishing a partial successful fit.
