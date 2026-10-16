# status

How to invoke:
`status`

What it does:
Inspect owned current-dataset/named-table metadata and the small retained linear
model summary without scanning data, refitting or initializing DuckDB.

What problem it answers:
What data and original fitted model are held before my next inspection,
prediction or explicit refit?

Examples:
- `status`
- `use data.parquet`
- `regress y x`
- `rename x current_x`
- `status`
- `rename current_x x`
- `predict fitted`

Notes:
- Empty-session status succeeds with no dataset/model and leaves the data engine uninitialized. After data operations, it reports actual engine initialization, current source/schema/row count, active named-table name and registered names in literal sorted order. It does not read named relations to collect counts.
- Retained linear metadata reports the original outcome/predictor names, intercept convention, fitted N, residual degrees of freedom and original candidate/excluded counts when available. It copies no design, coefficient, fitted-value, weight, cluster or row-index arrays.
- Current data and the retained model can differ after rename/drop/filter/activation. Status neither certifies prediction eligibility nor detects all value/type changes; restore required columns or explicitly refit as appropriate. Prediction uses current data, while lincom/test/VIF use the retained fit. Failed attempted regression clears the fit; failed load/other commands follow their existing state policy, not a new rollback guarantee.
- CLI-v1 `StatusResult.data` contains `data_initialized`, nullable `dataset` (existing metadata encoding), nullable `active_table_name`, sorted `named_tables`, and nullable `linear_model`. Model fields are `outcome`, ordered `predictors`, `include_intercept`, `observation_count`, `degrees_of_freedom`, nullable `sample_total_observations` and `sample_dropped_observations`. Dataset label/panel placeholders remain null, not dictionary snapshots.
- Shell/`--text` displays the bounded status. Default batch success stays quiet; use `--json` for structured metadata. No coefficients/inference listing, estimator/covariance inference, original fit source, automatic stale/ready flag, last-operation/materialization tracking, lazy/remote/panel runtime or future estimator-family status is installed.
