# Continuous interaction method evidence

## Installed contract

`regress`, `logit` and `probit` accept one explicit two-variable continuous
product such as `c.x#c.z`. It contributes one column `x*z` named
`x__interaction__z`; it does not add either main effect. Include `x` and `z`
separately when the model needs them. Factor interactions, `##` expansion and
other transformations are outside this bounded syntax.

Rows with a missing/nonfinite input or a nonfinite product are excluded jointly
from estimation. Current-row prediction rebuilds each product and returns NULL
when an input or its product is invalid. Binary `estat margins` reports
derivatives with respect to original continuous variables using the product
rule and the selected full covariance. The binary fit retains interaction
source values aligned to its sample, so those effects remain tied to the fit
after active data edits. `estat margins, discrete(...)` accepts an original
exact 0/1 source used in a product term and rebuilds the product for each
counterfactual. See the separate
[interaction discrete-change evidence](../binary-interaction-discrete-change/README.md).
Binary JSON
labels these as `average_original_continuous_variable_derivative`; additive
models keep the existing `average_continuous_predictor_column_derivative`
label.

## Independent synthetic reference

The deterministic fixture has 240 rows. Rows 3 and 4 have a missing interaction
input; row 5 has finite inputs whose product overflows; and rows 6, 7 and 8
have model-specific missing outcomes. Each OLS, logit and probit model therefore
retains 236 rows. It is synthetic method evidence and does not claim to
replicate a Cameron–Trivedi illustration.

[`reference.R`](reference.R) uses Base R 4.6.2 to generate the fixture and
independent values. The OLS reference uses `lm.fit` and the residual-variance
classical covariance with N−P degrees of freedom. The logit reference polishes
the score equations and uses inverse Fisher information. The probit reference
polishes the score equations and uses inverse observed information, matching
TabDat's documented probit covariance convention.

The runtime contract compares at 3e-8 absolute tolerance:

- OLS coefficient estimates, standard errors, the full covariance matrix and
  current-row predictions.
- Logit and probit coefficient estimates and standard errors, current-row
  probabilities, probability delta-method SEs and transformed 95% link-scale
  intervals.
- Binary average marginal derivatives and standard errors for both original
  variables, under full `x z c.x#c.z` and interaction-only designs.
- Retained-sample behavior after active-data edits, discrete changes that
  rebuild products from an exact 0/1 source, malformed and duplicate syntax,
  and failed-fit state handling. Independent contrast evidence is in
  [`binary-interaction-discrete-change`](../binary-interaction-discrete-change/README.md).

Run `Rscript examples/continuous-interaction/reference.R` from the repository
root to regenerate the data and reference CSVs. The integration contract is
`crates/tabdat-runtime/tests/continuous_interaction_contract.rs`.
