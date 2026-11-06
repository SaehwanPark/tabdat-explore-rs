# Household Gaussian exchangeable GEE: implementation contract

**Status: native bounded workflow implemented and independently validated.**
This is the household population-averaged block, not a whole-Table24.4 or
complex-survey completion claim.

## Exact illustration and sample

Cameron and Trivedi (2005), Table24.4 p.851, the population-averaged block of
`mma24p1olscluster.do`: `xtreg LNEXP12M $XLISTLINEAR, pa`. The author identifies
this as Gaussian identity-link, exchangeable-working GEE. Use the existing
publisher `vietnam_ex1.asc` preparation and separately the directly read DTA;
do not treat household `source_row` as time.

Outcome `lnexp12m`; numeric slopes in order `lnhhexp age sex hhsize farm educ`;
common intercept; groups `commune`. Joint-finite outcome/predictor/group selection
retains 5,006 of5,999 source rows and194 communes, with sizes1–39 and one
singleton. No weights, offsets, factors, time requirement or synthetic time index.
Native import remains CSV; optional preparation/reference R is not a production
dependency or native DTA importer.

## Estimation and inference contract

For selected group g, size T_g, residual vector e_g, and N selected cases:

- `phi = sum(e^2)/N`, not RSS/(N-K) or the equal-group average of group variances.
- `rho = sum_g[(sum(e_g))^2-sum(e_g^2)] / (phi*sum_g[T_g*(T_g-1)])`.
  The denominator counts ordered within-group pairs; singletons enter phi and
  the mean equations, but contribute no pairs.
- Working covariance `V_g=phi*((1-rho)*I+rho*11')` must be positive definite:
  `-1/(max(T_g)-1) < rho < 1`. Negative admissible correlation is supported;
  do not replace this fit with a positive-variance random-intercept model.
  A numerically indistinguishable zero residual scale also fails explicitly:
  `sqrt(phi) <= 128*machine_epsilon*max(abs(y))`.
- Iterate from pooled OLS. The native solve will use exchangeable whitening and
  existing QR helpers, not dense T_g-by-T_g matrices. Require relative coefficient
  change at most1e-12 and final inverse-bread-scaled score at most
  `1e-10*(1+abs(coefficient))`, strictly before the100-update boundary.
- With `H=sum_g X_g' V_g^-1 X_g` and `U_g=X_g' V_g^-1 e_g`, report both working
  covariance `H^-1` and uncorrected unit-sandwich covariance
  `H^-1 sum_g(U_g U_g') H^-1`. No N/(N-K), G/(G-1), HC1 or CR1 multiplier.
- Each covariance gets its own full matrix, coefficient SE/normal z/two-sided p,
  normal95 intervals and six-slope Wald chi-square/p. No Student t or residual-DF
  inference, likelihood, LR test or variance-component interpretation is implied.

R `geeM 0.10.1`, `useP=FALSE`, independently checks a second dense-block R solve.
R `gee`'s `v4.4compat=TRUE` uses different **equal-group** moments on unbalanced
panels; the existing balanced Chapter21 composition is not an unbalanced oracle.
Modern `gee` defaults also add moment DF corrections. Neither is silently
substituted. The native numerical audit will use fixed absolute1e-9,
nonzero-p relative1e-8 and exact-zero underflow checks, with exact sample/row keys.
Printed author facts use half the last printed digit, separately on DTA.

## User-visible workflow and boundaries

The installed form is:

```text
xtgee lnexp12m lnhhexp age sex hhsize farm educ, family(gaussian) link(identity) corr(exchangeable) id(commune)
predict gee_mean
predict gee_xb, xb
predict gee_residual, residuals
```

All four model options are explicit and required. Both inference variants are
reported, labeled separately, without an ambiguous covariance default or a
`robust` switch. Unsupported family/link/correlation, weights, offsets, factor,
robust/vce, time and survey options must fail before replacing a retained fit.
Parser acceptance alone is not delivery.

The owned GEE state retains the fitted coefficient/design contract. Identity-link
population means equal xb; default/xb predictions require only current X, not
current IDs or current y. Residuals use current y. Missing/nonfinite required
values produce NULL following existing prediction conventions; unseen IDs do not
prevent a marginal mean. No posterior group effects or re-normalization of a
prediction subset. Existing table/variable ownership and atomic-prediction rules
remain in force. Failed fits preserve the previous model. Export collisions
preserve the existing file and retained successful GEE fit; scripts do not become
transactions.

Bounds: at most40,000 candidate rows,1–32 slopes,10,000 raw groups and
`rows*(slopes+1)^2 <= 20,000,000`; require selected G>K, at least one repeated
observation pair, full column rank, positive residual scale and valid working
correlation. Reject boundary correlation, all-singleton/unidentified designs,
nonfinite numerical results, nonconvergence and resource overruns explicitly;
no clipping, working-independence fallback or relabeling as RE.

## Evidence and limits

The actual [script](household_gee.td) exports all source rows, sample membership,
current-X marginal mean/index and current-y residuals. On both original ASCII and
separately converted DTA it matches198 independent R/geeM fit, full-covariance
and inference quantities, all5,999 source/prediction rows and5,006 selected cases.
A DTA/author-log check matches14 printed coefficient/SE facts plus scale and
slope Wald (16 facts) within half the last printed digit. Text and JSON exports
are byte-identical. The strict auditor rejected15 forged input/result fields;
the author auditor rejected two further forgeries.

The checked-in 48-group synthetic fixtures cover positive (`rho=0.7204`) and
negative (`rho=-0.1377`) correlations, singletons and missing cases. Focused
kernel/integration tests cover full references, ordering/predictor scaling,
non-positive-definite working correlation, singularity, zero scale, convergence,
resource limits, retained-state/ownership, atomic predictions and successful or
colliding actual-script exports. No independent code review or hosted CI is
claimed by this local evidence.

Remaining household RE outputs, other GEE families/correlations, bootstrap,
complex-survey weights/strata and whole-table/book coverage remain required gaps,
not waived by this slice. No generic store/restore, post-estimation contrasts,
prediction uncertainty or new platform/release claim is included.

## Run the independent reference

Install optional R `geeM 0.10.1` (depends on Matrix); DTA additionally uses
`foreign`. Set `R_LIBS` to a non-default validation library if necessary.
Output directories must not already exist; all numerical/package checks precede
creation. The reference writes every source row, keyed predictions, group
scores/influences, and198 scalar/matrix/inference quantities:

```sh
Rscript examples/mma24p1olscluster/household_gee_reference.R vietnam_ex1.asc ref-ascii
Rscript examples/mma24p1olscluster/household_gee_reference.R vietnam_ex1.dta ref-dta --dta
Rscript examples/mma24p1olscluster/household_gee_reference.R vietnam.csv ref-csv --csv
```

Optional `TABDAT_GAUSSIAN_GEE_REFERENCE_ROOT` replays either original reference
in the focused kernel test. To audit an actual script run and the separate
DTA/log comparison, use:

```sh
python3 examples/mma24p1olscluster/check_household_gee.py native-ascii ref-ascii vietnam_ex1.asc
python3 examples/mma24p1olscluster/check_household_gee.py native-dta ref-dta vietnam_ex1.dta --dta
python3 examples/mma24p1olscluster/check_household_gee_author.py vietnam_ex1.dta mma24p1olscluster.txt ref-dta native-dta/result.jsonl
```

The auditor expects `vietnam.csv`, `result.jsonl` and the script's exported CSV
in each native directory. Prepare the DTA CSV with the existing optional
`dta_reference.R`; TabDat itself does not import DTA.
