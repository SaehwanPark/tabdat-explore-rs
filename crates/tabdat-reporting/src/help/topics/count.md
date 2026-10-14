# count

How to invoke:
`count [if expr]`

What it does:
Count all current rows, or only rows for which a bounded predicate is TRUE.
FALSE and NULL/UNKNOWN predicate rows are excluded. An empty selection returns
zero. Unlike summarize's per-variable finite samples or tabulate's default
category-NULL exclusions, every selected row counts regardless of unrelated
NULL/nonfinite/empty values. This is not a cached estimation-sample count.

What problem it answers:
How large is the current dataset or a selected cohort without filtering data?

Supported predicates reuse bounded arithmetic/comparisons, boolean columns and
NULL. Numeric/string truthiness, function calls and broader logical expressions
are unsupported. Explicit comparisons with NULL test missingness; arithmetic
that becomes UNKNOWN is excluded and direct comparisons use native semantics.

The operation preserves data/source/schema/order, labels, named relations and
cached models/samples on success and failure. Bare count retains its ordinary
metadata path. Readable shell/--text and CLI JSON reuse CountResult; JSON's
`data.row_count` is the selected count, a lossless integer, with no new fields.

Limits: no variable list, options, assignment or grouped conditional count.
Existing `by group: count` remains supported; `by group: count if ...` does not.
No filter, model refit or saved predicate/sample is installed.

Examples:
- `count`
- `count if eligible == 1`
- `count if age >= 18`
- `count if age == null`
