# Chapter 21 pooled OLS whole-person bootstrap — work in progress

## Bounded delivery contract

Reproduce the §21.3 Table 21.2 author-program pooled-OLS panel-bootstrap
analysis on the pinned original `MOM.dat`: 5,320 person-years, 532 people,
1979–1988, outcome lnhr, predictor lnwg and an intercept. Fit OLS and resample
532 complete person blocks with replacement, 500 replications and explicit local
seed 10001. Native xoshiro256++/SplitMix64 draws are deliberately **not** Stata's
historical draws; compare the algorithm and resulting estimates to an independent
reference, not literal historical bootstrap output. Report B-1 replicate
covariance/SE, bias and type-7 percentile95 intervals. The existing command is
read-only, not an automatic covariance replacement for the retained OLS fit.

Acceptance before completion:

1. Executable load/guard → fit → whole-person bootstrap → predict/interpret →
   report/export workflow in text and CLI-v1 JSON. Independent R fits each of the
   same explicitly generated indices and checks original sample/order, full
   covariance, means, bias, SE, intervals and every exported prediction row.
2. A small inspectable independent-reference actual-script fixture checks the
   bootstrap result and sample/block metadata. Verify reproducibility and
   text/JSON output, while retaining the original OLS model for predictions.
3. Missing/invalid keys, rank-degenerate original/replicate fit and an existing
   export fail without overwriting output; document existing prior session effects
   and sample exclusions rather than implying transactional scripts.
4. Update current capability/ledger status and run affected integration tests.

No automatic FE/RE refit, iid-person-year bootstrap, finite-sample coverage or
few-cluster guarantee, survey sampling, Stata RNG/bitwise parity or whole-table
claim. Independence is across persons, with arbitrary within-person dependence;
500 replications have Monte Carlo variability.
