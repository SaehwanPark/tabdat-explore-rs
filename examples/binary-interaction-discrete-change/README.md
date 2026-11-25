# Discrete probability changes in continuous-interaction designs

## Bounded contract

Extend `estat margins, discrete(x)` after logit and probit when `x` is a
retained original continuous input with exact 0/1 support. `x` may appear as a
main-effect term, a source of one or more supported `c.x#c.z` products, or both.
The fit sample must contain both values.

For each retained estimation row, set `x` to 0 and 1 in turn, keep every other
source value and factor level fixed, and rebuild every product involving `x`.
The estimand is the average fitted-probability difference from 0 to 1. Delta
uncertainty uses the full selected covariance and the gradient of both
counterfactual design rows. The calculation uses fit-time source values after
active-data edits and does not mutate current data or refit.

Unsupported factor interactions and `##` expansion remain outside this syntax.
Invalid source terms, values outside `{0,1}`, or a retained sample missing a
level fail without changing the fitted state or active data.

## Verification plan

- Compare deterministic synthetic logit and probit fits against independent
  Base R estimates for full `x z c.x#c.z` and interaction-only designs.
- Check point changes and full-covariance delta SEs under model-based,
  independent-row robust and one-way clustered covariance.
- Cover retained sample exclusions, both-level checks, unknown inputs,
  current-data edits, product reconstruction, and failure-state preservation.
- Keep this synthetic method evidence distinct from Cameron–Trivedi
  illustration replication.
