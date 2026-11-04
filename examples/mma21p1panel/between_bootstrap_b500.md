# First-program B500 Between pairs bootstrap

## Preimplementation contract — WIP

Target C21-P1-BE-BOOT, mma21p1panfeandre.do lines183–185: original MOM5,320
rows/532 complete ten-year people, lnhr on intercept/lnwg Between regression.
Create one joint-finite mean record per person in ascending numeric ID order;
resample those records with replacement, equal copy weights, B500/seed10001.
On complete panels this is whole-person pairs resampling and re-estimating the
Between regression: copying all ten observations preserves that person's means.
Use the existing native IID bootstrap continuing xoshiro256++/SplitMix64/u64
rejection stream, not the different per-rep cluster stream or Stata RNG.

The original `between_bootstrap.td` B200 manual-program workflow stays unchanged.
A separate B500 script/output preserves reproducibility and permits both examples
in the same directory. Reuse the existing independent R reference/auditor with
explicit B500 options; default B200 validation remains unchanged. No production
Rust/parser/kernel/schema/dependency change.

Acceptance:
1. Exact original source: independent Python draw oracle +500 R full QR refits;
   all18 bootstrap point/mean/bias/B-1 covariance/SE/type7 percentile95/size fields
   and532 ordered person mean/design/prediction rows match at fixed abs1e-9.
   Verify all5,320 raw input rows against the source hash and exact keys, not N
   alone. Do not claim native per-draw serialization: this API reports summaries.
2. A complete16-person/interleaved control gets independently frozen R outputs;
   JSON/text prediction bytes match. Original OLS Between point state survives
   bootstrap, and bootstrap uncertainty is not classical coefficient t inference.
3. Raw duplicate/missing/incomplete panels fail before fit/export; a reproducible
   unidentified bootstrap draw aborts without deleting/redrawing it. Existing
   export owners and source input files are preserved. No whole-script rollback.
4. Existing B200 reference/auditor default and script/output remain valid, using
   their original documented source target. New/modified tests are run explicitly,
   focused first; reuse unaffected current passing primitive evidence.

Bounds:3..532 complete T10 people, finite abs(log model fields)<=20 and exact
bounded MOM keys. Native bootstrap's existing identification/work limits apply.
No historical Stata RNG/repeated-ID/SE or all-interval replay claim, automatic
native between estimator, generic resampling API, full table/chapter/book closure,
individual FE-effect uncertainty or release/platform certification.
