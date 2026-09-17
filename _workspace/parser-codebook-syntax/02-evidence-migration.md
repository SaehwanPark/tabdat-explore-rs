# `codebook` syntax evidence

Status: accepted and merged in PR #18 (`1efc991`); all required hosted checks
passed and the temporary branch was deleted.

Producer: task owner

Consumer: reviewers and the next maintainer

Boundary: pinned Python parser contract → Rust syntax-only parser

Rust implementation revisions: `982c3c5` (contract), `f105d27` (typed command,
dispatch, bounded argument tests), `0816828` (reject unsupported punctuation in
the simple argument path), `7a87dfb` (Python-compatible quote boundaries),
`a2ddc72` (empty adjacent quote fragments), and `1ec1347` (unquoted `if`
boundary before a backtick fragment). The implementation and documentation
were squash-merged as PR #18 at `1efc991`.

Python oracle revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`), Python 3.13.3. The clean sibling
checkout and lock digest remain those recorded in `docs/migration/README.md`.

## Inputs and authority

The contract was recovered from the clean pinned sibling checkout
`../tabdat-explore`:

- `src/tabdat/models.py:168-176` (`CodebookCommand`);
- `src/tabdat/parser.py:271-305,567-572` (routing and command construction);
- `src/tabdat/parser.py:3040-3158` (generic argument/condition/option/
  assignment parsing);
- `tests/test_parser.py:205-207,1476-1477` (positive and invalid forms);
- `docs/commands/codebook.md:1-22` and
  `src/tabdat/help/topics/codebook.md:1-14` (public syntax/examples).

The focused oracle check passed:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py::test_parse_phase_3_inspection_commands
1 passed in 0.27s
```

The full pinned parser/script regression passed without modifying the oracle:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py tests/test_script.py
516 passed in 0.44s
```

Additional probes covered mixed-case/TabDat-control whitespace, ordered and
quoted/backtick variable names, condition/option/assignment rejection, missing
`if` expressions, trailing commas, unsupported punctuation, and assignment
target diagnostics.

## Changed paths

- `crates/tabdat-language/src/lib.rs`: owned `Command::Codebook`, direct
  dispatch, bounded argument diagnostics, and unit tests;
- `crates/tabdat-language/tests/parser_contract.rs`: public typed-command and
  exact-diagnostic coverage;
- `_workspace/parser-codebook-syntax/01-contract.md`: recovered migration
  boundary;
- this file: verification and deferral record.

No active relation, schema lookup, filesystem access, session mutation,
execution, result serialization, CLI, script engine, statistics, or backend
dependency changed.

## Rust verification

The merged implementation passed the local checks below before PR #18 was marked
ready:

```text
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
  root smoke: 1 passed
  tabdat-language unit tests: 22 passed
  tabdat-language integration tests: 11 passed
cargo clippy --locked --workspace --all-targets -- -D warnings
git diff --check
cargo deny check
cargo audit -D warnings
metadata-driven cargo geiger (root and tabdat-language): no unsafe usage
```

The final hosted checks passed on the PR head before merge:

- [Rust baseline and dependency/unsafe policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35212530397);
- [ReadStat feasibility spike](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35212530584);
- [libgretl feasibility spike](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35212530363);
- [libgretl OLS Rust spike](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35212530418).

Independent parser, contract, and workspace reviews reported no open findings.

## Supported and deferred behavior

Supported here is direct syntax only: an owned `Command::Codebook` records an
ordered variable list, including unwrapped quoted/backtick names, and rejects
the bounded condition/option/assignment and unsupported-punctuation forms with
the pinned diagnostics. No variable existence, type, wildcard, or range
validation occurs.

Deferred are active-dataset/schema semantics, wildcard/range expansion,
conditions and options, `by:` wrappers, full tokenizer/varlist/option and
expression grammar, script execution, results/reporting/serialization, and
backend capability initialization.
