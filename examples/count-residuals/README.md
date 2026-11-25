# Count-model Pearson and deviance residuals

## Planned scope

Add current-row `predict` modes for Pearson and deviance residuals after Poisson
and NB2 fits. The existing response residual mode remains `y - mu`. These are
diagnostics at the current observed outcome and predictor values, scored with
the retained coefficients, NB2 dispersion and any retained offset or
exposure. They do not refit the model or use the estimation sample as a filter.

For Poisson, use variance `V(mu) = mu`. For NB2, use
`V(mu) = mu + alpha * mu^2`, where `alpha` is the retained fitted dispersion.
Pearson residuals are `(y - mu) / sqrt(V(mu))`, without an estimated scale
factor. Deviance residuals are `sign(y - mu) * sqrt(D)`, where the Poisson
unit deviance is
`D = 2 * (y * log(y / mu) - (y - mu))`, taking `y * log(y / mu) = 0` at
`y = 0`; the NB2 unit deviance is
`D = 2 * (y * log(y / mu) - (y + 1/alpha) * log((1 + alpha*y) / (1 + alpha*mu)))`,
with the same zero-count convention.

Current rows with missing/nonfinite inputs or an undefined/nonfinite residual
produce NULL. A present outcome must still be an exact nonnegative integer in
the model's supported range; invalid outcomes fail atomically without changing
the data or retained fit. Prediction targets must remain new columns. Residual
values do not depend on whether the fit requested model-based, robust or
one-way clustered covariance. Residual intervals, standardized/studentized
residuals, leverage and diagnostic plots remain outside this slice.

## Independent reference and verification plan

Use a deterministic synthetic count fixture, Base R `glm` and `MASS::glm.nb`,
and explicit formulas on current rows. Compare Poisson/NB2 residuals after
current-data edits under model-based, robust and one-way clustered fits. Check
offset/exposure scoring, missing rows, zero outcomes, zero fitted means,
invalid current outcomes and output-collision recovery. Exercise JSON/text
scripts and verify response-residual and existing mean-prediction compatibility.

This is synthetic method evidence. It does not claim completion of a Cameron and
Trivedi illustration or a formal goodness-of-fit test.
