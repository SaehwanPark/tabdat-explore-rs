# save

How to invoke:
`save <path> [, replace]`

What it does:
Write the current active dataset to a local Parquet file. Requires an active dataset; does not change its source path, rows, schema, labels or saved regression model.

What problem it answers:
How do I persist my current data, including prediction columns?

Examples:
- `save output.parquet`
- `save "out dir/results,2026.parquet"`
- `save output.parquet, replace`

Notes:
- Only `.parquet` output is installed for `save`; use `export` for CSV. Paths can be quoted in matching single/double quotes. Parent directories are created as needed.
- Existing targets require `replace`. Directory targets and invalid parent paths fail. Refused overwrites leave existing bytes unchanged.
- CLI-v1 JSON returns `result_type: "SaveResult"` with requested `data.path` and output-oriented `data.dataset` in the existing metadata shape. Its dataset path is the output destination, while `describe` still reports the active source. Label/panel JSON fields retain existing null-placeholder semantics, not a full metadata snapshot.
- File publication is not atomic. A backend failure may leave partial output or alter a replaced file; created parent directories may remain. Active session state is preserved, but file rollback is not promised.
- Failed result output exits 1 and stops later commands; a file already written by the completed command is not removed or rolled back. A broken stream is not guaranteed a JSON error envelope (MIG-0008).

Links:
- `SPEC.md`
