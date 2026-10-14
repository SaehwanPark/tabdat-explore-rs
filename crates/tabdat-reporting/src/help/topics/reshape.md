# reshape

Terminal success now shows `Reshaped: N rows, K columns` using owned metadata,
not guessed direction/stubs/changed-row diagnostics. Inspect the selected layout
with `head`/`describe`; no reporting scan or pivot-semantics change is added.

How to invoke:
`reshape long|wide varlist, i(idvars) j(jvar)`

What it does:
Convert bounded repeated-measures layouts; CLI-v1 JSON reports owned `ReshapeResult` `data.dataset` post-operation metadata, not direction/stubs/changed-row diagnostics or a rescan.

What problem it answers:
How do I prepare wide observations for grouped EDA and an explicit long-data fit, then predict and persist the selected layout?

Examples:
- `reshape long income cost, i(id) j(year)`
- `reshape wide income cost, i(id) j(year)`
- `by year: count`

Notes:
- Long requires every requested `stub_suffix` for the union of suffixes. It emits identifiers in requested order, text j and requested stubs; unrelated columns are omitted. Source-row order is preserved, then j-values scan requested stubs in command order and matching wide columns in schema order, retaining each suffix's first appearance. Native UNION type promotion applies; SQL NULL cells are retained. Identifiers need not be unique; this is not uniqueness validation.
- Wide emits identifiers followed by requested stub/suffix columns. Non-NULL j keys are cast to text and suffixes sort lexically, not numerically. Rows follow the first active source row for each identifier group, including NULL identifier groups. Cells use native MAX(CASE ...) per stub: duplicate cells are combined, NULL cell values skipped and missing cells NULL. Different stub maxima need not come from the same row. NULL j rows contribute no cells, though their identifier groups remain. This is not a general lossless pivot or duplicate-cell rejection.
- Identifier/stub names may be quoted as supported by the parser; native identifier/literal quoting handles suffix contents. Long requires an unused j name; wide checks generated-name conflicts. Unknown names, missing stub columns, incompatible native UNION types and invalid options fail. Empty long datasets with valid stubs succeed; empty/all-NULL-j wide datasets fail because there is no generated suffix set.
- Source identity persists, but reshaping detaches active named ownership and leaves registered snapshots unchanged. Labels/attachments are pruned to surviving names; dictionaries persist, not propagated to generated names. Dataset JSON label/panel fields remain null placeholders, not metadata snapshots; no panel/factor/ordinal modeling is installed.
- Original cached fit, parameter names, sample indices and inference survive until explicit `regress`. Those indices describe the original fit, not the reshaped rows. Cached prediction with removed predictors fails rather than silently refitting. Explicitly refit on the desired shape before predicting it.
- Ordinary validation and a native staging-query failure preserve prior active data/state. Native publication faults are not injected; no general transactional/atomic publication guarantee is made. Output write/flush failure stops later pivots/files without rolling back completed effects (MIG-0008). CSV may re-infer text j keys/types on reload; Parquet retains stored native types. Neither archives label dictionaries.
