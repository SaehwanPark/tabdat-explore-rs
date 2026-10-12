# Parametric survival feasibility contract (in progress)

Base c5e6efb. Classical exponential and Weibull PH rate parameterization,
positive finite complete times, binary failures and two predictors/intercept.
Native duration uses AFT beta and raw sigma: exponential gamma=-beta; Weibull
PH gamma=-beta/sigma and log(shape)=-log(sigma). Transform full observed joint
covariance using its explicit Jacobian, retaining cross terms, likelihood,
exact retained/failure/censored rows and correctly labeled parameters.

Predeclare MLE abs 1e-7 + rel 1e-6. Compare pinned executor (original quasi-Newton
covariance retained) and independent R survival with PH/Jacobian conversion;
separately labeled external observed-Hessian Python probe if approximate covariance
fails. No tolerance widening, source modification or runtime Python/R dependency.
Native requires an intercept; no-intercept is explicitly rejected for this slice.
All-uncensored input omits native censor-list separator to avoid duplicate constant
in initial OLS; all-censored input is rejected as unidentified. Mixed censoring
must not silently drop predictors/sample. Same-source duration plugin independently
selected under shared-core/serialized lifetime controls. No Cox, robust/cluster,
left truncation/competing risks/missing sample/post-estimation product acceptance.
