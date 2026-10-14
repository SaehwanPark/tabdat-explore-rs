# isid

How to invoke:
`isid varlist [, missok]`

What it does:
Assert that the requested key combination uniquely identifies current active rows. Successful validation is read-only; it does not repair data or impose a persistent join/reshape constraint.

What problem it answers:
Can I safely use these proposed keys before a supported lookup or reshape workflow?

Examples:
- `duplicates report id visit`
- `isid id visit`
- `isid id visit, missok`

Notes:
- Installed eager DuckDB workflow only; no lazy/remote loading is installed by this command. At least one known key is required. Requested order/repeated/backtick-quoted names are retained. Empty input with known keys succeeds with zero rows/groups/missing rows.
- Without missok, any SQL NULL key component fails. With missok, NULL-containing combinations are allowed only if the entire combination is unique; repeated combinations, including NULLs, always fail. Missing-key rows counts rows having at least one NULL component, not missing cells. Stored empty strings/NaN/infinity are not SQL NULL. Equality/grouping remains native, not automatic key coercion or Python/Stata missing-order parity.
- Terminal success explicitly reports keys, checked rows, distinct groups, missing-key rows and missing allowance, with quoted/control-safe names. CLI-v1 `IsidResult` data contains exactly `variables`, `total_rows`, `unique_groups`, `missing_key_rows`, `missok`; wide counts require lossless integer parsing. No invented passed/duplicate counters, sample fields or constraint registry is added.
- A failed check produces the existing execution error and no success envelope. Diagnostics describe missing/duplicate groups; nested script path/line and stop-on-failure behavior are retained. Run duplicates first for aggregate diagnosis. Neither command lists/tags/drops duplicate rows or enforces cardinality on a later join.
- Validation uses current data, not retained model samples. Success/failure leaves data/schema/source/order, labels, named ownership and original cached fit/inference/sample unchanged. No automatic refit or metadata rescan is performed.
- No row filter/assignment, missok value, extra option or installed by runner. Generic by parsing is not execution support; CLI JSON preflight rejects grouped guards. Output write/flush failure stops later guards/files while retaining completed effects (MIG-0008).
