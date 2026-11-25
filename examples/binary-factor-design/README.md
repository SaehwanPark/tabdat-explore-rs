# Binary model factor design

## Planned bounded workflow

Add additive categorical main effects to `logit` and `probit` using the
established factor syntax:

```text
logit outcome income i.region
probit outcome income ib(West).region
```

Continuous predictors remain numeric columns. Factor predictors may be finite
numeric codes, text or booleans. The default omitted level, explicit reference
level, treatment-column ordering and coefficient names follow the existing
`regress i.var` / `ib(level).var` design contract. With `noconstant`, every
observed level contributes a column and an explicit reference is invalid.

The fit uses complete finite outcome/predictor rows in source order. Missing
outcomes or predictors are excluded; a complete row whose outcome is not
exactly 0 or 1 fails. Robust and one-way clustered fits retain their current
covariance conventions. The retained fit stores typed factor levels so
`predict ..., xb` and `predict ..., pr` score current data without refitting.
Missing values and factor levels absent from the estimation sample produce
NULL predictions. An explicit reference absent from the estimation sample
fails the fit.

Interactions and factor-specific `estat margins` are outside this slice.
Requests for unsupported factor margins must fail clearly; existing continuous
binary margins remain available for continuous-only fits. This synthetic method
fixture does not claim a textbook illustration replication.

## Independent validation plan

Use a deterministic synthetic CSV and Base R `glm` fits with logit and probit
links. Compare retained row identities, parameter order/names, coefficients,
model-based covariance, fitted probabilities, and `xb`/`pr` predictions. The
runtime contract will also cover robust and clustered factor fits, missing rows,
explicit and default references, `noconstant`, nonbinary complete outcomes,
unseen current-data levels, failed-fit state, JSON/text output and CLI execution.

Record reference versions and numerical tolerances here after the executable
fixture and tests are in place.
