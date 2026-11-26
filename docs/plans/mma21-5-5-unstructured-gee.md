# MMA §21.5.5 unstructured Gaussian GEE

## Outcome

Add `corr(unstructured)` to the bounded Gaussian identity-link `xtgee`
workflow, with required `time(var)` and the original MOM 532-person, 10-year
hours/wages panel. For this product slice, define the working matrix by
standardizing each empirical residual cross-moment with the two corresponding
period-specific residual scales, then fixing its diagonal at1. Report the
working covariance and person-cluster sandwich inference, retain the fit for
current-row `xb` and residual predictions, and compare the full calculation
with a separate Base-R implementation. This estimator is explicitly distinct
from the formula in Stata's current `xtgee` manual and is not claimed to
reproduce the historical Stata output.

Limit this slice to complete balanced panels with 2–10 shared time values,
unique group/time pairs, numeric finite model fields, an intercept and no
weights. Require the estimated unstructured matrix to be positive definite;
fail with a located diagnostic rather than clipping correlations, projecting
the matrix or silently changing the sample. Keep existing exchangeable GEE and
Poisson GEE contracts unchanged.

## Current behavior and evidence

- `xtgee` currently admits Gaussian/identity/exchangeable and
  Poisson/log/exchangeable. `GaussianGeeResult` stores one scalar `rho`, so an
  unstructured fit needs an additive result/state path that preserves the
  existing exchangeable wire contract.
- The official [March 2005 author program](https://cameron.econ.ucdavis.edu/mmabook/mma21p4pangls.do)
  fits `lnhr` on an intercept and `lnwg` with `corr(unstructured)`, `id(id)` and
  `t(year)`, then reports conventional and robust inference. The [author log](https://cameron.econ.ucdavis.edu/mmabook/mma21p4pangls.txt)
  gives N=5,320, G=532, slope 0.0910023, working SE 0.0137712 and robust SE
  0.0501344. Its header labels the table 21.6 while the executable sections
  label it 21.7; preserve that source discrepancy. The bootstrap command is
  commented out, and the final `unstrbootse` listing fails, so this slice makes
  no bootstrap claim.
- The official [Stata GEE manual](https://www.stata.com/manuals/xtxtgee.pdf)
  defines unstructured correlation as a symmetric matrix with unit diagonal,
  estimates its off-diagonal entries from Pearson residual moments, and warns
  that an estimated working matrix may fail positive-definiteness checks.

## Changes

1. Extend the typed GEE command and parser with `time(var)` only for Gaussian
   identity `corr(unstructured)`. Validate unique finite keys and a common
   balanced time grid before fitting; retain source-order sample indices while
   constructing each group in ascending time order.
2. Add a safe-Rust statistics fit that iterates Gaussian GLS coefficients and
   the period-standardized residual-moment correlation to convergence. Use a
   bounded 2–10 period design, positive-definite matrix checks, explicit
   iteration/score certificates, and no eigenvalue clipping or shrinkage. Keep
   the full correlation matrix, scale, sample and both covariance/inference
   results in a distinct result type. Identify the chosen standardization in
   help and machine-readable result metadata.
3. Retain the new fit as its own state/result variant. Add current-row `xb` and
   residual prediction and working/sandwich named-report support without
   changing exchangeable JSON fields or predictions. A failed new fit preserves
   the prior estimate and active data.
4. Add terminal/JSON reporting, help, command discovery and batch/script
   admission. Add a source-derived `unstructured_gee.td` example, independent R
   reference/checker, and synthetic balance, order, rank and non-PD controls.
5. Update the MMA §21.5.5 example documentation, coverage ledger, `SPEC.md`,
   command-result contract and roadmap. Keep the numbering conflict and omitted
   source bootstrap explicit.

## Verification

- Compare the source-derived fit against a separately coded dense Base-R
  implementation of the selected period-standardized formula: sample/design
  identity, coefficients, scale, iterations/score, all working-correlation
  entries, both full covariance matrices, normal inference, current-row
  predictions and source order. This validates the native calculation against
  the stated method; it does not establish parity with Stata's estimator.
- Report historical author-log values alongside native values and differences;
  do not widen tolerances or present the author values as a passing assertion.
  Verify working and sandwich covariance are distinct and that the selected
  panel sandwich has multiplier1.0. The Gaussian working scale uses Pearson
  RSS/(N−K); this product definition does not claim the historical Stata robust
  standard error.
- Exercise balanced T=2 and T=10 synthetic panels, physical row permutation,
  duplicate/missing time keys, incomplete panels, rank failure, singular or
  non-positive-definite working matrices, prediction collisions and failed-fit
  state preservation.
- Run the smallest affected stats, language, runtime, reporting and CLI tests,
  then workspace check, scoped Clippy, formatting and diff checks.

## Estimator choice and validation status

- **Selected by the user on 2026-11-26:** keep the per-period standardized
  residual-covariance method and document its difference from Stata. The native
  command is executable with this explicit product definition; do not describe
  it as Stata-compatible.
- The source-derived MOM run and the dense Base-R implementation of the current
  per-period standardized residual covariance agree across all 100 correlation
  entries, both full coefficient covariance matrices, inference and all 5,320
  source-ordered predictions. This is an implementation cross-check, not
  evidence that Stata or another package uses the same estimator.
- On that same source, the current prototype gives slope 0.0910990841,
  working SE 0.0134302714 and uncorrected panel-sandwich SE 0.0393844960;
  the author log reports 0.0910023, 0.0137712 and 0.0501344. These differences
  are reported as-is; tolerances were not widened.
- The current Stata `xtgee` manual's unstructured moment formula uses shared-
  panel Pearson-residual cross-products with a pooled residual-square
  denominator. On the original MOM residuals, that pooled-denominator matrix is
  non-positive-definite at the initial pooled-OLS coefficient; an exploratory
  direct GLS iteration cycles rather than converging. CRAN `gee` 4.13-30 and
  `geepack` 1.3.13 (R4.6.2) did not provide a converged Stata-formula fit on
  that case. `geepack` does converge on a separate synthetic G=60, T=4 panel,
  but returns its own pooled-denominator alpha and a different fit; that is not
  a reference for the selected product method.
- The official 2005 Stata log remains a separate empirical result and does not
  match this product estimator: the selected method gives slope 0.0910990841,
  working SE 0.0134302714 and uncorrected panel-sandwich SE 0.0393844960,
  versus author values 0.0910023, 0.0137712 and 0.0501344. Keep these differences
  visible in the example, coverage ledger and help. Exact Stata8 replication,
  its robust SE and the source's failed/commented bootstrap remain outstanding.

## Decisions and risks

- This is an unstructured working covariance for a marginal mean, not a random-
  effects model, conditional unit-effect fit, serial-correlation test, or
  guarantee that the selected correlation model is correct.
- Complete balanced samples make every time-pair moment share the same people;
  unbalanced, partially observed, time-varying covariate, Poisson, and other
  GEE families remain unsupported here.
- A non-positive-definite moment estimate is a model-fit failure. Do not silently
  project it to the positive-definite cone; any regularized alternative requires
  a separately specified estimator and reference.
