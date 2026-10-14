# duplicates

How to invoke:
`duplicates [report] [varlist]`

What it does:
Report owned duplicate-key aggregates for all public columns by default, or requested keys. It diagnoses only: no rows are listed, tagged, removed or repaired.

What problem it answers:
Do proposed lookup or reshape keys contain repeated combinations before I validate them with `isid`?

Examples:
- `duplicates`
- `duplicates report id`
- `duplicates id visit`

Notes:
- Read-only installed eager DuckDB data workflow; this command does not install lazy/remote loading. Requested order and repeated/backtick-quoted key names are preserved; default keys follow current schema order, including newly generated columns.
- Native grouping determines equality, including SQL NULL combinations, NaN/infinity, stored empty strings and native types. NULL keys are intentionally grouped; stored empty/nonfinite values are not reclassified as missing. No general cross-type/collation/key-coercion parity is implied.
- Terminal output quotes/escapes key names and labels distinct groups explicitly. CLI-v1 `DuplicatesResult` data contains exactly `variables`, `total_rows`, `unique_groups`, `duplicate_groups`, `duplicate_rows`, `extra_rows`, `max_copies`. Unique groups means all distinct combinations, including duplicate and NULL-containing groups, not only singleton groups. Duplicate rows counts every member of repeated groups; extra rows counts their surplus after one representative per group. Empty input reports zeros, including maximum copies zero. Wide counts require lossless JSON integer parsing.
- Counters describe current active data, not retained model samples. Data/schema/source/order, labels, named ownership and cached fit/inference/sample remain unchanged. No metadata rescan or guessed duplicate-row output is added.
- Unknown/no-active keys fail. No options, if clause or installed by runner; accepted generic by syntax is not executable here and is rejected before JSON execution. Error/output failure stops later script effects/files, retaining completed effects (MIG-0008). See `help isid` for uniqueness validation.
