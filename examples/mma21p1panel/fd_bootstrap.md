# Chapter 21 first-difference whole-person bootstrap — work in progress

## Bounded delivery contract

Reproduce the §21.3/Table 21.2 author-program first-difference OLS clustered
bootstrap on pinned `MOM.dat`: 4,788 consecutive annual pairs, 532 people,
1979–1988 source levels, outcome Δlnhr and predictor Δlnwg with an intercept.
Construct differences within person from raw ordered levels before excluding
non-finite endpoints; retain only consecutive years. Resample all retained pairs
of each selected person together, 532 draws per replication, B=500, explicit local
seed 10001. Native xoshiro256++/SplitMix64 draws differ from historical Stata.
Report B-1 covariance/SE/bias and type-7 percentile95 intervals. This is OLS on
fixed annual differences, not an automatic panel-model refit.

Acceptance before completion:

1. Execute load/key guard → raw annual pairs and explicit eligibility counts →
   fit → whole-person bootstrap → predictions/interpretation/export in text and
   CLI-v1 JSON. Independent R constructs original source pairs and refits every
   specified resample; audit exact membership/order, coefficients, full bootstrap
   covariance, intervals and every design/prediction row.
2. Independent small synthetic controls include irregular years, missing level
   values and unbalanced cluster sizes; verify no bridging, sample/exclusion
   counts, variable replicate counts, deterministic stream and text/JSON output.
3. Duplicate/invalid keys, singular original and replicate fits, and no-clobber
   export fail without publishing an output; prior script effects are not rolled
   back. Bootstrap does not replace the retained OLS prediction model.
4. Update capability/coverage status and run affected tests.

No Stata RNG parity, general panel lag declaration, automatic FE/RE bootstrap,
weak-IV inference, survey inference, finite-sample coverage or few-cluster
accuracy guarantee. Independence is across people; 500 draws have Monte Carlo
error.
