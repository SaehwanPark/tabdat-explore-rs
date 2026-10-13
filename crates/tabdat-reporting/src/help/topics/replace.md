# replace

How to invoke:
`replace varname = <expression> [if <condition>]`

What it does:
Update an existing numeric or string column, optionally for rows satisfying a boolean condition.

Examples:
- `replace cost = cost / 2 if age >= 42`
- `replace sex = 'X' if sex == 'F'`
- `replace cost = null if age == 30`
- `replace cost = 0 if cost == null`

Notes:
- Requires an active dataset and existing target. Numeric arithmetic, string values/columns and explicit NULL replacements are supported within the target domain; numeric/string conversion is not implicit. Function calls (including sqrt/ln/log) remain unsupported even where syntax parses.
- Conditions must produce boolean or NULL values; numeric/string truthiness is rejected. False/NULL conditions leave those row values unchanged. `== null`/`!= null` use the installed explicit missingness-comparison rule.
- Existing expression-safety/native-type rules remain unchanged. Row count/order and schema position are preserved, but inferred numeric type/width can change. No affected-row or overflow-row count is supplied by the owned result.
- CLI-v1 JSON returns `ReplaceResult` with `data.dataset` using the existing metadata shape. `row_count` is total relation size, not the number of changed rows. Use `head`/`tail` for actual values; label/panel fields retain null-placeholder semantics.
- Invalid/unknown/unsupported/type failures preserve prior rows and session state. A successful replacement removes the target's value-label attachment under existing rules; variable labels and value-label definitions are retained. The prior fitted model stays cached; replacement does not refit it automatically.
- Completed mutation is not rolled back by later output failure; later commands are stopped (MIG-0008).

Links:
- `SPEC.md`
