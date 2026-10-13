# drop

How to invoke:
`drop varlist`

What it does:
Remove the requested columns, preserving the remaining schema order without filtering observations.

What problem it answers:
How do I remove unused analysis or export variables?

Examples:
- `drop cost`
- `drop age bmi`
- ``drop `odd name```

Notes:
- Requires an active dataset and known columns. Removing every column is rejected. Repeated names are removed once. Rows, NULL values, remaining native types and source path are retained; empty datasets are supported.
- Successful projection prunes variable labels/value-label attachments for absent names, retains label definitions and the cached fit without refitting. Prediction still requires its predictor columns in current data; post-estimation uses the retained fit.
- CLI-v1 JSON returns `DropResult` with `data.dataset` post-projection metadata: total row count and ordered native schema, not values or affected-row counts. Label/panel fields remain null placeholders. Inspect cells with `head`/`tail`.
- Conditional `drop if <expression>` parses but row-filter execution and successful JSON reporting are not installed. No overflow-row diagnostics are supplied.
- Failed validation preserves prior data/session state. Successful effects are not rolled back by later output failure (MIG-0008).
