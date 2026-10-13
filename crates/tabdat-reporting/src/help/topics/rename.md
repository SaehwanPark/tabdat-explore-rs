# rename

How to invoke:
`rename old_name new_name`

What it does:
Rename one existing column without changing its position, values, native type or row order.

What problem it answers:
How do I normalize imported column names before transforming, modeling or exporting data?

Examples:
- `rename sex gender`
- ``rename `raw age` age``
- `` rename label `标签"列` ``

Notes:
- Requires an active bounded eager dataset and a known source column. Empty datasets and backtick-quoted names are supported. Source path and total row count are retained; lazy/remote data execution is not enabled.
- Existing targets are rejected, including renaming a column to its current name. Native case-insensitive collisions also fail without publishing the staged rename. Options, conditions and multiple-column rename syntax are not supported.
- Variable labels and value-label attachments move to the new name; label definitions are retained. The cached fit keeps its original parameter names and retained sample without refitting or rewriting them. Post-estimation uses that fit; prediction still requires the original predictor columns in current data. Rename before fitting, or explicitly refit after changing a predictor name.
- CLI-v1 JSON returns `RenameResult` with owned `data.dataset` metadata. No old/new-name or affected-cell fields are invented from the metadata-only execution result. Label/panel fields remain null placeholders, not full session snapshots. Use `head`/`tail` to inspect values.
- Validation/native collision failures preserve prior data/session state. Successful effects are not rolled back by later output failure (MIG-0008).
