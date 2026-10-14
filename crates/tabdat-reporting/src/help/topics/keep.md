# keep

How to invoke:
`keep varlist` or `keep if <predicate>`

What it does:
Project requested columns, or retain rows whose bounded predicate is TRUE.

What problem it answers:
How do I retain analysis/export variables in a chosen order, or select a cohort?

Examples:
- `keep age bmi`
- `keep sex age`
- `keep if age >= 18`
- ``keep `odd name` age``

Notes:
- Requires an active dataset and known columns. Rows, NULL values, native types and source path are retained; empty datasets are supported.
- Repeated columns produce native unique names (for example, `keep age age` produces `age` and `age_1`). Duplicate aliases do not automatically inherit label attachments.
- Successful projection prunes variable labels/value-label attachments for absent names, retains label definitions and the cached fit without refitting. Prediction still requires its predictor columns in current data; post-estimation uses the retained fit.
- CLI-v1 JSON returns `KeepResult` with `data.dataset` post-operation metadata: total row count and ordered native schema, not values or affected-row counts. Label/panel fields remain null placeholders. Inspect cells with `head`/`tail`.
- `keep if <predicate>` retains TRUE rows in prior relative order; FALSE/UNKNOWN rows are removed. All columns, types, source, labels and definitions survive. Active named tables are synchronized, not detached. CLI-v1 uses the existing `KeepResult` dataset shape for the current selected row count; it does not add predicate/matched/overflow counts.
- Predicates support existing comparisons, arithmetic, quoted identifiers/string literals, Boolean columns and NULL. Numeric truthiness, functions, conjunction/disjunction and additional options are not installed. Explicit `== null`/`!= null` test SQL NULL. Direct NaN/infinity comparisons remain native, not a finite filter; arithmetic uses the existing safe compiler, which converts nonfinite/failed arithmetic to NULL/UNKNOWN. For example, `keep if x > 0` can retain native NaN, whereas `keep if (x + 0) > 0` excludes it. Stored empty strings are not SQL NULL.
- Cached fit/inference/sample remain original until explicit `regress`; sample indices are not remapped to selected rows. Prediction uses current rows with the cached model. Empty/all/none/repeated selections are supported. Validation and a native staging-query failure preserve prior state; named synchronization/publication is multi-step, not a general atomicity guarantee. See MIG-0009.
- Failed validation preserves prior data/session state. Successful effects are not rolled back by later output failure (MIG-0008).
