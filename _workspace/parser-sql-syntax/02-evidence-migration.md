# Bounded syntax-only `sql` command migration evidence

Status: implementation and local validation complete; hosted PR checks and
post-merge evidence will be appended after acceptance.

Producer: task owner
Consumers: independent reviewer, PR reviewers, and the next maintainer

Boundary: pinned Python SQL parser behavior → Rust bounded syntax-only `sql`
command. The bounded contract is in [`01-contract.md`](01-contract.md).

## Authority and contract inputs

The Python oracle is the clean sibling checkout `../tabdat-explore` at pinned
revision `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, package `0.25.0`, Python `3.13.3`,
and `uv.lock` SHA-256
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.
The checkout was clean; no dependency synchronization or source edits were
performed.

The execution authority is:

- `src/tabdat/parser.py:939-994` for SQL query, triple-quoted body, trailing
  `into`, and table-name validation;
- `src/tabdat/models.py:379-382` for `SqlCommand(query, into)` AST model; and
- `tests/test_parser.py:338-365` and `1518-1523` for accepted and rejected
  SQL commands.

## Rust revisions and changed paths

- `9b2cc30` — contract checkpoint and draft PR [#77](https://github.com/SaehwanPark/tabdat-explore-rs/pull/77);
- `6085fda` — bounded SQL parser boundary, helper functions, and focused tests;
- hosted PR workflows on `6085fda`: [CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35632824442) and [TabDat runtime boundary](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35632824485) passed.

Changed paths:

- `crates/tabdat-language/src/lib.rs` — `SqlCommand` AST struct, `Command::Sql`
  variant, `parse_sql_command`, `parse_triple_quoted_sql`, `split_sql_into`,
  `parse_sql_into_remainder`, `split_whitespace_words`, delimiter/colon checks,
  and unit tests;
- `crates/tabdat-runtime/src/lib.rs` — `Command::Sql` handling in `command_name`;
- `crates/tabdat-runtime/tests/sql_contract.rs` — dedicated deferred execution
  contract test;
- `crates/tabdat-runtime/tests/use_contract.rs` — runtime execution deferral test.

No dependency, unsafe code, FFI, or backend selection changed.

## Pinned oracle probes

Focused Python parser probe:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py -k 'phase_4_sql_commands'
```

Observed: `1 passed, 488 deselected`.

Focused Python error behavior:
- `sql` -> `ParseError: sql expects a query`
- `sql """select * from active` -> `ParseError: sql multiline query is missing closing """`
- `sql select * from active into` -> `ParseError: sql into expects syntax: sql <query> into <table>`
- `sql select * from active into active` -> `ParseError: sql into cannot use reserved table name: active`
- `sql select * from active into __tabdat_next` -> `ParseError: sql into cannot use reserved table name: __tabdat_next`
- `sql select * from active into bad-name` -> `ParseError: sql into table name must be an identifier`

## Rust implementation evidence

Focused commands:

```sh
cargo test --locked -p tabdat-language parses_valid_sql_syntax
cargo test --locked -p tabdat-language rejects_invalid_sql_syntax_with_exact_diagnostics
cargo test --locked -p tabdat-runtime --test sql_contract
```

Observed: all passed.

Workspace checks:

```sh
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
git diff --check
cargo deny check
cargo audit -D warnings
```

Observed locally: all formatting, checks, tests, Clippy (-D warnings), diff,
`cargo deny`, and `cargo audit` checks passed cleanly.
