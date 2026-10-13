# describe

How to invoke:
`describe`

What it does:
Show dataset shape, columns, and types.

What problem it answers:
What variables are in the active dataset?

Examples:
- `use patients.parquet`
- `describe`

Notes:
- Requires an active dataset. Reports owned current metadata without rescanning or changing dataset/model state.
- CLI-v1 JSON uses `result_type: "DescribeResult"` and `data.dataset`, with the existing load/prediction metadata shape: source `path`, integer `row_count`, schema-ordered `columns` (`name`, native `data_type`), `execution_mode` and `lazy_engine`.
- The schema reflects current transformations/prediction columns; the source path remains the load source. This command does not save/export the dataset.
- `label_metadata` and `panel_metadata` retain the existing JSON null-placeholder policy, not a full snapshot of session labels/panel settings.
- Only no-argument `describe` is supported; variable lists/options are not implied.

Links:
- `SPEC.md`
