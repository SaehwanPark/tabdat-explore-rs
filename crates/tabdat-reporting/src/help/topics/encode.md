# encode

How to invoke:
`encode <strvar>, generate(<newvar>) [, label(<lblname>)]`

What it does:
Convert a string variable into a new integer-coded variable (1..K for sorted unique
nonmissing values), create a matching value-label set, and attach it.

What problem it answers:
How do I turn string categories into numeric codes with value labels for tabulate and modeling?

Examples:
- `encode sex, generate(sex_n)`
- `encode region, generate(region_id) label(regionlbl)`
- `label list regionlbl`

Notes:
- Bounded eager string-source execution requires an active dataset and a new target column. Sorted unique SQL-non-NULL strings map to 1..K; stored empty strings are categories, NULL stays NULL. No general missing-value normalization is implied.
- The new column appends without changing prior row order, source columns/count/path or cached fits. Native integer type is reported: small nonempty mappings use INTEGER, empty/all-NULL mappings use BIGINT. Quoted names and empty data are supported.
- The generated column inherits the source variable label. The default value-label set name is the generated column name; `label(name)` selects another name. Encode replaces an existing set of that name, retaining other attachments to it, so choose names deliberately. Inspect the dictionary with `label list`.
- Terminal success shows `Encoded: N rows, K columns` using owned metadata. `label list` now shows typed keys, mappings and attachments readably; `head` inspects row values. No reporting scan, recalculated category count or mapping field is added to the encode result.
- CLI-v1 JSON returns `EncodeResult` with owned `data.dataset` only; code/mapping/affected-row fields are not invented, and dataset label/panel fields stay null placeholders. Explicit `label list` reports the dictionary.
- Numeric category codes do not automatically establish factor/dummy or ordinal modeling semantics. Choose a supported explicit design/transform and refit explicitly after changes when needed.
- Unknown/non-string sources, existing targets and native case/name failures preserve prior data/labels/model state. Lazy/remote execution and label dictionary persistence are uninstalled. CSV loading/reloading may treat empty fields as NULL; Parquet preserves stored empty strings. Output failure does not roll back completed effects (MIG-0008).
