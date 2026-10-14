# by

How to invoke:
`by groupvars: count` or the existing non-JSON grouped-mean form `by groupvars: summarize [numeric_vars]`.

What it does:
Run the installed read-only grouped aggregate, not an arbitrary command runner. CLI-v1 JSON admits only `count` as the child command.

What problem it answers:
How do I count observations in subgroups before a supported analysis/reporting workflow?

Examples:
- `by sex: count`
- `by firm_id year: count`
- `by sex: summarize age` (non-JSON native grouped means)

Notes:
- Group names must exist. One/multiple/backtick-quoted/repeated groups are supported for counts; headers preserve requested order and duplicates, followed by Count. Native group order is ascending NULLS LAST for each key.
- COUNT(*) includes every observation, including SQL NULL-key groups. Stored empty strings are separate nonmissing keys; no finite-value filter or general missingness normalization is installed. Empty datasets yield headers with no rows, not an invented zero-count group.
- Raw native keys are reported, not substituted value labels. Existing CLI-v1 scalar policy applies: exact decimal strings, losslessly parsed wide integers, base64-prefixed bytes, and nonfinite keys encoded as null. Native NaN and SQL NULL groups can consequently have indistinguishable encoded keys while remaining distinct counted rows; reporting does not combine them.
- JSON returns `ByResult` with owned `data.headers`/`rows`, not dataset/group diagnostics or a rescan. Counting/serialization retains data/schema/source/order, labels, named state and cached fit/sample.
- Unknown/no-active groups fail; unsupported children such as summarize/tabulate/sort are rejected before JSON execution. Nested by commands are rejected by the parser; constructed nested commands are also outside JSON admission. Later effects/files stop on failure; completed exports survive later output failure (MIG-0008).
- The existing non-JSON summarize form returns native grouped AVG means, not full descriptive summaries. Its SQL-NULL/native nonfinite semantics must not be inferred to equal ordinary per-variable finite summarize samples. Grouped-mean JSON admission is deferred for separate statistical review. No general by runner, mutation, lazy/remote execution or whole-command parity is implied.
