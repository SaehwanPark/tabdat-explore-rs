# `generate` syntax migration evidence

Status: implementation and independent review are complete; hosted acceptance,
merge, and post-merge verification are recorded when this slice closes.

Boundary: pinned Python parser contract → backend-independent Rust syntax. This
artifact deliberately does not claim eager `generate` execution.

## Authority and recovered behavior

The Python authority is the clean sibling checkout `../tabdat-explore` at
revision `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, Python `3.13.3`, with
`uv.lock` SHA-256
`0f0e1dedbff49b4c77b470a75510b8809436c7d3dc77d1eac372ab9c7264d239`.

The recovered model/parser paths are `src/tabdat/models.py:14-101,290-292`,
`src/tabdat/parser.py:653-658,3327-3484`, and the focused examples in
`tests/test_parser.py:275-296,1334-1381`. The parser accepts a direct
`generate <target> = <expression>` form with owned identifiers, literals,
unary/binary operators, comparisons, parentheses, and function-call nodes.
Names retain exact spelling after quote unwrapping, command names normalize to
lowercase, and a trailing comma after a complete expression is accepted by the
pinned parser. The exact missing-assignment, incomplete-operator, duplicate-`if`,
option, quote, and unsupported-token diagnostics are frozen in `01-contract.md`.

The separate runtime contract remains future work: target collisions, unknown
variables, type/domain checks, arithmetic normalization and overflow,
function evaluation, lazy/materialized behavior, metadata, CLI/JSON/MCP, and
all session/backend effects are explicit deferrals.

## Oracle checks

The focused parser rerun passed at the pinned revision:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py -k 'generate or quoted_identifiers_preserves_exact_names or parse_null_literal_preserves_quoted_identifier'
5 passed, 484 deselected
```

The initial contract probe also passed `3 passed, 486 deselected` for
`tests/test_parser.py -k generate`. Broader executor/CLI/MCP recovery checks
were used to identify the later runtime boundary, not to claim Rust runtime
parity.

## Rust implementation and checks

Implementation head: `772eb58`.

- `tabdat-language` owns `Command::Generate` and a dedicated expression AST,
  including nested and multi-argument calls, without sharing or changing the
  existing `assert` expression contract.
- `tabdat-runtime` maps the new command name exhaustively but returns the
  existing `UnsupportedCommand` result. The parse→execute regression proves
  that no dataset/backend is initialized by this syntax-only command.
- No manifest, dependency, backend, FFI, unsafe code, filesystem, relation,
  session, CLI, serialization, or MCP surface changed.

Focused Rust checks passed:

```text
cargo test --locked -p tabdat-language --test parser_contract generate
2 passed
cargo test --locked -p tabdat-runtime --test use_contract leaves_generate_execution_deferred
1 passed
cargo fmt --all -- --check
git diff --check
```

The complete locked workspace baseline also passed locally:

```text
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
```

The local policy checks passed with no findings: `cargo deny check` reported
advisories/bans/licenses/sources OK, `cargo audit -D warnings` completed cleanly,
and the metadata-driven `cargo geiger` reports satisfied `forbid(unsafe_code)`
and zero first-party unsafe usage for every workspace package.

## Review and hosted evidence

Draft [PR #45](https://github.com/SaehwanPark/tabdat-explore-rs/pull/45) was
opened immediately after the contract commit `30fbfec`. Independent review
found and the implementation fixed three parser-boundary issues: the unquoted
`if` target now follows the pinned diagnostics, comparison chains are
left-associative, and an empty expression before a comma reports the assignment
expression error. The updated review found no remaining actionable findings;
see `03-review.md`.

The implementation-head hosted runs are:

- [CI policy and baseline run 35437154322](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35437154322)
  (policy and Rust baseline jobs);
- [runtime run 35437154329](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35437154329)
  (runtime Linux job).

Their final conclusions, the ready transition, merge SHA, branch cleanup, and
post-merge `main` checks must be appended before this artifact is marked
accepted. Superseded runs are not acceptance evidence.
