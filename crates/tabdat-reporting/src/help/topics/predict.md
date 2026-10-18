# predict

Installed syntax:
`predict newvar [, xb | residuals | pr | n]`

Append a DOUBLE prediction column using retained coefficients on **current rows**:
- Supported regress/2SLS: xb (default) or residuals (outcome minus xb).
- Logit: pr (probability) or xb (log odds).
- Probit: pr (Gaussian probability) or xb (latent index).
- Poisson: n (conditional mean exp(xb)) or xb (log conditional mean).

**Bare predict remains xb for every installed family.** Request probabilities
with `, pr` or count means with `, n` explicitly. Count means are not probabilities
or integer draws; do not clip at one. `pr` is not Poisson prediction and `n` is
not supported after linear/binary fits. Prediction kinds cannot be combined.

Example (after loading numeric x/y):
- `regress y x`
- `predict fitted`
- `predict fitted_explicit, xb`
- `predict residual, residuals`
- `head 8`
- `export predictions.csv`

Count example:
- `poisson visits age income, robust`
- `predict expected_visits, n`
- `predict log_mean, xb`

Data and model notes:
- No refitting or estimation-sample filtering: excluded rows can receive
  predictions if their current predictors are available. Outcome values do not
  matter except for linear residuals. Weight/cluster columns are not needed.
- xb requires the original predictor names. Renaming/dropping a required column
  does not rewrite the model: restore it or deliberately refit. Retained-X
  `estat margins` remains separate from these current-row predictions.
- Linear fits retain native SQL NULL/NaN/infinity arithmetic. Binary/Poisson
  nonfinite or unrepresentable indices yield NULL. Stable binary links avoid
  exponential overflow. Poisson overflowing means yield NULL; finite negative
  exponential underflow is zero. Neither changes row order or the fit.
- `newvar` must not exist. Choose a fresh name or deliberately drop an output;
  there is no replacement. A prediction failure leaves data/labels/model intact.
  A failed new runtime fit clears stale estimation, so successfully refit before
  predicting again.
- Terminal/--text confirms dataset dimensions. CLI-v1 `PredictionResult` contains
  dataset metadata, not vectors; inspect with head/tail or save/export values.

Not installed: binary/Poisson residuals, spatial_lag, posterior_predictive,
std/interval/level()/saving() prediction, or arbitrary count PMF/CDF requests.
`probabilities` is not an accepted option spelling. Discovery/parser breadth does not
imply an installed workflow for other families.
