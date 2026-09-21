# Bounded syntax-only `sql` command contract

Status: contract checkpoint; implementation and validation are in progress.

Producer: task owner, using `tabdat-migration`, `simple-code-writer`, and
`preferred-workflow`.
Consumer: the bounded Rust language/parser implementation and its focused review.

## Scope

Add the backend-independent parser boundary for the already documented
`sql <query> [into <table>]` command. The parser owns the SQL text as an
opaque string and an optional validated named-table target; it does not inspect
schemas, initialize DuckDB, execute SQL, or publish session state.

Supported forms in this slice:

- one-line SQL after the command name, preserving query text except for outer
  command whitespace;
- a triple-quoted SQL body (`sql """..."""`) with the first closing delimiter
  ending the query; and
- an optional trailing `into <table>` clause, with case-insensitive `into` and
  the existing named-table identifier/reserved-name rules.

Script-level multiline grouping, SQL execution, named-table lifecycle,
schema/type validation, result rendering, CLI/JSON/MCP surfaces, and broader
SQL grammar are explicitly deferred.

## Python contract

Pinned authority:

- repository: `https://github.com/SaehwanPark/tabdat-explore.git`;
- revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`;
- tree: `601b236788872323af9277d2276a236154a0f129`;
- package: `0.25.0`; and
- `uv.lock` SHA-256:
  `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.

Relevant authority paths:

- `src/tabdat/parser.py:939-994` — SQL query, triple-quoted body, trailing
  `into`, and table-name validation;
- `src/tabdat/models.py:379-382` — `SqlCommand(query, into)` shape; and
- `tests/test_parser.py:338-365` plus the malformed-command matrix around
  `1518-1523` — accepted and rejected forms.

The Python parser treats a final whitespace-delimited `into <word>` pair as
the target, preserves other SQL text opaquely, trims the query, and reports
stable diagnostics for missing queries, unclosed triple quotes, malformed
`into`, non-identifiers, and reserved targets.

## Rust contract

Add an owned `SqlCommand { query: String, into: Option<String> }` and a
`Command::Sql { command: SqlCommand }` variant. `parse_command` must return the
typed command for direct and triple-quoted forms and the exact bounded
diagnostics above. The runtime must recognize the command name but continue to
return its existing typed `UnsupportedCommand { name: "sql" }` error; no backend
or session behavior changes in this slice.

## Test contract

Focused parser coverage must include:

- direct query with punctuation and repeated spaces;
- direct query with and without `into`, including case-insensitive `into`;
- triple-quoted query with preserved internal newlines and trailing `into`;
- empty query, missing closing delimiter, incomplete/malformed `into`,
  reserved targets, and invalid identifier targets; and
- runtime deferral returning `UnsupportedCommand { name: "sql" }` without
  initializing DuckDB.

The pinned Python probe is:

```sh
cd ../tabdat-explore
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py -k 'phase_4_sql_commands'
```

## State and deferrals

Parsing is pure and backend-independent; all runtime/session state remains
unchanged. SQL result relations, named-table persistence, script multiline
grouping, and broader SQL/query parity are deferred to later roadmap slices.

Completion remains `partial` until implementation, independent review, hosted
acceptance, merge, and post-merge evidence are recorded.
