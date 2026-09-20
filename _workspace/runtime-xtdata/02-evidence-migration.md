# `xtdata` syntax migration evidence

## Status and boundary

Status: `complete`.

This is the accepted parser-only `xtdata` slice. It ports the direct command
boundary and typed ownership contract; it does not claim panel metadata,
within/between relation transforms, or active-session execution. Runtime
execution remains an explicit unsupported-command result until panel metadata
and relation-publication semantics are separately recovered and tested.

## Authority and recovered behavior

The Python oracle is the isolated checkout at
`C:\Users\saehwan\repos\tabdat-python-oracle`, pinned to revision
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`). Its `uv.lock` SHA-256 is
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.
The relevant sources are `src/tabdat/models.py` (`XtDataCommand`),
`src/tabdat/parser.py` (`_parse_xtdata`), and `docs/commands/xtdata.md`.
The focused parser command was:

    uv run --no-sync pytest -q -p no:cacheprovider tests/test_parser.py -k xtdata

It passed with `7 passed, 482 deselected`. The recovered parser accepts one or
more variables followed by a comma and exactly one unquoted flag, `within` or
`between`. Both transforms, neither transform, unsupported options, valued
flags, conditions, assignment syntax, and malformed variable boundaries are
rejected with bounded diagnostics. Repeated copies of one accepted flag retain
the oracle's set-membership behavior.

## Rust implementation

PR [#63](https://github.com/SaehwanPark/tabdat-explore-rs/pull/63) added
`Command::XtData`, `XtDataCommand`, and `XtDataTransform` in
[crates/tabdat-language/src/lib.rs](../../crates/tabdat-language/src/lib.rs).
Parser coverage is in
[crates/tabdat-language/tests/parser_contract.rs](../../crates/tabdat-language/tests/parser_contract.rs),
including quoted variables, flag boundaries, and exact diagnostics.

The runtime records `xtdata` in command naming but deliberately continues to
return `RuntimeError::UnsupportedCommand { name: "xtdata" }`. The focused
runtime test demonstrates that no panel metadata or backend execution is
introduced by this slice:
[crates/tabdat-runtime/tests/xtdata_contract.rs](../../crates/tabdat-runtime/tests/xtdata_contract.rs).

The contract checkpoint is
[32307dd](https://github.com/SaehwanPark/tabdat-explore-rs/commit/32307dd073cfc71150378fc0b2e2363fb0b3db5b);
the implementation checkpoint is
[29b3d6d](https://github.com/SaehwanPark/tabdat-explore-rs/commit/29b3d6d2421bc69e32ebc20a3986b77e65d695d7); and the squash merge is
[74eea071d118f3c97c931a9c9345c17293e56440](https://github.com/SaehwanPark/tabdat-explore-rs/commit/74eea071d118f3c97c931a9c9345c17293e56440).

## Verification evidence

The focused Rust parser and runtime contract tests passed. The following
repository checks also passed locally:

    cargo fmt --all -- --check
    cargo check --locked --workspace --all-targets -j 1
    cargo test --locked --workspace --all-targets -j 1
    cargo clippy --locked --workspace --all-targets -j 1 -- -D warnings
    cargo deny check
    cargo audit -D warnings
    metadata-driven cargo geiger checks for the workspace packages
    git diff --check

The successful test run covered the scaffold, 44 language unit tests, 64
parser-contract tests, 33 runtime unit tests, the xtdata runtime contract, and
all existing runtime integration contracts. The geiger reports found no
first-party unsafe code in `tabdat-explore-rs`, `tabdat-language`, or
`tabdat-runtime`.

Hosted PR-head acceptance passed:

- [CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35513359258),
  including [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35513359258/job/106085045335)
  and [dependency/unsafe policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35513359258/job/106085045263).
- [Runtime boundary workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35513359261),
  including [Linux runtime](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35513359261/job/106085045101).

The post-merge `main` workflows passed:

- [merge-head CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35514485430),
  including [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35514485430/job/106087996887)
  and [dependency/unsafe policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35514485430/job/106087996704).
- [merge-head runtime workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35514485511),
  including [Linux runtime](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35514485511/job/106087996957).

## Deviations and deferrals

This evidence covers only the parser boundary. Panel metadata ownership,
active-schema and numeric validation, within/between formulas, generated-column
naming and collisions, missingness, ordering, relation publication, labels,
lazy/materialized behavior, formatting, CLI, JSON, MCP, and broad Python
`xtdata` parity remain deferred. Phase 11.1 `xtdata` runtime work therefore
remains unchecked; the accepted syntax item is recorded under the direct Phase
5.1 language slice.
