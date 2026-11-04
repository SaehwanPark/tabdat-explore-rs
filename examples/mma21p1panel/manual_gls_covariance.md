# Manual fixed-GLS-theta covariance comparison — WIP contract

P2-GLS-IID/CL/HC1, manual source264–269/278–279 in
[manual inventory](manual_inventory.md). Original N5320/G532/T10; bounded
3..532 complete ten-year people, unique exact source_row/id/year keys in original
ranges, finite abs(lnhr/lnwg)<=20, identified within/between wage designs,
positive within error variance and existing rank/work limits. No silent sample
deletion. Raw fields/keys retained. No weights/extra slopes/time effects.

Original balanced Swamy–Arora: within component e=RSS_within/(N-G-1),
raw_u=RSS_between/(G-2)-e/10, u=max(raw_u,0), theta=1-sqrt(e/(e+10u)). Preserve
signed raw_u and valid boundary theta0. Compute once, freeze across all three
auxiliary fits. Native RE population baseline uses normal inference; auxiliary
OLS with intercept on y-theta*ybar/x-theta*xbar uses Student-t and quasi
intercept=(1-theta)*population intercept. Auxiliary IID covariance uses its own
RSS/(N-2), not blindly the within component e. HC1 N/(N-2); person CR1
G/(G-1)*(N-1)/(N-2), inference DF G-1 versus ordinary/HC1 N-2. No exact finite
sample/nominal coverage guarantee when theta is estimated or under arbitrary
within-person dependence. This is not ML information or component refitting.

Acceptance before implementation:
1. Independent R/plm Swamy–Arora plus raw within/between QR and quasi QR/explicit
   score sums check30 native fields and90 auxiliary fields (full matrices,
   both coefficients/SE/t/p/95CI/DF), original components/theta, named provenance
   and23-column raw/population/quasi prediction export. Fixed abs1e-9,
   p-relative1e-8/exact underflow0. Population/quasi coefficient/covariance scaling
   checked separately, not mixed by parameter name.
2. Independent frozen interleaved16-person positive-component and negative-raw-u/
   theta0 controls; positive coefficient score variances, no full-joint inversion
   requirement. Text/JSON prediction bytes match. Both final cluster intervals
   execute after reports/prediction/summarize to check owned covariance/DF.
3. Duplicate/missing/incomplete/fractional keys, unidentified within/between
   designs and nonpositive error-component failures stop before export; existing
   export/source owner preserved. Earlier successful commands are not rolled
   back as a whole session.
4. Auditor verifies exact raw fields/sample indices/rowwise cluster labels,
   normal-versus-t/covariance/intercept labels and final export success; forged
   numerical/provenance/source/partial traces rejected.

Reuse existing SQL/FE/RE/OLS/lincom/report/export primitives. No production
Rust/parser/API/dependency change. Native RE snapshots/full mixed source table,
fixed-ML auxiliary targets and the source GLS-under-ML-HC1 reporting anomaly
remain open; no literal Stata float32 or chapter/book closure claim.
