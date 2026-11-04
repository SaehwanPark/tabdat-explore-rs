# Chapter21 FE normalized-level-intercept bootstrap

## Preimplementation contract — in progress

Target C21-P1-FE-BOOT: author first program lines211–213/Table21.2,
MOM.dat SHA256 `2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77`.
Original5,320 rows/532 people/T10/year1979–1988, unweighted lnhr on lnwg plus
absorbed person intercepts. Native observation-weighted normalization is
common_intercept=mean(y)-slope*mean(x), equivalent to mean of person intercepts
on this balanced panel. Recompute the sample means/normalization every draw.
The existing centered auxiliary-OLS intercept is a different target and stays
unchanged; this does not add classical analytic common-intercept inference.

B500, whole-person pairs with independent ten-row copied people, per-replicate
xoshiro256++/SplitMix64 seed10001+r-1 and fixed-u64 rejection (existing bounded
composition convention, not historical Stata RNG replay). Refit within slope and
recover normalized level intercept per draw; report full B-1 covariance/SE/bias
and type7 percentile95, preserving original native FE xb/xbu/u/residual state.

Acceptance:
1. Audit every original-data draw and refit, summary, normalization and original
   source/prediction against independent copied-raw-panel R QR; baseline dense
   dummy QR separately verifies the level-intercept constraint.
2. Two generated controls exercise differing unit composition and normalization,
   full frozen refits/covariance/intervals, typed/readable reports and JSON/text
   equality; zero bootstrap variance is not by itself an invalid point fit.
3. Empty/missing/duplicate/incomplete/invalid panels, rank failure in a draw after
   an identified original, retained FE state, ordering and all export collisions
   are tested. Invalid draws abort, with no drop/redraw or partial summary.

Bounds: 3..532 complete ten-year people, exact finite bounded source/person/year
keys, finite model abs values<=20, within sum of squares>1e-12, final coefficient
abs values<=100. Per-draw residual DF=N-G-1 counts independent copied people;
within RSS may be zero (point fits do not require refit standard errors).
No weights/unbalanced panels, general bootstrap API, unit-effect intervals,
classical analytic common-intercept covariance, historical RNG/full-output parity,
whole-script transaction or finite/few-cluster/causal inference guarantee.
No implementation/capability is claimed until independent checks pass.
