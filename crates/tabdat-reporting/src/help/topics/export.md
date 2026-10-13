# export

How to invoke:
`export <path> [, replace]`

What it does:
Write the current active dataset to a local CSV file with a header. Requires an active dataset; does not change its source path, rows, schema, labels or saved regression model.

What problem it answers:
How do I write current data, including prediction columns, for another tool?

Examples:
- `export cleaned.csv`
- `export 'out dir/results,2026.csv'`
- `export cleaned.csv, replace`

Notes:
- Only `.csv` output is installed for `export`; use `save` for Parquet. Parent directories are created as needed; existing targets require `replace`. Directory targets and invalid parent paths fail. Refused overwrites leave existing bytes unchanged.
- CSV is a text interchange format, not a general type/label-preserving archive. For supported roundtrips, quoted text, NULLs and row order are tested; use Parquet when native schema preservation matters. Session labels/panel settings are not advertised as persisted metadata.
- CLI-v1 JSON returns `result_type: "ExportResult"` with requested `data.path` and output-oriented `data.dataset` in the existing metadata shape. Its dataset path is the destination; `describe` still reports the active source. Label/panel JSON fields remain null placeholders, not a full metadata snapshot.
- File publication is not atomic. Backend failure may leave partial output or alter a replaced file; created parents may remain. Active session state is preserved, but file rollback is not promised.
- Failed result output exits 1 and stops later commands; a file already written by the completed command is not rolled back. Broken streams are not guaranteed JSON error envelopes (MIG-0008).

Links:
- `SPEC.md`
