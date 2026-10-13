# gsort

How to invoke:
`gsort [+|-]varlist`

What it does:
Stable-sort current rows by per-key directions, applied left-to-right. Prefix an unquoted key with `+` for ascending or `-` for descending; an omitted prefix is ascending. SQL NULLs are last in both directions.

What problem it answers:
How do I rank rows or combine ascending and descending keys before inspection or export?

Examples:
- `gsort -date +patient_id`
- `gsort +site -score`
- `` gsort `-literal_name` `` (quoted literal name, ascending)

Notes:
- Requires an active bounded eager dataset and known keys. Ties on all keys preserve their prior active-row order, including after an earlier sort. Repeated keys and empty datasets are supported; columns/types/source/row count and labels are retained.
- A backtick-quoted identifier is a literal name, not a direction prefix. Options, conditions, assignments and `by:` execution are not installed here. Lazy/remote data execution is not enabled.
- The cached fit and its original retained sample/design are preserved without refitting or reinterpreting sample indices as sorted row positions. Prediction evaluates the current ordered rows; post-estimation still uses the retained fit.
- CLI-v1 JSON returns `GsortResult` with owned `data.dataset` metadata. There is no key list, changed-row count or cell data in this result; inspect values with `head`/`tail`. Label/panel fields remain null placeholders, not full session metadata.
- Failed validation preserves prior data/session state. Successful sorting is not rolled back by later output failure (MIG-0008).
