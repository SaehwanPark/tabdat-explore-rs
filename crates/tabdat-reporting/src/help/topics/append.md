# append

Terminal success now shows `Appended: N rows, K columns` using owned current
metadata, without a rescan or invented source-size/added-row diagnostics. Inspect
values with `head`/`describe`; existing compatible-schema and ordering rules remain.

How to invoke:
`append table_name`

What it does:
Stack rows from an existing session-local named table after the active dataset using installed compatibility rules.

What problem it answers:
How do I combine compatible observations vertically before summary/model/persistence?

Examples:
- `sql select x,y from active order by x into followup`
- `use primary.csv`
- `append followup`
- `summarize x y`
- `regress y x`

Notes:
- Requires an active dataset and registered named table. Column-name sets must match; right column order may differ and is aligned by name into left schema order. Names match under current exact-name checks.
- Canonical native type strings must match after alias/case/whitespace normalization. Integer widths and decimal width/scale remain distinct. No implicit schema union, missing-column fill or general type coercion is installed; use an explicit supported transform/SQL design when needed.
- Active rows remain first, followed by named rows in their stored order. NULLs and duplicate rows are retained; append does not sort, deduplicate or interleave. Empty/self/repeated appends and quoted column names are supported. Sort/query relations deliberately before append if reproducible ordering matters.
- CLI-v1 JSON returns `AppendResult` with owned `data.dataset`: retained source, total rows and ordered native schema, not input paths/added-row diagnostics. Dataset label/panel fields stay null placeholders.
- Existing left variable labels, definitions and attachments remain; right labels are not propagated. The original cached fit/sample remains unchanged. Explicitly refit combined observations for estimation; prediction uses current rows with the retained fit.
- Append detaches active named-table ownership but retains the dataset source identifier. Registered prior named snapshots remain unchanged. Reactivating that source name with `use` restores its old relation, not the appended data, including after later prediction/transformation of the detached relation.
- Unknown tables, extra/missing columns and incompatible types reject without changing prior data/session/named state in tested cases; parser errors and output guards stop later effects/files. Native publication faults were not injected and no general atomicity guarantee is added. Completed effects/files survive later output failure (MIG-0008). By/lazy/remote execution and whole-command parity are not installed.
- In a capable interactive terminal, Tab completes the first bare table-name argument from literal/case-sensitive registered session names, refreshed before each prompt after partial scripts/failures. No filesystem/path completion or activation/loadability validation is added; piped input is unchanged.
