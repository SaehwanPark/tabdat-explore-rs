# Household pooled OLS commune bootstrap: WIP contract

**Status: WIP, not yet an executable validated workflow.** This bounded slice
covers the first of three additional cluster bootstrap blocks in the author
`mma24p1olscluster.do`; the author notes these results are not reported in the
book's Table24.4. It does not complete FE/RE bootstrap or survey design.

## Exact outcome and sample

Use original `vietnam_ex1.asc` with the established preparation and verify the
optional separately converted DTA: 5,999 source rows, 5,006 joint-finite outcomes
and regressors, 194 communes of sizes1–39, six numeric slopes `lnhhexp age sex
hhsize farm educ` and intercept. Sort by stable `source_row`; it is not time.
OLS on complete households is the original point fit. Draw194 **whole communes**
with replacement in first-seen order per replication, preserving within-commune
row order and repeated copies, B=500, seed10001. Unequal group sizes imply
variable replicate N. The native xoshiro256++/SplitMix64/rejection stream is
explicit and **not the historical Stata stream**.

Report point estimates, replicate means/bias, full B−1 sample covariance and
standard errors, R type-7 marginal percentile95 intervals; distinguish these from
Stata's normal, bias-corrected and historical-stream bootstrap intervals. Do not
claim 500-rep Monte Carlo precision or few-commune robustness. The bootstrap is a
read-only result, not a replacement of the owned original OLS state/covariance;
default/xb and residual predictions come from that retained OLS model.

## Resource decision

Existing generic cluster bootstrap conservatively limits
`G * largest_selected_group_size * B * K^2 <= 100,000,000`, K including intercept.
The actual original design requires `194*39*500*7^2 = 185,367,000`, so the exact
author-program replication count is rejected before fitting. A real250-rep
probe (`92,683,500` worst-case work) completed in1.57s/72.4MB maximum RSS on the
current development machine; that is feasibility evidence, not a platform claim.

Raise **only the cluster bootstrap** conservative maximum to200,000,000, retaining
checked arithmetic, max G×largest group allocation bound, B2..5000, deterministic
stream, immediate fail-on-any-replicate semantics and unchanged100,000,000 IID
work cap. Benchmark the actual 500-rep workflow after implementation; retain the
change only if measured resource cost is reasonable. Explicitly test both sides
of the new cluster boundary, the unchanged IID boundary and failure-state safety.
This is a bounded numerical work-limit change, not permission for unlimited
replications or broad changes to bootstrap semantics.

## Acceptance and evidence

Risk: **high** (numerical resampling and public resource limit).

1. Script load/validate → ordered complete cases → original OLS → B500 whole-
   commune bootstrap → retained OLS mean/residual → report/export, in terminal and
   CLI-v1 JSON. Source/data and successful/colliding-export ownership are preserved;
   scripts remain nontransactional. Fit/group/missingness errors do not silently
   redraw or discard samples or mutate the prior model.
2. R `lm.fit`, sample covariance and type7 quantiles independently refit all500
   resamples using the separately implemented Python oracle of the pinned Rust RNG;
   compare all7 point estimates, means, bias, SE/intervals, full7×7 covariance,
   500 replicate observation counts, all5,999 source rows and all5,006 retained
   prediction rows. Check raw stream over selected explicit indices as well as
   full output and size sequence. Use fixed absolute1e-9 (p-relative1e-8 where
   relevant); investigate any discrepancy, do not widen tolerances.
3. Small inspectable uneven/singleton-group synthetic fixture exercises the actual
   script, determinism, full covariance/intervals, correct missingness and
   retained original prediction state; test original/resample rank failures,
   wrong/missing keys, work caps, source/file no-clobber and negative exit status.
   Demonstrate auditor rejection of forgeries.
4. Optional DTA precision audit reads DTA directly in R and uses separate CSV in
   native. Compare printed author **original OLS coefficients** within half last
   printed digit; do not claim parity of historical random draws, bootstrap SEs,
   bias-corrected or normal intervals.
5. Explicitly run changed/new focused tests, affected stats/integration and
   reverse-consumer regressions, scoped Clippy/format/diff, and review the final
   diff. Update SPEC, roadmap and coverage only to the independently evidenced
   slice. Report hosted CI and independent code-review status honestly.

No automatic FE/RE/Poisson refit, variance-component resampling, survey weights,
strata, multiway clustering, bootstrap t/normal/BC interval, new RNG interface,
or native DTA importer is included. All remain required where the roadmap calls
for them; no scope exclusion is implied.
