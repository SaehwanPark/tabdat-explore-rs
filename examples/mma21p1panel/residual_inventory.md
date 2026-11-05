# Chapter21 residual-analysis source inventory

This is a source-scoped inventory of the March2005 author program
[`mma21p3panresiduals.do`](https://cameron.econ.ucdavis.edu/mmabook/mma21p3panresiduals.do)
(SHA256 `6c4ccfa560eb8f818412e87895e4c205f51c1e00865346f3b0c532902b7a43e8`)
and its [author output](https://cameron.econ.ucdavis.edu/mmabook/mma21p3panresiduals.txt)
(SHA256 `925aff11176cb78bbacf71abc69c2bef4876bdcc0e5544c001c2df4f5f8f56b5`).
They target §21.3.4 pp713–715, Table21.3 pooled residual and Table21.4 within
residual correlations. Original `MOM.dat` is532 people x10 years1979–1988,
8 columns `lnhr lnwg kids ageh agesq disab id year` in person-year order;
[raw-source preparation](README.md) pins its hash. No author data/program is
redistributed here. Source lines below are 1-indexed; publisher-log text represents
the historical run, not proof of the Rust behavior or exact float64 parity.

The source's early comments call for one regressor and exactly10 years. Its long
matrix blocks reshape by id/year and run complete-case `corr` across people, while
its final two examples correlate one person's series with its annual lag across
nine time pairs. Those are different sample axes. Report correlations as
descriptive quantities, not automatic model specification tests or cluster-robust
inference. Remaining entries are required, not implied by an existing estimator.

| ID / source lines | Sample, preparation and requested result | Current status / evidence and outstanding work |
| --- | --- | --- |
| P3-LOAD / 65–71 | Load/summarize raw MOM, preserve original id/year/person order | Bounded converter/scripts validate exact source and keys; native DTA IO is not implied. |
| P3-POLS / 77–79 | N5320 intercept+lnwg pooled OLS, classical inference and level residual per source row | [Bounded workflow](pooled_residual_correlations.md) validates QR fit/covariance/late CI and all5,320 residual/prediction rows. |
| P3-FE / 81–83 | Same sample, within FE and `predict ..., e` | [Bounded workflow](within_residual_correlations.md) validates native conditional FE residuals, all4 prediction modes, slope/full normalized covariance and late CI against full-dummy QR. |
| P3-RE / 85–87 | Same sample, RE-GLS and author `predict ..., e` | [Bounded composition](random_effects_residual_correlations.md) validates native population fit and original-sample shrinkage unit/e predictions against plm `ranef`; native population residual alone is not author `e`. No general retained RE unit-effect API. |
| P3-RESID-SAVE / 89–101 | Summarize all three residuals; save `mom3`, keep/reshape by id/year to532 wide people | [Bounded combined long snapshot and report](residual_summary.md) validates64 fit fields, all5,320x30 source/prediction cells and55 summaries against R/plm; source keys and unique id/year establish same-sample reshape equivalence, not DTA/binary identity. A separate float32 hypothesis matches53 counts/211 of212 printed numeric facts; the overall RE mean differs by8.38e-14 and exact last-digit parity remains unresolved. |
| P3-LEVEL-CORR / 105–106 | Correlate lnhr and lnwg across people, each a10x10 year matrix | [Validated bounded source workflow](level_correlations.md): exact raw rows, all200 R/table cells and90 printed author-log off-diagonals. |
| P3-POLS-CORR / 107–108 | Table21.3 pooled residual10x10 across people,532 complete pairs each | [Validated bounded workflow](pooled_residual_correlations.md): independent R QR/100 cells/all source predictions,45 author-log off-diagonals. |
| P3-RE-CORR / 109 | RE `ure`10x10 across people | [Validated bounded composition](random_effects_residual_correlations.md): plm `ranef`/100 cells/all source predictions and45 author-log off-diagonals. |
| P3-FE-CORR / 110–111 | Table21.4 within residual10x10 across people | [Validated bounded workflow](within_residual_correlations.md): independent full-dummy QR/100 cells/all source predictions and45 author-log off-diagonals. |
| P3-CROSS-1979 / 114 | Regress 1979 lnhr on lnwg with intercept/classical OLS | [Validated bounded workflow](cross_section_1979.md) checks29 R QR fit/test/lincom fields, all532 source/prediction rows and12 author printed fit facts under a separate plausible float32 storage path. |
| P3-FD-PREP / 119–125 | Reload `mom`, subtract prior physical row then drop year1979; intercept OLS on4,788 differences and residuals | [Bounded annual-pair reconstruction](fd_residual_correlations.md) from MOM.dat checks4,788x17 source/prediction cells and25 R QR fields with no cross-person leakage; 12 author fit facts agree under a separate plausible float32 storage path. External `mom.dta` binary identity/order remains unknown; equivalence is limited to checked outputs, not an identity claim. |
| P3-FD-CORR / 127–134 | Reshape by id; three9x9 correlations of dlnhr, dlnwg and ufdiff across532 people | [Bounded annual-pair workflow](fd_residual_correlations.md) validates243 R/table cells and108 printed author off-diagonals; requires resolved variation (population SD>1e-12). External reload identity remains unproved. |
| P3-INDIVIDUAL / 142–156 | Reload `mom3`, reshape lnhr/lnwg by year with id columns, `tsset year`, `corr lnhr1 L.lnhr1` and analogous person2 | [Validated bounded workflow](individual_autocorrelation.md) checks all4,788 source/annual pairs,532 within-person R correlations and both logged examples; nine adjacent time pairs per person, not532 people. The program's reshape comment incorrectly says year-suffixed rather than id-suffixed; executed reshape is authoritative. |

This inventory does not establish the full chapter or full textbook illustration
inventory. Remaining Chapter21 manual mixed reports, companion GLS program and
other book illustrations retain their own outstanding requirements.
