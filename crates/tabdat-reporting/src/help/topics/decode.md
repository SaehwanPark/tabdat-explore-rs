# decode

How to invoke:
`decode <numvar>, generate(<newvar>)`

What it does:
Convert a numeric variable that has attached value labels into a new string variable of those
labels. Codes without labels become missing.

What problem it answers:
How do I recover readable string categories from a labeled numeric variable?

Examples:
- `decode sex_n, generate(sex_str)`
- `label list`

Notes:
- Requires an active dataset, a numeric source with an attached value-label set, and a new target column. Every key in that set must be an integer key; numeric-spelling/text keys are rejected even if a spelling looks integer-valued. Empty mappings are supported.
- Native SQL numeric equality compares source values to integer keys, without explicit fractional truncation. Cross-type wide numeric comparisons follow native coercion, not a lossless DOUBLE/i64 equality guarantee. Unknown or NULL codes yield NULL; a mapped empty display string stays a stored nonmissing string.
- The VARCHAR output appends, preserving source columns/order/count/path, label definitions/attachments and the original cached fit/sample. It inherits the source variable label, but does not attach a new value-label set. Quoted names and empty data are supported.
- CLI-v1 JSON returns `DecodeResult` with owned `data.dataset` only, not a guessed mapping/count summary. Dataset label/panel fields stay null placeholders; inspect dictionaries using `label list`.
- Missing attachments, non-integer keys, nonnumeric sources, existing targets and native name failures preserve prior state. A new `use` clears session-local dictionaries; CSV/Parquet save/export does not archive them. Lazy/remote execution and dictionary persistence are uninstalled. CSV reload may turn empty strings into NULL. Output failure does not roll back completed effects (MIG-0008).
