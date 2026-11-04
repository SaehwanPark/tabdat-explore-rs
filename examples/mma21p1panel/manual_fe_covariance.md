# Manual FE auxiliary covariance comparison

## Preimplementation contract — WIP

Target P2-FE-IID/CL/HC1 in [manual inventory](manual_inventory.md), source
lines240–245/254–255. Original MOM5,320 observations/532 complete T10 people,
lnhr/lnwg deviations from joint-sample person means; intercept plus one slope.
Compare native centered OLS with RSS/(N-2), row-HC1 N/(N-2), and person-score
CR1 G/(G-1)*(N-1)/(N-2). OLS/HC1 slope t95 uses5318 DF, cluster uses531.
Canonical FE baseline has4787 DF and is not substituted for these auxiliary rows.
Ordinary/HC1 rows are working comparisons, not valid absorbed-FE covariance by
assertion; clustered reference assumes independent people/adequate asymptotics.

The centered auxiliary intercept is mathematically zero, not normalized level a.
Generic fits/named reports may display floating-point intercept t/p artifacts;
print explicit interpretation notes in text/structured output, validate only its
point/SE/covariance and numerical-zero bounds, and never claim those t/p artifacts
are meaningful inference. Report slope lincom95 for all three modes; no full
joint restriction requiring invertible clustered covariance. No data-driven
variance clipping or forced nonzero SE. Keep original raw fields/keys and final
cluster OLS point predictions. Named reports are three auxiliary OLS models, not
a claim to support canonical FE snapshots/full mixed Table21.2 comparisons.

Acceptance:
1. Independent original raw means/full QR: all three full covariance matrices,
   points/SEs/slope t/p/95CI/DF and named-report provenance; independent dummy QR
   validates canonical baseline slope/SE/t/p/sample/DF. Fixed abs1e-9, p-relative
   1e-8; clustered intercept t/p excluded analytically, not tolerance-widened.
2. Interleaved16-person independent frozen reference and byte-identical text/JSON
   prediction exports; raw source fields exact, derived means/deviations/fitted/
   residual cells abs1e-9. Primitive/registry state unchanged by reads/prediction.
3. Duplicate/missing/incomplete/bad-key or unidentified within designs fail before
   exports; collision preserves its owner/source. Valid near-zero score variance
   is not turned into a fit-identification failure. No whole-script rollback.

Scope:3..532 complete T10 people, exact bounded keys/finite abs(model)<=20,
existing rank/work limits, no weights/time effects/extra regressors. Reuse existing
SQL/OLS/FE/lincom/named-report/export primitives; no parser/kernel/API/dependency
change. No fixed-ML auxiliary work, historical float32/printed-SE parity,
centered-intercept inference, general panel diagnostics or chapter/book closure.
