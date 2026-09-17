# `use` syntax evidence

Status: implementation complete locally; PR #17 is open as a draft and awaits
independent review, hosted checks, ready-for-review transition, and merge.

Producer: task owner

Consumer: reviewers and the next maintainer

Boundary: pinned Python parser contract → Rust syntax-only parser

Rust implementation revisions: `860a534` (contract), `9086fa9` (typed command,
option tokenizer, diagnostics, and tests), `4eaa5d7` (attached `use:data`
command-boundary diagnostic fix), and `a0b842a` (Unicode numeric option-token
classification), and `b43feff` (attached comma-option boundary diagnostics).
Documentation/evidence updates are on the same draft branch.

Python oracle revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`)

## Inputs and authority

The contract was recovered from the clean pinned sibling checkout
`../tabdat-explore`:

- `src/tabdat/models.py:118-133` (`UseCommand`);
- `src/tabdat/parser.py:271-276,360-364,870-936` (routing and direct parser);
- `src/tabdat/parser.py:3159-3325,3327-3386` (option grammar/tokenizer);
- `tests/test_parser.py:90-110,1414-1425,1898-1904` (positive/invalid cases);
- `docs/commands/use.md:1-27` and `src/tabdat/help/topics/use.md:1-24`.

The focused oracle command selected the `use` positive and invalid parser cases:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py -k 'test_parse_use_command or test_parse_invalid_commands'
419 passed, 70 deselected in 0.42s
```

The full pinned parser/script regression passed without modifying the oracle:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py tests/test_script.py
516 passed in 0.46s
```

Targeted probes also confirmed raw local/URI classification, exact one-token
paths, eager/lazy defaults, engine constraints, option ordering, strict
`has_header`, delimiter forms, duplicate/unknown options, generic tokenizer
errors, and command-boundary behavior for attached comma/equal forms.

## Changed paths

- `crates/tabdat-language/src/lib.rs`: public source/mode/engine enums,
  `Command::Use`, direct dispatch, bounded option tokenizer, exact diagnostics,
  and unit tests;
- `crates/tabdat-language/tests/parser_contract.rs`: public typed-command
  coverage;
- `README.md`, `SPEC.md`, `ARCHITECTURE.md`, and
  `docs/TABDAT_RUST_PORT_ROADMAP.md`: current-state and in-progress scope;
- `_workspace/parser-use-syntax/01-contract.md`: migration boundary.

No active relation, file or URI access, named-table lookup, session mutation,
execution, result serialization, CLI, script engine, statistics, or backend
dependency changed.

## Rust verification

All required local baseline and policy checks pass on `b43feff` plus the docs
changes:

```text
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
  root smoke: 1 passed
  tabdat-language unit tests: 20 passed
  tabdat-language integration tests: 9 passed
cargo clippy --locked --workspace --all-targets -- -D warnings
git diff --check
cargo deny check
cargo audit -D warnings
metadata-driven cargo geiger (root and tabdat-language): no unsafe usage
```

Hosted CI and independent review are the acceptance authority; local success
does not yet make this a verified `main` slice.

## Supported and deferred behavior

Supported here is direct syntax only: an owned `Command::Use` records a local
path or raw URI, eager/lazy mode, optional lazy engine, delimiter, and header
flag. No format inference, path validation, file/network read, named-table
activation, relation construction, lazy planning, or runtime effect occurs.

Deferred are script and `by:` wrappers, quoted/escaped path syntax, full generic
tokenizer/option parity, session/data contracts, execution/results, reporting,
serialization, and backend capability initialization.
