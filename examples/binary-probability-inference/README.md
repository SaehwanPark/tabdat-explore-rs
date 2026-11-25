# Pointwise probability uncertainty after logit and probit

## Installed workflow

After logit or probit, `predict target, pr se` writes the delta-method standard
error of the current-row fitted probability. It uses the retained coefficient
covariance selected by the fit: model-based, independent-row robust, or one-way
clustered. Current predictors are treated as fixed; this is not uncertainty in
their distribution or uncertainty in a future binary outcome.

`predict target, pr interval [level(95)]` writes the fitted probability and
`target_lower`/`target_upper` columns. Bounds are formed by taking a normal
interval for the retained latent index and transforming its endpoints through
the fitted logit/probit link. This keeps bounds in [0,1] and can make them
asymmetric. The default level is 95; requested levels must be finite and
strictly between 0 and 100.

Predictions score current rows, including rows excluded at fit time, using the
retained model. Missing/nonfinite predictors and factor levels absent at fit
time produce NULL in every requested output column. Any target collision,
invalid covariance calculation, or publication failure must preserve the
current dataset and fit. `xb`, point-only `pr`, and bare `predict` keep their
existing behavior. Standard errors and intervals apply only to probability
predictions; latent-index uncertainty, prediction intervals for outcomes,
simultaneous bands and few-cluster guarantees are out of scope.

## Independent validation

`Rscript reference.R` in `../binary-factor-design` uses Base R 4.6.2 and the
deterministic 240-row fixture for both mixed-factor and continuous-only designs.
The independent R calculation forms each current-row design matrix, evaluates
`x' V x`, derives the probability SE, and transforms the 90% and 95% normal
latent-index bounds through each link. Model-based, independent-row robust and
one-way clustered covariance are checked for logit and probit. The reference file
`../binary-factor-design/binary_factor_probability_inference_reference.csv`
contains 5,760 current-row/level/design records; all numerical outputs match
within 2e-8 absolute tolerance. Mixed-factor rows with missing x/region have
blank values; continuous-only rows require x but do not require region. The row
with missing y remains scoreable in both designs.

The stats contract checks every estimable current row against R and covers
zero variance, invalid levels, dimension errors and a negative variance.
Runtime tests exercise `pr se`, default 95% and explicit 90% intervals across
both designs and all six link/covariance combinations, missing predictors,
current-data edits, unseen factor levels, target/bound collisions, atomic
failure and retained-fit preservation. The scripts in `../binary-factor-design/logit.td` and
`probit.td` demonstrate the new commands. This synthetic method validation does
not claim a Cameron–Trivedi illustration replication.
