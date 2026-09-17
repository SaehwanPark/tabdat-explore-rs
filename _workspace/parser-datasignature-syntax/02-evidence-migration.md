# `datasignature` syntax evidence

Status: partial; implementation is pushed in draft PR #16 and awaits review
and hosted verification.

Producer: task owner

Consumer: reviewer and next maintainer

Boundary: Python parser contract → Rust syntax-only parser

Rust implementation revisions: `3cc79a0` (contract) and `a22248e` (typed
variant, dispatch, diagnostics, and tests)

Python oracle revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`)

## Inputs and authority

The contract was recovered from the pinned clean sibling checkout
`../tabdat-explore`:

- `src/tabdat/models.py:202-205` (`DatasignatureCommand`);
- `src/tabdat/parser.py:108-123,594-604` (registration and validation);
- `tests/test_datasignature.py:54-65` (positive and invalid parser cases);
- `docs/commands/datasignature.md:7-14` (public direct syntax).

The focused oracle check passed without changing the sibling checkout:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_datasignature.py -k test_parse_datasignature_form
1 passed, 10 deselected
```

The full pinned parser/script regression suite also passed without changing the
oracle checkout:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py tests/test_script.py
516 passed in 0.44s
```

Targeted probes confirmed case-insensitive command names, surrounding and
U+001C separator whitespace, the empty typed command, argument/condition/option
rejection, missing-`if` handling, assignment, unsupported `==`/`-`/`+`, and
trailing-comma diagnostics. The pinned parser accepts `by id: datasignature`
despite the command docs excluding `by:`; that wrapper and its execution
behavior remain explicitly deferred.

## Changed paths

- `crates/tabdat-language/src/lib.rs`: `Command::Datasignature`, direct dispatch,
  exact zero-argument diagnostics, and unit tests;
- `crates/tabdat-language/tests/parser_contract.rs`: public typed-command test;
- `_workspace/parser-datasignature-syntax/01-contract.md`: bounded migration
  contract;
- `README.md`, `SPEC.md`, `ARCHITECTURE.md`, and
  `docs/TABDAT_RUST_PORT_ROADMAP.md`: current-state and evidence wording.

No active relation, hashing, schema inspection, session mutation, execution,
result, serialization, CLI, filesystem, environment, statistics, or backend
dependency changed.

## Rust verification

The implementation currently passes locally:

```text
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
  root smoke: 1 passed
  tabdat-language unit tests: 18 passed
  tabdat-language integration tests: 7 passed
cargo clippy --locked --workspace --all-targets -- -D warnings
git diff --check
```

Draft PR #16 is the hosted-check authority; baseline, dependency/unsafe policy,
ReadStat, and libgretl workflows remain required before merge.

## Supported and deferred behavior

Supported here is only direct parsing: `datasignature` produces an owned,
fieldless `Command::Datasignature` with deterministic diagnostics. The parser
does not compute a SHA-256 signature, inspect active schema or row order, handle
missing/non-finite values, require a dataset, mutate session state, or render a
result.

Typed signature results, execution/state semantics, prefixed commands, terminal/
JSON/MCP output, and the complete tokenizer/option grammar remain deferred.
