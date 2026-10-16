# estat

How to invoke:
`estat vif`
`estat ar, null(b1 b2 ...)`

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
- A supplied `mean_vif` aggregate row follows predictor rows when values are available. Labels are not unique identifiers; a predictor can itself be called `mean_vif`. Undefined factors appear as missing cells; the aggregate is absent if no values are available. An auxiliary-fit failure is an execution error.
- A lone predictor without an intercept uses the zero-column auxiliary model: its fitted values are zero, so SSR equals uncentered TSS and VIF/mean-VIF are 1 for a nonzero finite predictor. The library returns an undefined factor for an all-zero predictor; ordinary regression rank and zero-parameter validation are unchanged. This corrects the former diagnostic failure, allowing batches/scripts to continue normally (MIG-0011).
- Terminal output remains the ordered `Variable`/`VIF` table. CLI-v1 JSON returns `result_type: "TableResult"`, `data.headers` and ordered `data.rows`. Cells use the existing scalar policy: unavailable/non-finite numbers become `null`, finite zero remains numeric. JSON null does not distinguish unavailable values from infinity. No VIF-specific result type or extra model arrays are included; parent-fit metadata is in the preceding `RegressionResult`.
- Other `estat` forms may parse, but are not installed in this bounded runtime or admitted by CLI-v1 JSON. The supported forms are `vif` and `ar` (described below). Generic table serialization alone does not admit additional commands. Direct `sql` SELECT/WITH queries are independently admitted as `TableResult`; `sql ... into name` returns `SqlCreateResult`, and `use name` returns `ActivateResult`. See `help sql` for supported forms and native validation/state limits.

## Joint Anderson–Rubin test

After a **nonrobust** `ivregress 2sls` fit, run for example:

```text
ivregress 2sls y w, endog(x1 x2) iv(z1 z2)
estat ar, null(1.7 -0.6)
```

`null()` must specify a finite numeric value for **every** endogenous coefficient,
in the parent's `endog()` order. Decimal and scientific notation are accepted;
whitespace separates coefficients, with no whitespace inside a signed number.
Included exogenous coefficients (including the intercept when fitted) remain
unrestricted nuisance parameters. This is a joint test, not a scalar/subvector
test with other endogenous coefficients left unspecified.

The test regresses `y - X_endog * b0` on the included exogenous variables alone
and on the included exogenous plus excluded instruments, then compares residual
sums of squares. It reports F(q, N − rank[W,Z]), where q is the excluded-instrument
count. Valid instruments and homoskedastic errors are required; exact finite-sample
F inference additionally requires conditional normality. The test is robust to
weak identification, **not** heteroskedasticity or clustering, and it does not test
instrument validity. Robust/cluster parent fits are rejected, not silently
converted to classical inference. Undefined auxiliary variance, rank or residual
DF failures are explicit errors.

The retained complete-case fit sample/design is used even after current data is
changed; AR does not refit 2SLS, reread current columns or mutate model/data state.
`--text`, scripts and CLI-v1 JSON expose the joint null, endogenous names/order,
excluded instruments, N, both DF, F, p-value and assumptions (`AndersonRubinResult`).
A small p-value rejects the **entire supplied joint null** under these assumptions;
it does not identify which endogenous coefficient differs.
Confidence sets, endogenous nuisance coefficients and robust/cluster AR are not
implemented. Ordinary coefficient p-values and `test` remain conventional, not
weak-IV-robust inference.

Links:
- `SPEC.md`
