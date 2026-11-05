# WIP: joint source-level residual snapshot and summary

## Contract before implementation

Reproduce the Chapter21 author residual program lines77–101: pooled intercept
OLS, within FE and balanced Swamy–Arora RE, same complete MOM sample and one
combined long-form source snapshot with pooled residual, conditional within-FE
residual and RE idiosyncratic e. RE native population residual is u+e: compute
estimation-sample unit predictions using shrink=1−(1−theta)², then e=(y−xb)−u.
The author saves the combined mom3 before reshape; an ordered long snapshot with
unique `(id,year)` and the same source keys is its bounded equivalent, not a
generic DTA save/reshape or retained RE unit-effect API.

Input:3..532 complete people, unique exact keys, ten years1979–1988, finite
bounded lnhr/lnwg, identified within/between designs and positive FE variance;
negative raw RE unit variance is truncated to zero. Original N5320/G532. Report
and export sample-standard-deviation/count/mean/min/max for lnhr, lnwg, pooled,
FE and RE e, overall and by year (55 ordered rows), and all source/prediction
rows. Describe residual definitions and scope; estimates are descriptive, not
a serial-correlation test or causal claim. The last fitted RE model should still
own its native population prediction state after SQL; earlier model snapshots
are not claimed retained after replacement.

Acceptance: (1) all source rows, each pooled/FE/RE prediction and all55 summary
rows independently checked against raw-double R QR/full-dummy QR/plm ranef;
(2) fit/coefficient/covariance/component metadata independently checked against
R with abs1e-9/p-relative1e-8/exact underflow0; (3) historical source-log summary
facts checked separately at their printed precision, investigating float32
rounding before claiming exact parity; (4) interior and theta0 controls, text/JSON
and retained RE state. Failures: missing/duplicate/incomplete/out-of-range keys,
rank/component failures and both export collisions preserve input/owners and
expose any partial first export. No new native API or production Python/R.
Remaining external FD mom binary identity, GLS companion and full book remain open.
