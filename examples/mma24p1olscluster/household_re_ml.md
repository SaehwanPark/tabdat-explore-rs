# WIP contract: Chapter 24 household Gaussian RE maximum likelihood

**Not yet delivered or validated.** This slice targets the author's Gaussian
random-intercept `xtreg ..., mle` output in §24.7/Table24.4, using the existing
native unbalanced ML estimator rather than changing the default RE-GLS method.

- Source: pinned `vietnam_ex1.asc` and precise DTA as documented in the
  [household README](README.md). Outcome `lnexp12m`; six numeric predictors
  `lnhhexp age sex hhsize farm educ`, intercept and `commune` random intercept.
  Joint-finite sample 5,006 households/194 communes, sizes1..39 including a
  singleton; `source_row` is only a unique key, not time. Prepare and validate
  source, estimate and report ML, retain population xb/residual predictions,
  export ordered records and document precise missingness/state/failure rules.
- Independent reference: base R profile likelihood and analytic/numerical joint
  observed information, independently cross-checked with `nlme::lme` on the
  identical sample/design. Validate complete coefficient and joint covariance,
  variance-component uncertainty, log likelihood, slope and boundary likelihood
  tests and all source/prediction rows; a separate author-log check uses half a
  final printed digit where reproducible, disclosing discrepancies rather than
  widening tolerances. Compare text/JSON and preserve existing fit on export failure.
- Acceptance: (1) ASCII and precise DTA exact samples/source order, (2) complete
  independent ML/inference comparison and all predictions, (3) synthetic
  unbalanced/singleton and zero-unit-variance boundary controls, (4) text/JSON
  and owned state reproducibility. Invalid source keys, rank/convergence failure
  and export collisions must not silently yield a partial report or clobber files.
- Exclusions: no complex survey, weights, causal, posterior unit-effect, historical
  optimizer/bitwise parity, generic Hausman or whole-Table24.4 completion claim.
  Boundary slope LR and component uncertainty follow existing native policy.

Implementation and evidence will replace this plan before the draft is ready.
