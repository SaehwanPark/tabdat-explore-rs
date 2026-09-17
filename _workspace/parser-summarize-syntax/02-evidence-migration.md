# `summarize` syntax evidence

Status: draft; implementation and verification are in progress on PR #21.

Producer: task owner

Consumer: reviewers and the next maintainer

Boundary: pinned Python parser contract → Rust syntax-only parser

Rust implementation revisions: `9b1bedb` (contract and in-progress state
documentation), `e100c79` (typed command, dispatch, bounded diagnostics, and
tests), and `2c1aaa2` (quote-boundary tests).

Python oracle revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`), Python 3.13.3. The clean sibling
checkout and lock digest remain those recorded in `docs/migration/README.md`.

## Inputs and authority

The contract was recovered from the clean pinned sibling checkout
`../tabdat-explore`:

- `src/tabdat/models.py:157-165` (`SummarizeCommand`);
- `src/tabdat/parser.py:271-305,555-572` (routing and direct/structured command
  construction);
- `src/tabdat/parser.py:3040-3158` (generic tokenization and diagnostics);
- `tests/test_parser.py:191-202` (direct parser cases);
- `tests/test_parser.py:218-238` (structured `if`/option forms);
- `docs/commands/summarize.md:1-25` (public syntax and examples);
- `src/tabdat/help/topics/summarize.md:1-17` (in-app syntax/help text).

The focused direct oracle check passed:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py::test_parse_summarize_command_with_variables \
  tests/test_parser.py::test_parse_summarize_preserves_punctuated_variable_names \
  tests/test_parser.py::test_parse_summarize_command_without_variables
3 passed in 0.29s
```

The full pinned parser/script regression passed without modifying the oracle:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py tests/test_script.py
516 passed in 0.47s
```

Additional probes covered case/control whitespace, ordered and duplicate
variables, quoted strings, backtick names and doubled backticks, missing `if`
expressions, trailing commas, assignments, and unsupported `==`/`-`/`+`/`!`/`@`
tokens.

Python accepts `summarize age if age >= 18` and
`summarize age bmi, detail limit=10` as structured `ParsedCommand` values. The
Rust slice intentionally rejects those forms with the bounded direct-command
diagnostics because its generic expression and option AST is not yet present.
This is an explicit scope deviation, not a claim of full summarize parity.

## Changed paths

- `crates/tabdat-language/src/lib.rs`: owned `Command::Summarize`, direct
  dispatch, bounded diagnostics, and unit tests;
- `crates/tabdat-language/tests/parser_contract.rs`: public typed-command and
  exact-diagnostic coverage;
- `_workspace/parser-summarize-syntax/01-contract.md`: recovered migration
  boundary;
- `_workspace/parser-summarize-syntax/03-review.md`: independent review
  record;
- this file: verification and deferral record;
- `README.md`, `ARCHITECTURE.md`, `SPEC.md`, and
  `docs/TABDAT_RUST_PORT_ROADMAP.md`: current in-progress slice references.

No active relation, schema lookup, filesystem access, session mutation,
execution, result serialization, CLI, script engine, statistics, or backend
dependency changed.

## Rust verification

The current implementation passed the local checks below after `2c1aaa2`:

```text
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
  root smoke: 1 passed
  tabdat-language unit tests: 28 passed
  tabdat-language integration tests: 17 passed
cargo clippy --locked --workspace --all-targets -- -D warnings
git diff --check
cargo deny check
cargo audit -D warnings
metadata-driven cargo geiger (root and tabdat-language): no unsafe usage
```

The independent parser, contract, and workspace review records report no
actionable implementation finding; the contract pass's missing-artifact finding
is resolved by this file and `03-review.md`. Hosted Rust baseline, ReadStat, and
libgretl workflows are acceptance gates for the final evidence revision and
remain pending until this record is pushed.

## Supported and deferred behavior

Supported here is direct syntax only: an owned `Command::Summarize` records an
ordered variable list, including unwrapped quoted/backtick names and duplicate
names, and rejects the bounded condition/option/assignment and
unsupported-punctuation forms with the recorded diagnostics.

Deferred are active-relation/schema semantics, numeric-column validation,
summary statistics, missingness rules, conditions and options, `by:` wrappers,
full tokenizer/varlist/option and expression grammar, script execution,
results/reporting/serialization, and backend capability initialization.
