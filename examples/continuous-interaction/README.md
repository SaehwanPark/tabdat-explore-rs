# Continuous interaction design

## Bounded contract

This work adds one explicit product term, `c.x#c.z`, to `regress`, `logit`,
and `probit`. It contributes the single numeric column `x#z = x * z`; users
include `x` and `z` separately when they want the corresponding main effects.
The syntax does not expand `##`, factor-by-factor products, polynomial terms, or
other transformations. The term order in the command determines parameter
order. Duplicate terms and generated-name collisions must not silently alter
the requested design.

Rows with a missing/nonfinite input or a nonfinite product are excluded from
the fit sample. Current-row predictions reconstruct the product from retained
source-variable names and return NULL when either input is missing/nonfinite.
For binary fits, continuous AMEs use the product-rule derivative and the full
retained covariance delta method. Discrete 0/1 changes in an interaction model
must fail explicitly until their changed-product estimand is implemented.

## Verification plan

- Fit an explicit deterministic continuous `x`, `z`, `x*z` design in R and in
tabdat; compare OLS coefficients/covariance/predictions and logit/probit
coefficients, fitted probabilities, AMEs, and AME covariance-based standard
errors.
- Verify a missing predictor and a nonfinite product row are excluded, current
row prediction preserves order and missingness, and a failed fit leaves no
stale inference state.
- Verify malformed/factor interaction syntax and duplicate terms fail without
mutating data or preserving a prior model.
- Keep this synthetic design evidence distinct from Cameron–Trivedi textbook
replication. It establishes the installed command contract only.
