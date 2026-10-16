# predict

Installed syntax:
`predict newvar [, xb | residuals | pr]`

What it does:
Append a DOUBLE column of fitted linear values (xb, the default) or residuals
(observed outcome minus xb) after a successful supported `regress`/2SLS fit.
After `logit`, `pr` produces stable probabilities and `xb` produces log odds.
**Bare predict remains xb**, including after logit; request probabilities explicitly.

What problem it answers:
How do I apply the retained linear coefficients to current data and inspect or
persist observation-level results?

Examples (after loading numeric columns x and y):
- `regress y x`
- `predict fitted`
- `predict fitted_explicit, xb`
- `predict residual, residuals`
- `head 8`
- `export predictions.csv`

Data and model notes:
- Prediction uses current rows and the retained coefficients/intercept, not just the estimation sample. It does not refit or filter data. Rows excluded from estimation may receive predictions when their current predictors are available.
- xb requires the original predictor columns; residuals additionally requires the original outcome column. Weight/cluster columns are not required after fitting. Renaming or dropping required columns does not rewrite the retained model: restore those columns or explicitly refit before predicting.
- For linear fits, native SQL arithmetic applies: SQL NULL propagates to NULL; NaN/infinities are not newly filtered into a finite prediction sample. Residuals are outcome minus xb, not standardized or weighted residuals. After logit, missing/nonfinite predictors or an unrepresentable index instead yield NULL; stable sigmoid branches avoid exponent overflow. No outcome is required for logit probabilities/xb.
- `newvar` must not already exist. Choose a fresh name (or deliberately drop the old output) rather than expecting replacement. Prediction preserves row order and the cached fit; a failed attempted regression clears that fit, so fit successfully again before predicting.
- The shell/`--text` confirms the resulting dataset dimensions; CLI-v1 JSON returns `PredictionResult` dataset metadata, not a vector of predictions. Use `head`/`tail` to inspect values and `save`/`export` to persist current data.

Supported boundary:
Linear fits support xb/residuals; logit supports pr/xb only. Logit residuals,
probit prediction, `spatial_lag`, `posterior_predictive`, and Bayesian
`std`/`interval`/`level()`/`saving()` forms do not execute. Discovery and syntax
previews are not execution guarantees;
`probabilities` is not an accepted option spelling. Other estimator families
shown in reference topics are not installed prediction workflows.
