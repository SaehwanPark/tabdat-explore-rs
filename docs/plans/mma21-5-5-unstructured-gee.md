# MMA §21.5.5 unstructured Gaussian GEE

## Outcome

Add `corr(unstructured)` to the bounded Gaussian identity-link `xtgee`
workflow, with required `time(var)` and the original MOM 532-person, 10-year
hours/wages panel. Fit the marginal mean and estimate a full symmetric working
correlation matrix with unit diagonal. Report the working covariance and
person-cluster sandwich inference, retain the fit for current-row `xb` and
residual predictions, and validate the complete result against an independent R
reference.

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
2. Add a safe-Rust statistics fit that iterates Gaussian GLS coefficients,
   Pearson-moment unstructured correlations and scale to convergence. Use a
   bounded 2–10 period design, positive-definite matrix checks, explicit
   iteration/score certificates, and no eigenvalue clipping or shrinkage. Keep
   the full correlation matrix, scale, sample and both covariance/inference
   results in a distinct result type.
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

- Compare the source-derived fit against R 4.6.2 `gee` and a separately coded
  dense-matrix reference: sample/design identity, coefficients, scale,
  iterations/score, all working-correlation entries, both full covariance
  matrices, normal inference, current-row predictions and source order.
- Check author-log facts at printed precision without widening independent-R
  tolerances. Verify working and sandwich covariance are distinct and the
  selected result retains a unit-cluster correction of G/(G−1).
- Exercise balanced T=2 and T=10 synthetic panels, physical row permutation,
  duplicate/missing time keys, incomplete panels, rank failure, singular or
  non-positive-definite working matrices, prediction collisions and failed-fit
  state preservation.
- Run the smallest affected stats, language, runtime, reporting and CLI tests,
  then workspace check, scoped Clippy, formatting and diff checks.

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
