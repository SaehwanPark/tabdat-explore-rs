# Fixed-ML-theta whole-person auxiliary bootstrap — WIP contract

Required P2-ML-BOOT, source297–299 in the [manual inventory](manual_inventory.md).
Risk: high (statistical and retained-state semantics). Original MOM N5320/G532/T10;
finite bounded complete ten-year people and exact original keys, same domain as the
[fixed-ML covariance comparison](manual_ml_covariance.md). Fit native balanced
Gaussian RE-ML; derive and freeze original ML theta and construct quasi y/x once;
fit quasi OLS and resample G complete person blocks B500/seed10001 with generic
native bootstrap. Export the original ordered source/ML component/design/prediction
rows and report full B-1 covariance, standard errors, bias and type7 intervals.
Quasi-intercept is on the (1-theta) population scale. Do not re-estimate components
or group means in draws; do not silently substitute GLS theta or claim historical
Stata RNG parity. Retain original auxiliary OLS prediction state after bootstrap.
No shared Rust/API/dependency change is intended.

Acceptance before implementation:
1. Independent R likelihood/QR profile with direct covariance likelihood check
   validates original ML theta/components and quasi design; independent Python
   native-stream indices + R QR recompute all500 person-block fits, full covariance,
   coefficient summaries/intervals and every ordered source/design/prediction cell.
   Exact raw membership; fixed abs1e-9, p-relative1e-8 where applicable, exact
   underflow0, no unexplained tolerance widening.
2. Independent interleaved dyadic16-person controls cover interior and theta0
   boundary ML fits, compare complete summaries, native indexed draw sequence and
   all prediction rows. Both text/JSON modes; original fit prediction state survives
   bootstrap and SQL/report operations; distinguish fixed-theta from refitted ML.
3. Invalid duplicate/fractional keys, missing/incomplete sample, rank/variance
   failures and export collision fail without overwriting source/existing export;
   deliberately rank-deficient bootstrap draws abort without redraw or export.
   No whole-session rollback claim.
4. Clearly document report schema, seed/stream, intercept scale, inference limits,
   runnable fresh-path reproduction and source inventory/roadmap status. Report
   boundary/independent unit assumptions; no historical SE/parity or full chapter claim.

Inspect the existing fixed-GLS bootstrap and the ML profile/reference and generic
bootstrap stream before implementation. Focused new tests then affected root suite,
final diff/source review and required CI before merge. Implement exactly this plan;
if code conflicts, stop and report rather than improvising. Report files, checks,
deviations and remaining risks.
