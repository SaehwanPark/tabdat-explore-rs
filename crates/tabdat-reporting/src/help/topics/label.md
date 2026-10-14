# label

How to invoke:
`label variable varname "text"`, `label variable varname, clear`,
`label define set_name value "text" ... [, replace]`,
`label values varname set_name`, `label values varname, clear`,
`label list [set_name ...]`, `label drop set_name ...`

What it does:
Manage bounded session-local variable labels and named value-label dictionaries on the active dataset, without changing cells or cached models.

What problem it answers:
How do I document a variable or attach category meanings for tabulation and inspect that dictionary in an automated workflow?

Examples:
- `recode age (min/17=0) (18/max=1), generate(adult)`
- `label variable adult "Adult indicator"`
- `label define adult_codes 0 "Child" 1 "Adult"`
- `label values adult adult_codes`
- `tabulate adult`
- `label list adult_codes`
- `label values adult, clear`
- `label drop adult_codes`

Notes:
- Requires an active dataset; unknown variables/sets fail without changing prior metadata/data. Definitions must be new unless `replace` is supplied; duplicate keys reject. Drop validates all requested names before removing sets and their attachments.
- Clear removes only the selected variable label or attachment. Replacing a definition preserves attachments. Variable/attachment/set names are normalized into deterministic order; mappings retain definition order, not sorted numeric order. Backtick-quoted column names and empty data are supported.
- Terminal results show the completed action/message and owned variable labels, attachments and value-label sets. Names/text are quoted and controls escaped. Keys are explicitly typed: `integer 1`, `number "1.20"` (exact source spelling), or `text "1.20"`; neither numeric spelling nor wide integers are rounded. Supplied variable/set/mapping order is retained. Empty sections show `(none)`; an absent dictionary shows `No label metadata.`.
- `label list` is read-only. With set names it returns those sets and their attachments, but still includes all variable labels; it does not change the session dictionary.
- CLI-v1 JSON returns `LabelResult` with `data.action` (variable/define/values/list/drop), `message`, and owned `metadata` or null when empty. Metadata has `variable_labels` and `attachments` pair arrays, and `value_sets` containing `name` and `mappings` pairs `[key, display_text]`. Key objects use `{kind, payload}`: integer/i64 JSON number, number/exact numeric spelling string, text/string. Wide integers require lossless consumer parsing. Reporting preserves owned order and spelling, without rescanning or mutating state.
- Dataset envelopes still have null label/panel placeholders; use `label list` to inspect the dictionary explicitly. These labels do not imply general metadata-aware formatting in describe/codebook or other commands.
- Rename remaps variable labels/attachments; projection prunes absent variables, retains definitions. Successful replace/recode replacement removes selected attachments, retaining variable labels/definitions and cached fits. Explicitly refit after changing values if needed.
- Labels are session-local. A new `use` clears them; CSV/Parquet save/export does not archive the dictionary. `label save`/`label use` and third-party metadata formats are not installed. Output failure does not roll back completed effects (MIG-0008).
