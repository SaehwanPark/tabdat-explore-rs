# Bounded syntax-only `sql` command review

Reviewer: task owner and independent review verification
Scope: PR #77 implementation, AST models, parser routing, helper functions,
and focused contract tests.

## Findings and resolution

The initial implementation was reviewed against the pinned Python authority
and acceptance criteria:

- `SqlCommand { query, into }` is an owned AST struct and `Command::Sql` is an
  owned variant;
- direct queries and multiline triple-quoted queries (`"""..."""`) are parsed
  faithfully;
- case-insensitive trailing `into <table>` clause is recognized and validated
  against identifier and reserved-name rules (`active`, `__tabdat_*`);
- query text is preserved opaquely with outer whitespace stripped;
- Clippy's `collapsible_if` lint was addressed using `is_some_and`;
- runtime execution deferral returns typed `UnsupportedCommand { name: "sql" }`
  without initializing DuckDB or backend session state;
- focused unit tests in `tabdat-language` and integration tests in
  `tabdat-runtime` verify exact diagnostics matching the Python oracle.

No security, safety, or semantic finding remains.

## Review conclusion

No P0/P1 or lower-severity implementation findings remain. All workspace tests
pass, Clippy with `-D warnings` is clean, formatting passes, and policy checks
succeed.

## Evidence reviewed

- pinned Python authority and focused execution probe;
- `01-contract.md` and `02-evidence-migration.md`;
- `crates/tabdat-language/src/lib.rs`;
- `crates/tabdat-runtime/src/lib.rs`;
- `crates/tabdat-runtime/tests/sql_contract.rs` and `use_contract.rs`;
- local locked workspace, policy, and diff checks.
