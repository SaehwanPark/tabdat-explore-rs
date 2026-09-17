# `duplicates` syntax evidence

Status: draft; implementation and verification are in progress on PR #20.

Producer: task owner

Consumer: reviewers and the next maintainer

Boundary: pinned Python parser contract → Rust syntax-only parser

Rust implementation revisions: `62c9270` (contract and in-progress state
documentation) and `a70bba0` (typed command, dispatch, bounded diagnostics,
quote tracking, and tests).

Python oracle revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`), Python 3.13.3. The clean sibling
checkout and lock digest remain those recorded in `docs/migration/README.md`.

## Inputs and authority

The contract was recovered from the clean pinned sibling checkout
`../tabdat-explore`:

- `src/tabdat/models.py:186-190` (`DuplicatesCommand`);
- `src/tabdat/parser.py:271-305,581-589` (routing and command construction);
- `src/tabdat/parser.py:3040-3158` (generic tokenization and diagnostics);
- `tests/test_duplicates.py:67-78` (focused parser cases);
- `docs/commands/duplicates.md:1-42` (public syntax and deferred report
  semantics).

The focused oracle check passed:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_duplicates.py -k 'parse_duplicates_forms'
1 passed, 13 deselected in 0.27s
```

The full pinned parser/script regression passed without modifying the oracle:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py tests/test_script.py
516 passed in 0.46s
```

Additional probes covered case/control whitespace, ordered variables, the
optional leading `report` alias, quoted-string stripping, backtick-quoted
`report` retention, doubled-backtick names, condition/option/assignment
rejection, missing `if` expressions, trailing commas, and unsupported
`==`/`-`/`+`/`!`/`@` tokens.

## Changed paths

- `crates/tabdat-language/src/lib.rs`: owned `Command::Duplicates`, direct
  dispatch, bounded alias/diagnostics, and unit tests;
- `crates/tabdat-language/tests/parser_contract.rs`: public typed-command and
  exact-diagnostic coverage;
- `_workspace/parser-duplicates-syntax/01-contract.md`: recovered migration
  boundary;
- `_workspace/parser-duplicates-syntax/03-review.md`: independent review
  record;
- this file: verification and deferral record;
- `README.md`, `ARCHITECTURE.md`, `SPEC.md`, and
  `docs/TABDAT_RUST_PORT_ROADMAP.md`: current in-progress slice references.

No active relation, schema lookup, filesystem access, session mutation,
execution, result serialization, CLI, script engine, statistics, or backend
dependency changed.

## Rust verification

The current implementation passed the local checks below after `a70bba0`:

```text
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
  root smoke: 1 passed
  tabdat-language unit tests: 26 passed
  tabdat-language integration tests: 15 passed
cargo clippy --locked --workspace --all-targets -- -D warnings
git diff --check
cargo deny check
cargo audit -D warnings
metadata-driven cargo geiger (root and tabdat-language): no unsafe usage
```

Independent parser, contract, and workspace reviews report no actionable
finding at the current implementation head. The hosted Rust baseline, ReadStat,
and libgretl workflows are acceptance gates for the current PR head; their
final results remain pending after the evidence and review artifacts are
pushed.

## Supported and deferred behavior

Supported here is direct syntax only: an owned `Command::Duplicates` records an
ordered variable list, strips a first unquoted/string `report` alias according
to the pinned Python behavior, preserves backtick-quoted `report` as a
variable, and rejects the bounded condition/option/assignment and
unsupported-punctuation forms with the pinned diagnostics.

Deferred are active-relation/schema semantics, duplicate grouping and counts,
null-key behavior, variable existence, wildcard/range expansion, conditions
and options, `by:` wrappers, full tokenizer/varlist/option and expression
grammar, script execution, results/reporting/serialization, and backend
capability initialization.
