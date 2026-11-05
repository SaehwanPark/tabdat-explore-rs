# Chapter21 exchangeable-working GEE companion report

## Preimplementation contract (WIP)

P4-EXCH, [source inventory](gls_inventory.md), `mma21p4pangls.do` lines87–94:
Gaussian identity-link `lnhr ~ 1 + lnwg`, the published complete MOM panel,
5,320 person-years/532 people, 1979–1988. The author reports both exchangeable
working and person-semi-robust covariance, normal z/95 and slope Wald chi-square.
Reuse the independently validated [legacy Pearson exchangeable GEE](gaussian_pa.md)
fit and its full uncorrected person-score sandwich, without changing its existing
outputs or retained pooled-OLS model. Add a bounded same-session report using
`G/(G-1)` times that full sandwich, normal tails and slope chi-square, clearly
distinguishing the uncorrected GEE and OLS CR1 corrections. Keep source-keyed
predictions, working-correlation admissibility, score/convergence and state
contracts of the original workflow. Report the two complete population-scale
covariance matrices and both coefficient/normal95 rows; no joint Wald is claimed.

Acceptance and failure cases:

1. A complete same-session load/guard/fit/predict/report/export composition
   works from a fresh directory through text and typed CLI-v1 JSON. The existing
   four Gaussian-PA exports remain byte-identical; new source P4 report/fit
   exports are no-clobber and their partial side effects are documented.
2. Independent R `gee`4.13.30, dense per-person covariance/score matrices and
   base-R normal tails check original full working/adjusted matrices, both
   coefficient rows/tails, all5,320 source/prediction rows and532 unit scores.
   Fixed absolute1e-9, nonzero p relative1e-8 and exact underflow zero apply;
   author-log comparisons use printed precision only, with precision differences
   investigated rather than tolerance widening.
3. Positive and admissible negative exchangeable-correlation controls check all
   values against independent R, confirm the adjusted-to-raw factor is G/(G-1),
   and preserve original retained OLS prediction state.
4. Missing/invalid keys, rank/singular working correlation, exhausted iteration,
   unresolved near-zero person-score variance and each output collision fail
   without clobbering source or owned output. Earlier outputs/state may remain.
   Verify report cannot silently consume an absent prerequisite named fit.

The source's separate B500 exchangeable bootstrap, historical72-of-500 count,
AR1/unstructured working models, saved mixed table and general native GEE state/API
remain open. Independent people and a correct marginal mean are assumed; this is
neither causal inference nor a formal correlation-specification test. No production
R/Python dependency, cross-person covariance or few-cluster guarantee.
