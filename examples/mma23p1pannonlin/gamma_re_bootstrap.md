# Chapter 23 gamma random-effects Poisson whole-firm bootstrap — WIP contract

## Exact target

Extend the independently validated [gamma-RE fit](gamma_re_poisson.md) with a
whole-firm pairs bootstrap for the §23.3/Table 23.1 patents illustration in the
[author program](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.do)
and [log](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.txt).
The pinned `patr7079.asc` SHA-256 is
`87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8`.
Use all 346 source-record-order firms/1,730 observations in 1975–79, including
22 zero-total firms and missing nonmodel ARDSSIC. Specify the same shared-gamma
firm likelihood with marginal mean `exp(a+b*LOGR)`, no other covariates,
weights, year effects or offsets. This is a bounded composition, not a general
native `xtpoisson` or bootstrap command.

For each of 500 replicates, draw 346 five-row firms with replacement; each
copy receives a **separate shared gamma effect**, so a source firm's likelihood
contribution is multiplied by its draw frequency, not combined into a single
longer panel. Refit `(a,b,ln(alpha))` jointly using the original pooled-Poisson
starting coefficients and ln(alpha)=0, with the original fit's bounded
full-Newton/convergence/information/sensitivity criteria. A failed replicate
aborts without omission, redraw or export. Return original fit, all three
replicate parameter distributions, full B−1 covariance, biases/SEs and type7
95% percentile intervals, plus all ordered draw IDs, RNG steps and parameters.
The fixed per-replicate xoshiro256++/SplitMix64 stream (base seed 10001) is
independent of Stata and of native TabDat's single-stream bootstrap.

The author program prints only coefficient bootstrap SEs and uses
`cluster(id)` without visibly specifying `idcluster()`. The historical
handling of repeated `id` values is unresolved; grouping duplicate copies into
one longer gamma panel is a different likelihood. This declared distinct-copy
analysis does not establish historical ID or RNG parity. Inference assumes
independent firms; it is neither causal identification nor a few-cluster
guarantee. Normal-RE Poisson, Figure 23.1 and remaining book requirements are
separate required work.

## Acceptance and failure cases

1. Independently verify all 173,000 ordered draws/steps with Python and all
   500 three-parameter refits, full covariance and percentile summaries against
   an R implementation of the integrated gamma likelihood with distinct copied
   firm contributions. Validate sample/design and original predictions; compare
   each floating result at justified fixed tolerances, investigate discrepancies
   without widening them, and retain unresolved author-output differences.
2. Exercise a small inspectable fixture, all 6,000 draws and R refits, text and
   JSON reports, reversed input and no-clobber. Include zero-total firms.
3. Verify malformed, missing, duplicated and out-of-range data, singular or
   cancellation-dominated information, dispersion boundary, nonconvergence,
   predictor and iteration/RNG limits and a genuine failed replicate. Test
   input/export and retained pooled-fit/session-state invariants; distinguish
   nontransactional earlier native work from failure of the SQL statement.
4. Update SPEC, roadmap, ledger and related notes only after implementation and
   independent evidence. R and Python are development-only. WIP is not a
   capability or a completed textbook illustration.
