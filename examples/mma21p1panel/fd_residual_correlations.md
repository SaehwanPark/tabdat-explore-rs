# WIP: annual first-difference series and residual correlations

## Contract before implementation

Reproduce the Chapter21 residual author program lines119–134 from published raw
MOM.dat. Guard a complete balanced set of3..532 people with unique exact keys,
finite bounded log hours/wages and all10 years1979–1988; order within id and
explicitly pair each year1980–1988 with its previous calendar year. Do not bridge
people or missing years. Fit an intercept OLS regression of dlnhr on dlnwg using
all9G joint pairs, report classical N−2 covariance/t inference, predicted
fitted values and residuals, then all three separate9x9 Pearson year-pair
correlation matrices of dlnhr, dlnwg and FD residuals across the same people.
Show descriptive, not serial-correlation-test/covariance-correction, scope;
retain linear-model state across SQL/report/export operations. Export ordered
annual pairs/source keys/predictions and all243 ordered matrix cells.

Acceptance: (1) exact raw/pair sample and independent R QR/`cor` all coefficient,
covariance, inference and4,788 prediction rows plus243 matrix/table cells;
(2) compare historical coefficient/inference and all108 off-diagonal author-log
values at their printed precision, separately from R abs1e-9/p-relative1e-8;
investigate float/storage differences rather than relax R tolerance; (3)
interleaved synthetic controls and text/JSON parity; (4) linear postestimation
state after SQL. Failure: duplicates/incomplete/missing/nonfinite/out-of-range
keys or values, rank-deficient fit and a year/series with cross-person standard
deviation at most1e-12 fail before any export; both output collisions preserve
owners, document partial file state. The fixed resolution guard also rejects
roundoff-scale residuals that otherwise masquerade as identified correlation.
Do not add native API or production R/Python dependency.

The author program reloads an external `mom` whose original binary identity/order
is unavailable. An explicit annual-pair reconstruction from MOM.dat is a
candidate equivalent only for independently checked observed sample, fit and
printed author outputs—not proof of the missing binary's identity or ordering.
If logged values disagree at printed precision, report the discrepancy and do not
claim this target validated until its reason is resolved. Unrelated individual
correlation blocks and complete chapter/book inventory remain open.
