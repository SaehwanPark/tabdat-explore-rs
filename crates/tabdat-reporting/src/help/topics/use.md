# use

How to invoke:
`use <path>`
`use "path with spaces,commas.csv", delimiter(<char>) has_header(true|false)`

What it does:
Load a local eager CSV or Parquet file into the active dataset slot, or reactivate a session-local named table.

What problem it answers:
Open the data you want to inspect, transform, or model.

Examples:
- `use patients.parquet`
- `use "data files/patients.parquet"`
- `use 'data files/survey,2026.csv'`
- `use "owner's survey.csv"`
- `use "semi, data.csv", delimiter(";") has_header(true)`

Notes:
- Paths are bare single tokens or enclosed in matching single/double quotes. Quotes are removed; spaces, commas, Unicode and backslashes inside are literal. The matching quote cannot occur inside the path; use the other quote where possible. No shell escape decoding or tilde expansion is performed.
- After a quoted path, only the end of input or a comma introducing options is allowed. Empty and unclosed quotes or extra paths are syntax errors.
- In a capable interactive terminal, Tab completes the first local-path argument. Start a compatible quote to complete spaces/commas/quotes; directories append a separator and keep the quote open, files close it. Existing closing quotes/options are preserved. Prefixes are case-sensitive; hidden names require a dot prefix. Suggestions do not certify loadability or include shell expansions or piped input. At a bare word end without options, literal/case-sensitive registered identifiers also join sorted/deduplicated local candidates. Registry keys refresh after partial scripts/errors without backend scans/model copies. Explicit paths, quotes and option suffixes remain filesystem-only for completion.
- `delimiter` and `has_header` are only supported for CSV files.
- `use name` reactivates a session-local named table and restores its stored row sequence. An exact registered name takes precedence over a same-name file/directory; `status` lists registered names. Completion does not change activation semantics or certify data/model compatibility.
- Remote/URI, lazy and DTA loading are not installed in the bounded runtime, even where their syntax parses. Quoted paths for other commands are not implied.

Links:
- `SPEC.md`
