# Chapter21 per-draw component-refitting RE-GLS bootstrap

## Preimplementation contract — in progress

Target C21-P1-REGLS-BOOT: first author program mma21p1panfeandre.do lines285–287,
Table21.2. Use original MOM.dat SHA256
`2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77`,
5,320 observations/532 people/T10, years1979–1988; lnhr on intercept and lnwg.

Implement B500 whole-person pairs draws using the existing declared
per-replicate native xoshiro256++/SplitMix64/rejection convention (base10001).
Recompute Swamy–Arora components and theta on every draw; negative raw unit
variance is truncated at zero as in native RE-GLS. Copies of a repeated person
remain independent panel copies. Bootstrap the population-scale intercept and
wage slope, not a quasi-transformed intercept. Preserve the original native
RE-GLS fit for subsequent population predictions.

Acceptance:
1. Validate every refit/component/theta against R/plm on independently generated
   draws; audit every draw, full B−1 coefficient covariance/SE/bias/type7
   percentile95 intervals and all original source/prediction rows.
2. Exercise interior and zero-unit-variance controls, repeated copied people,
   interleaved/reversed source order and retained original model predictions.
3. Fail on invalid/incomplete/missing data and any unidentified/invalid replicate,
   without dropping/redrawing; preserve source files and existing output bytes.
4. Match JSON/text CSV bytes, with ordered exports and explicit nontransactional
   script failure semantics.

Exclusions: general bootstrap/xtreg API, unbalanced panels, component-bootstrap
intervals, historical Stata stream/duplicate-ID/all-interval parity, changes to
existing fixed-theta workflow, nominal/few-cluster or causal guarantees. No new
capability/validation is claimed before implementation and checks complete.
