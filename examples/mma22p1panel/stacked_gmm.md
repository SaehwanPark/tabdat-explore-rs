# Table 22.2 year-stacked clustered GMM — work in progress

## Bounded delivery contract

Reproduce the author program's year-stacked two-step GMM column, not general
panel GMM. Use the pinned `MOMprecise.dat` source and complete 1979–1988 panels:
4,256 differenced person-years from 532 people, five endogenous differences,
no intercept and the existing 72 year-block instruments. The pilot is no-intercept
2SLS; cluster by person. Use uncentered cluster moments and preserve the base
workflow's distinct fixed-pilot-weight sandwich/objective and author-updated
inverse-information covariance/objective, with no finite-sample correction and
normal/chi-square asymptotics. The historical author program and independent R
reference must be distinguished where their precision differs.

Acceptance before this slice is complete:

1. Execute an ordered text and CLI-v1 JSON load/prepare/fit/predict/export script;
   reproduce every source/design/prediction row, all five coefficients, both full
   covariance matrices, intervals and both objective tests against an independent
   R implementation, with documented, investigated numerical tolerances and
   selected author-log facts. Confirm sample membership and instrument ordering.
2. Expand the existing GMM instrument bound only as needed for 72 instruments,
   while preserving base-instrument and exactly identified behavior; test a
   synthetic many-instrument independent reference and rescaling.
3. Reject singular/ill-conditioned instrument or cluster-moment designs,
   insufficient clusters and out-of-bound input; test retained-state invalidation,
   incomplete/duplicated panel data and no-clobber export failure. Scripts are
   ordered, not transactional.
4. Document design, inference, asymptotic/weak-instrument limitations, actual
   tolerances, executable reproduction and accurate roadmap/ledger status. Run
   affected-crate and interface tests; no kernel-only merge.

Out of scope: intercept/included exogenous regressors, weighted or iterative GMM,
finite-sample adjustments, dynamic-panel estimators, weak-identification robust
inference and general GMM post-estimation. The many-instrument setting may have
poor finite-sample inference; reproduction is not proof of valid instruments.
