# drop

How to invoke:
`drop varlist` or `drop if <predicate>`

What it does:
Remove requested columns, or remove rows whose bounded predicate is TRUE.

What problem it answers:
How do I remove unused variables or exclude observations matching a condition?

Examples:
- `drop cost`
- `drop age bmi`
- `drop if age == null`
- ``drop `odd name```

Notes:
- Requires an active dataset and known columns. Removing every column is rejected. Repeated names are removed once. Rows, NULL values, remaining native types and source path are retained; empty datasets are supported.
- Successful projection prunes variable labels/value-label attachments for absent names, retains label definitions and the cached fit without refitting. Prediction still requires its predictor columns in current data; post-estimation uses the retained fit.
- CLI-v1 JSON returns `DropResult` with `data.dataset` post-operation metadata: total row count and ordered native schema, not values or affected-row counts. Label/panel fields remain null placeholders. Inspect cells with `head`/`tail`.
- `drop if <predicate>` removes TRUE rows only; FALSE/UNKNOWN rows survive in prior relative order. Thus keep/drop partition input even when a comparison is UNKNOWN. All columns, types, source, labels and definitions survive. Active named tables are synchronized, not detached. CLI-v1 uses the existing `DropResult` dataset shape for the selected row count, without predicate/matched/overflow counts.
- Predicates use the installed comparison/arithmetic subset, quoted names/string literals, Boolean columns and NULL; no numeric truthiness, functions, logical conjunction/disjunction or extra options. Explicit NULL equality tests SQL NULL only. Direct nonfinite comparisons are native; safe arithmetic turns nonfinite/failed arithmetic into NULL/UNKNOWN. `drop if x > 0` can therefore remove native NaN, while `drop if (x + 0) > 0` retains it. Stored empty strings are not NULL. See `help keep` and MIG-0009.
- Original cached fit/inference/sample persist until explicit refit; original sample indices are not selected-row indices. Prediction uses current rows. Empty/all/none/repeated selections work. Validation and native staging-query failure preserve prior state; named synchronization/publication is multi-step, not generally atomic.
- Failed validation preserves prior data/session state. Successful effects are not rolled back by later output failure (MIG-0008).
