# Household Gaussian exchangeable GEE: implementation contract

**Status: WIP; no native GEE command or retained GEE model is installed yet.**
The optional independent R reference is executable. This document defines the
bounded next slice; it is not a capability or whole-Table24.4 completion claim.

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

## Planned user-visible workflow and boundaries

Install only:

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

## Acceptance and implementation order

Risk: **high** (new estimator, inference and owned-state/public result boundaries).

1. Add the pure Gaussian GEE kernel/result/options under `tabdat-stats`, reusing
   `problem.rs`, `sample.rs` and `matrix.rs`; check a small inspectable unbalanced
   independent fixture, including a singleton, missingness, both matrices and
   inference. Add negative-interior correlation and row/order/scaling controls;
   identification, invalid correlation/scale, convergence and bounds must fail.
2. Add the domain command/result/state/family and execution adapter, parser,
   retained predictions, terminal/JSON and discovery/help. Follow the existing
   grouped-estimator boundary rather than introducing a crate or dependency.
   Integration cases cover parsing/rejection, current-X predictions without ID/y,
   current-y residuals, fit/prediction failures and unchanged source/owner/model.
3. Add the actual script, strict native/reference and printed-DTA auditors. Replay
   ASCII and DTA independently: all5,999 source rows, sample membership, full
   matrices/inference, all prediction rows and text/JSON exports. Exercise the
   actual script's successful and colliding exports; mutate auditor inputs to
   prove they reject incorrect results. Document any genuine author discrepancy.
4. Run explicitly named new focused tests, affected-crate/reverse-consumer tests,
   relevant integrations and scoped Clippy/format/diff checks. Update README,
   SPEC/architecture/roadmap/coverage only to the behavior actually delivered.
   Review the final diff and evidence before merging and safely cleaning branches.

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

No native implementation, native numerical audit, regression test run or hosted
CI evidence is claimed by this reference-only checkpoint.
