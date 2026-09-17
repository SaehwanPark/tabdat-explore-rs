# `missing` syntax evidence

Status: draft; implementation and verification are in progress on PR #19.

Producer: task owner

Consumer: reviewers and the next maintainer

Boundary: pinned Python parser contract → Rust syntax-only parser

Rust implementation revisions: `e342b75` (contract and in-progress state
documentation) and `3635131` (typed command, dispatch, bounded diagnostics, and
tests).

Python oracle revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`), Python 3.13.3. The clean sibling
checkout and lock digest remain those recorded in `docs/migration/README.md`.

## Inputs and authority

The contract was recovered from the clean pinned sibling checkout
`../tabdat-explore`:

- `src/tabdat/models.py:179-183` (`MissingCommand`);
- `src/tabdat/parser.py:271-305,574-579` (routing and direct parser);
- `src/tabdat/parser.py:3040-3158` (generic argument/condition/option/
  assignment parsing);
- `tests/test_missing.py:16-28` (focused parser cases);
- `docs/commands/missing.md:1-30` (public syntax and deferred report semantics).

The focused oracle check passed:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_missing.py -k 'parse_missing'
2 passed, 7 deselected
```

The full pinned parser/script regression passed without modifying the oracle:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py tests/test_script.py
516 passed in 0.47s
```

Additional probes covered mixed-case/control whitespace, ordered and
quoted/backtick variable names, condition/option/assignment rejection, missing
`if` expressions, trailing commas, and unsupported `==`/`-`/`+`/`!`/`@` tokens.

## Changed paths

- `crates/tabdat-language/src/lib.rs`: owned `Command::Missing`, direct dispatch,
  bounded diagnostics, and unit tests;
- `crates/tabdat-language/tests/parser_contract.rs`: public typed-command and
  exact-diagnostic coverage;
- `_workspace/parser-missing-syntax/01-contract.md`: recovered migration
  boundary;
- this file: verification and deferral record.

No active relation, schema lookup, filesystem access, session mutation,
execution, result serialization, CLI, script engine, statistics, or backend
dependency changed.

## Rust verification

The current implementation passed the local checks below after `3635131`:

```text
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
  root smoke: 1 passed
  tabdat-language unit tests: 24 passed
  tabdat-language integration tests: 13 passed
cargo clippy --locked --workspace --all-targets -- -D warnings
git diff --check
```

Dependency and unsafe-code policy scans, independent parser/contract/workspace
review, and hosted CI remain acceptance gates. Local success alone does not
close this migration slice.

## Supported and deferred behavior

Supported here is direct syntax only: an owned `Command::Missing` records an
ordered variable list, including unwrapped quoted/backtick names, and rejects
the bounded condition/option/assignment and unsupported-punctuation forms with
the pinned diagnostics.

Deferred are active-relation/schema semantics, null-count and percentage rules,
schema-order behavior, unknown-variable errors, wildcard/range expansion,
conditions and options, `by:` wrappers, full tokenizer/varlist/option and
expression grammar, script execution, results/reporting/serialization, and
backend capability initialization.
