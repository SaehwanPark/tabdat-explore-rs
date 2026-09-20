# Bounded append syntax contract

## Status and scope

- Status: `in progress` at the contract checkpoint.
- Owner: the parent task owner; this slice is intentionally parser-only.
- Target: recover and port the direct `append` command boundary without
  claiming named-table execution.
- Rust paths in scope: `crates/tabdat-language/src/lib.rs`, its parser tests,
  the runtime unsupported-command contract, and this evidence directory.
- Runtime paths are otherwise out of scope. `tabdat-runtime` will continue to
  return an explicit typed unsupported-command error for `Command::Append`
  until named-table state and SQL/table creation are separately implemented.

The accepted form is:

    append <table>

The parser produces an owned typed command containing one named table. The
table must satisfy the oracle's SQL table-name validation; `active` and names
beginning with `__tabdat_` remain reserved.

## Authority and evidence inputs

- Python oracle checkout: `C:\Users\saehwan\repos\tabdat-python-oracle`.
- Pinned revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`.
- Pinned tree: `601b236788872323af9277d2276a236154a0f129`.
- Lockfile SHA-256: `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.
- Oracle sources: `src/tabdat/models.py` (`AppendCommand`),
  `src/tabdat/parser.py` (`_parse_append` and table-name validation), and
  `docs/commands/append.md`.
- Oracle tests: `tests/test_parser.py`, including the positive append form and
  malformed arity, option, condition, reserved-name, and identifier cases.

The parser-focused append selection is the differential checkpoint. Executor
append tests are recorded as deferred evidence because they require the Python
named-table and SQL state model that does not yet exist in this Rust workspace.

## Rust contract

1. `parse_command` recognizes `append` case-insensitively and returns
   `Command::Append { table_name }`.
2. Exactly one table argument is required. Quoted and backtick identifier
   spellings retain their decoded table text.
3. The table token must be a valid non-reserved SQL table identifier. `active`
   and names beginning with `__tabdat_` remain reserved.
4. Options, conditions, assignment syntax, and extra arguments are rejected
   with the bounded `append expects syntax: append <table>` diagnostic.
5. Parsing does not inspect files, initialize DuckDB, or mutate session state.
6. Runtime execution is explicitly deferred; no named-table registry or hidden
   backend relation is introduced by this parser slice.

## Acceptance checks

- Oracle: focused `tests/test_parser.py -k append` passes at the pinned
  revision.
- Rust: focused language unit/integration parser tests cover the positive form,
  case/quote handling, malformed boundaries, reserved names, and invalid table
  identifiers.
- Runtime: a focused contract test proves parsed append returns the typed
  unsupported-command error without backend work.
- Repository baseline: format, locked workspace check/test, and clippy with
  warnings denied pass before the PR is marked ready.
- Documentation and `git diff --check` pass; only this slice's files are
  staged.

## Known gaps and next dependency

This slice does not implement SQL, named-table creation/activation, the session
named-table registry, schema compatibility checks, column union/type and
missingness rules, row ordering/publication, labels, lazy execution, or
CLI/JSON/MCP/reporting adapters. Those behaviors remain future append-runtime
evidence, gated by the named-table and SQL roadmap work. The parser can be
merged without implying that a parsed append command executes today.
