# Bounded join syntax contract

## Status and scope

- Status: `accepted` after PR #59 and its merge-head workflows passed.
- Owner: the parent task owner; this slice is intentionally parser-only.
- Target: recover and port the direct `join` command boundary without claiming
  named-table execution.
- Rust paths in scope: `crates/tabdat-language/src/lib.rs`, its parser tests,
  and this evidence directory.
- Runtime paths are out of scope for this slice. `tabdat-runtime` will continue
  to return its existing typed unsupported-command error for `Command::Join`
  until named-table state and SQL/table creation are separately implemented.

The accepted form is:

    join <table> on <keylist> [, how=inner|left suffix(_right)]

The parser produces an owned typed command containing the named table, an
ordered non-empty key list, a typed `inner` or `left` join mode, and a non-empty
right-column suffix. Defaults are `how=inner` and `suffix(_right)`. Table names
follow the oracle's SQL table-name validation; key names may be quoted
identifiers and retain their decoded text.

## Authority and evidence inputs

- Python oracle checkout: `C:\Users\saehwan\repos\tabdat-python-oracle`.
- Pinned revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`.
- Pinned tree: `601b236788872323af9277d2276a236154a0f129`.
- Lockfile SHA-256: `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.
- Oracle sources: `src/tabdat/models.py` (`JoinCommand`),
  `src/tabdat/parser.py` (`_parse_join` and table-name validation), and
  `docs/commands/join.md`.
- Oracle tests: `tests/test_parser.py`, including the positive join forms,
  malformed-command matrix, duplicate-key, option, mode, and suffix checks.

The oracle's parser-focused join selection is the initial differential
checkpoint. Its executor join tests are recorded as deferred evidence rather
than treated as a Rust pass because they require the Python named-table and SQL
state model that does not yet exist in this Rust workspace.

## Rust contract

1. `parse_command` recognizes `join` case-insensitively and returns
   `Command::Join { command: JoinCommand { ... } }`.
2. The table token must be a valid non-reserved SQL table identifier. `active`
   and names beginning with `__tabdat_` remain reserved.
3. The unquoted `on` separator is required, followed by at least one key.
   Key order is retained and duplicate keys are rejected before execution.
4. Only `how` and `suffix` options are accepted. Each appears at most once;
   `how` accepts only `inner` or `left`, and `suffix` must be non-empty.
5. Conditions and assignment syntax are rejected with the bounded join syntax
   diagnostic. Malformed options retain deterministic parser diagnostics.
6. Parsing does not inspect files, initialize DuckDB, or mutate session state.
7. Runtime execution is explicitly deferred; no named-table registry or hidden
   backend relation is introduced by this parser slice.

## Acceptance checks

- Oracle: focused `tests/test_parser.py -k join` passes at the pinned revision.
- Rust: focused language unit/integration parser tests cover positive forms,
  quoted keys, defaults, duplicate keys, malformed boundaries, options, and
  reserved/invalid table names.
- Repository baseline: format, locked workspace check/test, and clippy with
  warnings denied pass before the PR is marked ready.
- Documentation and `git diff --check` pass; only this slice's files are staged.

The contract checkpoint was committed as
[`c9cb448ff3d781a2f8ccb19f7f0adb6892bbfca6`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/c9cb448ff3d781a2f8ccb19f7f0adb6892bbfca6).
The implementation was committed as
[`dfa1ecc72a99f8dac42170c4bc38cf840886bb74`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/dfa1ecc72a99f8dac42170c4bc38cf840886bb74)
and merged to `main` as
[`585c53fcdce135456abded9b27df75fbcbcda8cf`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/585c53fcdce135456abded9b27df75fbcbcda8cf).

## Known gaps and next dependency

This slice does not implement SQL, named-table creation/activation, the session
named-table registry, type-compatible key validation, null-key join semantics,
collision-free right-column naming, stable active/match ordering, relation
publication, labels, lazy execution, or CLI/JSON/MCP/reporting adapters. Those
behaviors remain future join-runtime evidence, gated by the named-table and SQL
roadmap work. The parser can therefore be merged without implying that a parsed
join command executes today.
