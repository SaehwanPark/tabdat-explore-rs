# Pointwise probability uncertainty after logit and probit

## Planned contract

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

The independent reference will use Base R 4.6.2 on the synthetic mixed-factor
fixture, both links and model-based/robust/cluster covariance. It will compare
retained-sample estimates and link-scale bounds, plus current-row values after
edits. This validates a method contract, not a Cameron–Trivedi illustration.
