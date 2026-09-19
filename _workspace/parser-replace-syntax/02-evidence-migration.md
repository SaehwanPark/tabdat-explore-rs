# `replace` syntax migration evidence

Status: accepted; implementation, independent review, hosted acceptance,
merge, branch cleanup, and merge-head verification are complete.

Boundary: pinned Python parser contract → backend-independent Rust syntax. This
artifact deliberately does not claim eager `replace` execution.

## Authority and recovered behavior

The Python authority is the clean sibling checkout `../tabdat-explore` at
revision `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, Python `3.13.3`, with
`uv.lock` SHA-256
`0f0e1dedbff49b4c77b470a75510b8809436c7d3dc77d1eac372ab9c7264d239`.

The recovered model/parser paths are
`src/tabdat/models.py:262-264,296-300`,
`src/tabdat/parser.py:127-131,646-668,3073-3147`, and
`tests/test_parser.py:275-292,1490-1503`. The direct form is
`replace <target> = <expression> [if <condition>]`; target, replacement
expression, and optional condition are retained as owned typed nodes. The
bounded diagnostics and nested function/parenthesis boundaries are frozen in
`01-contract.md`.

The focused pinned oracle check passed:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py -k 'test_parse_phase_3_generate_and_replace_commands or test_parse_invalid_commands'
419 passed, 70 deselected
```

## Rust implementation and checks

Implementation head: `1f0b945` before squash merge.

- `tabdat-language` owns `Command::Replace` and reuses the existing
  `GenerateExpression` AST for the replacement and optional condition;
- top-level unquoted `if` and option boundaries are split without treating
  nested function/parenthesized content as command structure;
- `tabdat-runtime` maps the command name exhaustively but returns the existing
  `UnsupportedCommand` result without initializing a backend or active data;
- no manifest, dependency, native backend, FFI, unsafe code, filesystem,
  relation, CLI, serialization, or MCP surface changed.

Focused Rust checks passed:

```text
cargo test --locked -p tabdat-language --test parser_contract replace
2 passed
cargo test --locked -p tabdat-runtime --test use_contract replace_execution_remains_deferred_without_initializing_a_backend
1 passed
```

The complete locked workspace baseline and policy checks also passed locally:

```text
cargo fmt --all -- --check                         passed
cargo check --locked --workspace --all-targets    passed
cargo test --locked --workspace --all-targets     passed
cargo clippy --locked --workspace --all-targets -- -D warnings
                                                     passed
git diff --check                                  passed
cargo deny check                                  passed
cargo audit -D warnings                           passed
metadata-driven cargo geiger loop                 passed
```

## Hosted PR-head evidence

Draft [PR #47](https://github.com/SaehwanPark/tabdat-explore-rs/pull/47) was
opened at the contract boundary, marked ready after review, and its final
implementation head `1f0b945` passed:

- [CI workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35445905834),
  including [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35445905834/job/105904698700)
  and [dependency/unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35445905834/job/105904698591);
- [TabDat runtime boundary](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35445905910)
  ([Linux job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35445905910/job/105904661911)).

## Merge, cleanup, and merge-head evidence

PR #47 was squash-merged as
[`87ec017`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/87ec0173aae4c3e2cb8b27d1b26295aaf17cd18f)
with `--delete-branch`. The local and remote
`feat/parser-replace-syntax` refs were absent after the merge.

The merge-head workflows passed:

- [post-merge CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35447004384),
  including [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35447004384);
- [post-merge runtime boundary](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35447004424).

## Deferrals

The Rust runtime intentionally does not claim relation mutation, schema/type
validation, predicate truthiness, missing/non-finite arithmetic, overflow
accounting, panel/label metadata, lazy/materialized execution,
`last_operation`, transform sequencing, CLI, JSON, MCP, or full tokenizer and
expression parity. These remain explicit future slices rather than hidden
behavior.
