# Selected `estat` syntax migration evidence

## Status and boundary

Status: `complete`.

This is the accepted parser-only selected `estat` slice. It ports the direct
no-option subcommand boundary for `firststage`, `overid`, `endogenous`, and
`hausman`; it does not claim post-estimation calculation, statistical
model-state lookup, IV/panel validation, or active-session execution. Runtime
execution remains an explicit unsupported-command result until the typed model
and post-estimation contracts are separately recovered and tested.

## Authority and recovered behavior

The Python oracle is the isolated checkout at
`C:\Users\saehwan\repos\tabdat-python-oracle`, pinned to revision
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`). Its `uv.lock` SHA-256 is
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.
The relevant sources are `src/tabdat/models.py` (`EstatCommand`),
`src/tabdat/parser.py` (`_parse_estat`), and `docs/commands/estat.md`.
The focused parser command was:

    uv run --no-sync pytest -q -p no:cacheprovider tests/test_parser.py -k estat

It passed with `19 passed, 470 deselected`. The recovered selected forms accept
exactly one subcommand argument, normalize command and subcommand case,
accept single- and double-quoted subcommands, reject backtick-quoted
subcommands, and reject options with `estat <subcommand> does not support
options`. Conditions, assignments, extra arguments, and unsupported selected
boundaries preserve the recovered syntax diagnostics. Structured spatial and
`report` options, and other `estat` subcommands, are outside this slice.

## Rust implementation

PR [#66](https://github.com/SaehwanPark/tabdat-explore-rs/pull/66) added
`Command::Estat`, `EstatCommand`, and `EstatSubcommand` in
[crates/tabdat-language/src/lib.rs](../../crates/tabdat-language/src/lib.rs).
Parser coverage is in
[crates/tabdat-language/tests/parser_contract.rs](../../crates/tabdat-language/tests/parser_contract.rs),
including all four selected forms, case/quote boundaries, options, syntax
boundaries, and exact diagnostics.

The runtime records `estat` in command naming but deliberately continues to
return `RuntimeError::UnsupportedCommand { name: "estat" }`. The focused
runtime test demonstrates that no model-state lookup, diagnostic calculation,
or backend execution is introduced by this slice:
[crates/tabdat-runtime/tests/estat_contract.rs](../../crates/tabdat-runtime/tests/estat_contract.rs).

The contract checkpoint is
[82c9b2f](https://github.com/SaehwanPark/tabdat-explore-rs/commit/82c9b2f);
the implementation checkpoint is
[6600b0e](https://github.com/SaehwanPark/tabdat-explore-rs/commit/6600b0e); and
the squash merge is
[484148ee6a25dfe5f245fd2f7be01eec5efd27b3](https://github.com/SaehwanPark/tabdat-explore-rs/commit/484148ee6a25dfe5f245fd2f7be01eec5efd27b3).

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

The successful test run covered the scaffold, 44 language unit tests, 70
parser-contract tests, 33 runtime unit tests, the estat runtime contract, and
all existing runtime integration contracts. The geiger reports found no
first-party unsafe code in `tabdat-explore-rs`, `tabdat-language`, or
`tabdat-runtime`.

Hosted PR-head acceptance passed:

- [CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35527592097),
  including [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35527592097/job/106122404177)
  and [dependency/unsafe policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35527592097/job/106122404273).
- [Runtime boundary workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35527592158),
  including [Linux runtime](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35527592158/job/106122371226).

The post-merge `main` workflows passed:

- [merge-head CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35528915431),
  including [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35528915431/job/106125900131)
  and [dependency/unsafe policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35528915431/job/106125900172).
- [merge-head runtime workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35528915585),
  including [Linux runtime](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35528915585/job/106125900625).

## Deviations and deferrals

This evidence covers only the selected parser boundary. First-stage,
overidentification, endogeneity, and Hausman calculations, statistical
model-state ownership, IV/panel validation, covariance, missingness, result
formatting, labels, CLI, JSON, MCP, remaining `estat` subcommands, and broad
Python `estat` parity remain deferred. Phase 11.1 runtime work therefore
remains unchecked; the accepted syntax item is recorded under the direct Phase
5.1 language slice.
