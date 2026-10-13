# generate

How to invoke:
`generate new_name = <expression>`

What it does:
Append a numeric column to the active dataset using the bounded eager expression evaluator.

Examples:
- `generate doubled = age * 2`
- `generate centered = age - 40`
- ``generate `age value` = age + 1``

Notes:
- Requires an active dataset and a new target name. Supported expressions are numeric columns/literals, unary minus, parentheses and arithmetic `+`, `-`, `*`, `/`.
- String/explicit NULL expressions, comparisons and function calls (including `log`) may parse but are not installed for generate. Unknown columns and type mismatches fail without changing prior data/session state.
- NULL input values propagate through arithmetic; output uses the existing numerical-safety/compiler and native-type policies. No new precision, overflow-count or function-family support is implied.
- CLI-v1 JSON returns `GenerateResult` with `data.dataset` in the existing metadata shape: source path, total relation row count and current ordered native schema. The new column is appended. This is metadata, not cell values, affected-row counts or overflow diagnostics; use `head`/`tail` to inspect values. Label/panel fields remain null placeholders.
- Existing source/order/row count and prior fitted model are retained; generating a feature does not automatically refit it. Successful effects are not rolled back by later output failure (MIG-0008).

Links:
- `SPEC.md`
