# Table 22.2 year-stacked clustered GMM

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

Validation contract:

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
   tolerances, executable reproduction and accurate roadmap/ledger status.

Out of scope: intercept/included exogenous regressors, weighted or iterative GMM,
finite-sample adjustments, dynamic-panel estimators, weak-identification robust
inference and general GMM post-estimation. The many-instrument setting may have
poor finite-sample inference; reproduction is not proof of valid instruments.

## Reference and reproduction

After creating `MOMprecise.csv`, the base `reference` and `stacked-reference`
directories as described in the [example README](README.md), run in a fresh
directory:

```sh
/path/to/tabdat-explore-rs --json -f /path/to/repo/examples/mma22p1panel/stacked_gmm.td > stacked-gmm-result.jsonl
Rscript /path/to/repo/examples/mma22p1panel/stacked_gmm_reference.R stacked-reference/design.csv stacked-gmm-reference
python3 /path/to/repo/examples/mma22p1panel/check_stacked_gmm.py /path/to/MOMprecise.dat stacked-gmm-result.jsonl stacked_gmm_design.csv stacked-gmm-reference
```

Independent R 4.6.1 yields wage coefficient 0.3299975062, fixed-weight SE
0.1157475977, author-updated SE 0.1096508142, fixed J 67.54915743 and
updated J 69.50624256 (67 degrees of freedom). The original Stata log reports
0.32999732, 0.10965082 and 69.506226 for the coefficient, updated SE and J;
the source uses historical float storage, and digit-level parity is not claimed.
The checker uses absolute 1e-6 for the selected coefficient/SE and 2e-5 for
updated J against the historical log, rather than conflating it with the
independent double-precision reference.

The R audit compares every design/prediction row and both covariance matrices,
coefficients, normal intervals/p-values and chi-square objectives/p-values at the
same `1e-9 + 1e-9*abs(reference)` gate as the base workflow. Instrument and
sample ordering are checked. The pilot and updated cluster-moment correlation
matrices have R 2-norm condition numbers 6.58e7 and 7.70e7. The observed native
minus R J differences are +1.39e-8 and -1.91e-8 on values near 68–70; the wage
coefficient difference is -1.62e-10. These are below the unchanged relative
gate. R uses direct solves and native evaluation uses an explicit inverse on an
ill-conditioned moment system; their exact contributions to the discrepancy have
not been isolated. No exact historical equivalence is implied. A separate deterministic
20-instrument, 64-cluster synthetic fixture compares both coefficients, full
covariances and objectives against R and checks instrument rescaling. Source-script
controls test reversed input order, duplicate/missing years, insufficient clusters
and an existing export. Inputs and retained GMM state follow the base workflow's
rules; prior script effects are not rolled back on failure.
