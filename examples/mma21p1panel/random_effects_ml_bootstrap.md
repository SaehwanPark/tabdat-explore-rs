# Chapter21 RE-ML whole-person bootstrap

## Preimplementation contract — in progress

Target: first author program `mma21p1panfeandre.do` lines301–303, inventoried as
C21-P1-REML-BOOT. Original MOM.dat (SHA256
`2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77`),
5,320 rows/532 people/ten years1979–1988; lnhr on intercept and lnwg.

Implement a bounded native SQL composition with B500 whole-person pairs draws,
base seed10001, existing per-replicate xoshiro256++/SplitMix64/rejection convention.
Refit marginal Gaussian ML, including both variance components, on every draw;
frequency weights represent independent copied people, not a merged longer
panel. Retain original native RE-ML state for population prediction. Report
original and replicate coefficients/components, likelihood/profile certificates,
ordered draws, B−1 full coefficient covariance/SEs/bias and type7 percentile95.

Acceptance:
1. Independently validate every original-source draw and ML refit against Python
   unsigned-integer RNG and R direct marginal-likelihood profiling; check full
   summary covariance/intervals and all original predictions/source values.
2. Match JSON/text CSV bytes and keep original native ML prediction state.
3. Cover balanced synthetic interior and boundary data, repeated copied people,
   ordering and malformed/key/incomplete-panel failures.
4. Fail on any unresolved/nonfinite/rank-deficient replication without silent
   dropping/redrawing; enforce bounded profile search and explicit output
   no-clobber/ordered-script failure semantics.

Exclusions: general bootstrap/xtreg API, unbalanced panels, historical Stata RNG
or duplicate-ID/interval parity, variance-component uncertainty from bootstrap,
RE-GLS resampling, causal interpretation, nominal/few-cluster coverage claims.
Native profile candidates use eta=log(1+10*u/e) grid0:.5:24, polished
positive-to-negative score crossings plus the zero-u boundary. This is a bounded
regular-profile selection, not proof of global optimality between grid points.
No capability or numerical validation is claimed until implementation and checks
are complete. Source data are external and are not redistributed.
