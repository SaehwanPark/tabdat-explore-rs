# Table21.4 within-FE residual correlations — WIP contract

**Exact illustration:** Cameron–Trivedi (2005) §21.3.4 p715/Table21.4,
March2005 author [residual program](residual_inventory.md) lines81–83,95–111,
output lines365–388. Original MOM N5320/G532/T10 1979–1988. Classical native FE
intercept normalization + lnwg slope, retain conditional fitted `xbu` and
idiosyncratic residual y-xbu (author `predict ufe, e`), correlate the residuals
across the same people for every ordered year pair (100 cells); report/export
source/prediction rows and ordered `(year_a,year_b,people,correlation)` matrix.
This is descriptive, not a serial-correlation test/covariance correction.

**Sample and failure semantics:** 3..532 complete ten-year people; exact unique
bounded source_row/id/year, finite model y/x in [-20,20], identified FE slope,
positive finite cross-person residual variance each year and all100 finite year
pairs. No missing/unbalanced sample, weights, extra regressors or imputation.
Validate before first export; two exports do not provide whole-session rollback
and never replace existing targets or input. Preserve FE model for late slope
`lincom` after SQL materialization. Supported source-row ordering and complete
people as in the [pooled matrix](pooled_residual_correlations.md).

**Acceptance before implementation (risk high):**
1. Independent base R full person-dummy weighted-zero-sum QR on exact raw MOM
   checks FE slope/full normalized intercept-slope covariance, FE slope/CI/DF,
   all5,320x13 source/xb/xbu/u/residual cells and100 ordered year-pair cells.
   Absolute numerical1e-9; p-relative1e-8/exact underflow0; exact raw and keys.
2. Compare all45 off-diagonal 4-decimal author-log entries independently; no
   unexplained discrepancy or widened independent-reference tolerance. Confirm
   identity residual=lnhr-xbu and per-person residual sums are zero.
3. Interleaved16-person deterministic control checks full sample, matrix, both
   CLI modes and retained FE inference. Test duplicate/fractional keys, missing/
   incomplete years, rank and constant-year residual failure, both export
   collisions/source preservation and explicit partial output after second collision.
4. Source inventory/SPEC/roadmap/ledger and reproducible R/author evidence remain
   honest: no RE `e`, first-difference/level/individual matrix or chapter closure.

Use existing native FE/SQL/predict/export only, no production API/dependency.
Start focused script tests, then affected root suite, final source review and CI.
If native/source residual semantics conflict, stop and report rather than substitute.
