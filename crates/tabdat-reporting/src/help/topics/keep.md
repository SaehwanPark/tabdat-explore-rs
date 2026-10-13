# keep

How to invoke:
`keep varlist`

What it does:
Keep only the requested columns, in request order, without filtering observations.

What problem it answers:
How do I retain analysis or export variables in a chosen order?

Examples:
- `keep age bmi`
- `keep sex age`
- ``keep `odd name` age``

Notes:
- Requires an active dataset and known columns. Rows, NULL values, native types and source path are retained; empty datasets are supported.
- Repeated columns produce native unique names (for example, `keep age age` produces `age` and `age_1`). Duplicate aliases do not automatically inherit label attachments.
- Successful projection prunes variable labels/value-label attachments for absent names, retains label definitions and the cached fit without refitting. Prediction still requires its predictor columns in current data; post-estimation uses the retained fit.
- CLI-v1 JSON returns `KeepResult` with `data.dataset` post-projection metadata: total row count and ordered native schema, not values or affected-row counts. Label/panel fields remain null placeholders. Inspect cells with `head`/`tail`.
- Conditional `keep if <expression>` parses but row-filter execution and successful JSON reporting are not installed. No overflow-row diagnostics are supplied.
- Failed validation preserves prior data/session state. Successful effects are not rolled back by later output failure (MIG-0008).
