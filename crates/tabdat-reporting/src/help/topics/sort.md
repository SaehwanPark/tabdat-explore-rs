# sort

How to invoke:
`sort varlist`

What it does:
Stable-sort current rows by ascending columns, applied left-to-right. Numeric values use numeric order, text uses native lexicographic order, booleans are false before true, and SQL NULLs are last.

What problem it answers:
How do I order the dataset before inspecting, predicting or exporting it?

Examples:
- `sort treatment age`
- `sort household_id visit`
- `` sort `odd name` ``

Notes:
- Requires an active bounded eager dataset and known keys. Ties on all keys preserve their prior active-row order, including after an earlier sort. Repeated keys and empty datasets are supported; columns/types/source/row count and labels are retained.
- `sort` is ascending; use `gsort -score +id` for mixed directions. Expressions, options, conditions and `by:` execution are not installed here. Lazy/remote data execution is not enabled.
- The cached fit and its original retained sample/design are preserved without refitting or reinterpreting sample indices as sorted row positions. Prediction evaluates the current ordered rows; post-estimation still uses the retained fit.
- CLI-v1 JSON returns `SortResult` with owned `data.dataset` metadata. There is no key list, changed-row count or cell data in this result; inspect values with `head`/`tail`. Label/panel fields remain null placeholders, not full session metadata.
- Failed validation preserves prior data/session state. Successful sorting is not rolled back by later output failure (MIG-0008).
