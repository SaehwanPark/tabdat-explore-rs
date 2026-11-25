# Discrete probability changes for binary interaction sources

`estat margins, discrete(treated)` supports a retained original numeric source
whose fit-time values are exactly 0 and 1, even when the source appears in one
or more continuous product terms. The source may be included as a main effect,
occur only in product terms, or also appear beside additive factor terms. Both
source levels must occur in the retained fit sample.

For each retained row, the command sets `treated` to 0 and 1, holds all other
source values and factor levels fixed, and reconstructs every product involving
`treated`. It averages the fitted probability difference from 0 to 1. The delta
gradient averages `f'(eta_1) X_1 - f'(eta_0) X_0` across rows and uses the full
selected covariance, including cross terms for interaction coefficients.
Inference is asymptotic normal and conditional on the retained fit design.
Active data edits do not change fit-time sources or mutate the fitted state.

## Independent numerical evidence

The contract test uses the fixed 120-row fixture from
[`binary-discrete-change`](../binary-discrete-change/README.md). It makes `x`
missing for group 1 before fitting, leaving 114 rows and 19 clusters. The Base R
reference independently polishes the logit and probit score equations, builds
model-based, independent-row robust (`N/(N-1)`) and one-way clustered (`G/(G-1)`)
covariances, then evaluates both counterfactual designs and their full delta
gradients. The test covers:

- `x z treated c.treated#c.x c.treated#c.z`, with main effects and two products;
- the same model plus an additive factor, which remains fixed in both
  counterfactuals;
- `c.treated#c.x c.treated#c.z`, where the selected source occurs only in the
  products;
- logit and probit under all three covariance choices;
- active-data edits, unknown/nonbinary requests and fit-state preservation.

The runtime test is
[`binary_interaction_discrete_change_contract.rs`](../../crates/tabdat-runtime/tests/binary_interaction_discrete_change_contract.rs).
It compares estimates, delta standard errors, z statistics, p-values and 95%
interval endpoints at `3e-10` absolute tolerance. The Base R script reads the
shared fixture and prints its reference rows:

```sh
Rscript examples/binary-interaction-discrete-change/reference.R \
  > crates/tabdat-runtime/tests/fixtures/binary_interaction_discrete_change_reference.csv
```

Factor interactions, `##` expansion, weights and multiway clustering remain
unsupported. This is synthetic method evidence and does not claim an exact
Cameron–Trivedi illustration replication.
