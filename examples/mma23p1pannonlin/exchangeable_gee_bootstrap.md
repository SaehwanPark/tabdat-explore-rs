# Chapter 23 exchangeable Poisson GEE whole-firm bootstrap — WIP contract

## Exact bounded target

Extend the independently validated [exchangeable GEE fit](exchangeable_gee.md)
with a whole-firm pairs bootstrap for the §23.3/Table 23.1 patents illustration
in the [author program](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.do)
and [log](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.txt).
This is a bounded composition, not a general `xtgee` or bootstrap command.
Pinned `patr7079.asc` SHA-256:
`87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8`.
Use all 346 source-order firms/1,730 complete 1975–79 records including 22
zero-patent firms and missing nonmodel ARDSSIC metadata. Fit
`mu_it=exp(a+b*LOGR_it)` for PAT with no extra regressors, time effects,
weights or offsets; legacy exchangeable Pearson moment and fixed Poisson scale
1, exactly as specified in the linked GEE workflow.

For each of 500 replicates draw 346 whole firms with replacement and treat
each copied five-row firm as a **distinct cluster** in all GEE moment and score
calculations; frequency weights are equivalent only when each copy contributes
its original five-row score/moments separately. Start each fit at the original
native pooled-Poisson coefficients, recompute rho at every full Fisher-scoring
step, and validate finite input, positive-definite correlation, bread,
convergence, score and numerical bounds. Do not silently clamp rho, regularize,
drop/redraw a failed replicate, or publish a failed iterate.

Use the established per-replicate xoshiro256++/SplitMix64 stream seeded
`10001+r−1` with unbiased u64 rejection in source-ID order; this differs from
Stata and native TabDat's single stream. Report original coefficients, 500
replicate means/biases, full B−1 covariance and SEs, and type7 percentile 95%
intervals. Export ordered draw ID/step and both coefficients. This bootstrap
assumes independent firms, not causal identification, a correct mean or
few-cluster reliability; no p-values, BCa intervals or restorable bootstrap
fit are claimed. The preceding native Poisson state and predictions may survive
a later script failure; document and test actual state and no-clobber semantics.

## Historical ambiguity — not silently resolved

The author program bootstraps `xtpoisson ..., pa i(id)` with `cluster(id)` but
does not visibly specify `idcluster()`. Current [Stata guidance](https://www.stata.com/support/faqs/statistics/bootstrap-with-panel-data/)
says repeated panel draws should be assigned distinct IDs. The precise handling
of repeated IDs by that historical Stata version/program is not established;
in particular, treating duplicate copies as a single longer exchangeable panel
can change GEE equations. This workflow explicitly uses distinct five-row
copies as the well-defined cluster-pairs bootstrap. Different RNG streams and
unresolved historical ID behavior both preclude exact historical bootstrap
parity; do not silently count that ambiguity as resolved.

## Acceptance and failure cases

1. Validate all 173,000 draw IDs/steps against independent Python; compare all
   500 intercept/slope GEE refits, full bootstrap covariance and summaries
   against R `gee` 4.13-30 with legacy (`v4.4compat=TRUE`) moment convention,
   fixed scale 1, original pooled start and distinct copied-cluster IDs. Check
   original sample/design, GEE coefficients/rho and original prediction rows
   independently. Investigate differences rather than widen tolerances.
2. Check a 12-firm inspectable fixture, all draws/refits, source order,
   text/JSON and no-clobber; include negative rho if feasible.
3. Test invalid, missing, duplicate, nonfinite and singular data; zero Pearson
   variance, exact/near correlation boundaries, cancellation-dominated bread,
   invalid predictor, nonconvergence, RNG/iteration bounds and an actual failed
   replicate after a valid original fit. Verify session/data/file invariants.
4. Update SPEC/roadmap/ledger and prior workflow notes with evidence, retaining
   historical-ID uncertainty and all other required textbook gaps. Run focused
   and affected tests and review before any merge. R/Python and author data are
   development-only.

RE Poisson, RE/bootstrap and the remaining book inventory remain required.
