# estat

How to invoke:
`estat vif`

What it does:
Report variance-inflation factors from the retained linear-regression predictor design, in predictor order. Requires an active dataset and a prior successful bounded `regress` fit.

What problem it answers:
How much does the fitted predictor design inflate variance through linear dependence among predictors?

Examples:
- `regress y x1 x2` then `estat vif`
- `regress y x1 x2, noconstant` then `estat vif`
- `regress y x1 x2, wls(weight)` then `estat vif`
- Batch JSON: `--json -c "use data.csv" -c "regress y x1 x2" -c "estat vif"`

Notes:
- Each auxiliary regression is **unweighted**, using the parent's retained sample and intercept convention, including after WLS/GLS. Missing/non-finite outcome/predictor values and required weight/group exclusions are inherited from the parent fit. This is not a weight-adjusted VIF diagnostic.
- The retained design is used even if current data changes after fitting; `estat vif` does not refit the model, rescan current data, or mutate dataset/model state.
- A supplied `mean_vif` aggregate row follows predictor rows when values are available. Labels are not unique identifiers; a predictor can itself be called `mean_vif`. Undefined factors appear as missing cells; the aggregate is absent if no values are available. An auxiliary-fit failure is an execution error. The single-predictor/no-intercept case currently fails because the zero-column auxiliary model is not estimable.
- Terminal output remains the ordered `Variable`/`VIF` table. CLI-v1 JSON returns `result_type: "TableResult"`, `data.headers` and ordered `data.rows`. Cells use the existing scalar policy: unavailable/non-finite numbers become `null`, finite zero remains numeric. JSON null does not distinguish unavailable values from infinity. No VIF-specific result type or extra model arrays are included; parent-fit metadata is in the preceding `RegressionResult`.
- Other `estat` forms may parse, but are not installed in this bounded runtime or admitted by CLI-v1 JSON. Generic table serialization does not enable direct SQL JSON.

Links:
- `SPEC.md`
