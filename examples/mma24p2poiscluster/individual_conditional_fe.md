# Chapter 24 individual conditional Poisson FE — in progress

Bounded native `xtpoisson y xvars, fe id(group)` plus the §24.7/Table 24.6
individual FE workflow. The pinned source/preparation and ten original numeric
predictors are those in [README](README.md). Of 27,766 source rows, one entirely
missing row is excluded, then commune 64's 94 all-zero observations are excluded,
leaving 27,671 individuals in 193 positive-total communes. Source rows are
cross-sectional people, not temporal repeated measurements.

Contract: conditional multinomial likelihood given each group's selected count
total, no common intercept, inverse conditional information/normal95 inference,
conditional same-total null/LR and joint slope Wald chi-square. Joint finite-row
selection precedes removal of zero-total groups. Exact group keys are structural
and audited even on rows subsequently omitted. Counts on joint-finite cases must
be exact integers 0..100000. Candidate rows<=40000, K1..32, rows*K²<=20million,
candidate groups<=10000 and retained positive-total G>K; rank/finite-information,
strict scaled Newton score/step convergence and line-search guards apply.

Acceptance before merge:
- Independent R conditional likelihood/score/information and dummy-Poisson GLM
  cross-check all original coefficient/full-covariance/inference/likelihood fields,
  exact sample membership and every conditional fitted-mean/residual/index row.
  Absolute tolerance 1e-9, likelihood 1e-8 and nonzero-p relative 1e-8; exact zero
  for underflow. Investigate discrepancies, not unexplained widening.
- Separately compare the pinned DTA/author log's printed FE coefficient/SE facts;
  keep ASCII precision and optimizer differences explicit.
- Synthetic multiple slopes, unbalanced/interleaved groups, positive singleton,
  zero-total group and missing-case control; scaling/within-group shifts must not
  change slope inference or original conditional means.
- Terminal/scripts/CLI-v1 report sample/exclusions, method, normal inference and
  convergence. Owned retained normalizations support current-data `predict ..., n`
  and residuals on fitted positive-total IDs, and slope-only `xb`. No silent
  renormalization after current data changes; unseen/zero-total IDs give NULL means.
- Test invalid counts/keys/rank/separation/iteration failure, retained state,
  prediction atomicity, source preservation and no-clobber export. Failed new fits
  clear stale models without mutating data; syntax errors preserve old state.

Exclusions: RE, robust/cluster/survey covariance, weights/offsets, factors,
general post-estimation/margins, unit-effect uncertainty, automatic bootstrap,
new-group/population forecasts or general Stata parity. The existing scalar
Chapter23 SQL workflow remains intact. Parser/kernel-only WIP is not mergeable.

**Status:** contract only; implementation, executable workflow, references and
verification pending. Individual RE and remaining textbook methods are required.
