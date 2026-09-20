# bounded `ttest` migration evidence

## Status and boundary

Status: `complete`.

This is the accepted language-layer syntax slice for direct `ttest` forms. It
recovers value comparisons, paired-variable comparisons, and grouped
comparisons with `by()`, `welch`, and `unequal`. It does not fit a model,
calculate a statistic or p-value, inspect a relation, validate column types,
mutate session state, initialize a backend, or expose statistical runtime
behavior.

## Authority and recovered behavior

The Python oracle is the isolated checkout at
`C:\Users\saehwan\repos\tabdat-python-oracle`, pinned to revision
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`). Its `uv.lock` SHA-256 is
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.
The relevant source is `src/tabdat/parser.py` (`_parse_ttest`, option parsing,
and the shared tokenizer).

The full pinned parser command passed with `489 passed`. The focused `ttest`
selection had no dedicated oracle tests, so direct probes established the
bounded contract. Recovered accepted forms include:

- `ttest wage == 0`, signed values such as `-1.5`, `+2`, and `.5`;
- `ttest wage == exposure` for a paired-variable comparison;
- `ttest wage, by(group)` with flag-only `welch` and `unequal` options;
- unquoted, quoted, and backtick-decoded variable names;
- repeated `welch`/`unequal` flags, with `unequal` mapped to the typed `welch`
  field.

Recovered diagnostics include empty/missing comparison forms, duplicate
comparisons and commas, invalid LHS/RHS shapes, missing or duplicate `by()`,
unsupported options, valued `welch`/`unequal` flags, malformed numbers, and
the existing option-grammar diagnostics for invalid `by()` values.

## Rust implementation

PR [#69](https://github.com/SaehwanPark/tabdat-explore-rs/pull/69) adds
`Command::Ttest { command: TtestCommand }` to
[`crates/tabdat-language/src/lib.rs`](../../crates/tabdat-language/src/lib.rs).
The owned command stores `varname1`, an optional `varname2`, an optional
numeric source spelling, an optional `by_variable`, and the normalized `welch`
flag. Numeric text remains an owned `String` so the public command remains
`Eq`-compatible; conversion belongs to a future statistical boundary.

The parser reuses the shared tokenizer and existing option grammar without
changing their global behavior. Runtime dispatch names the command as
`ttest`, while the existing explicit unsupported-command path keeps inference
deferred. Focused coverage is in
[`crates/tabdat-language/tests/ttest_contract.rs`](../../crates/tabdat-language/tests/ttest_contract.rs)
and
[`crates/tabdat-runtime/tests/ttest_contract.rs`](../../crates/tabdat-runtime/tests/ttest_contract.rs).

The contract checkpoints are `898faf6` and `238513f`; the implementation
checkpoint is `ad8add7`; the squash merge is
[`4dcd892`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/4dcd892f59422774e49742019e5f4eefb459711b).

## Verification evidence

The following repository checks passed locally:

    cargo fmt --all -- --check
    cargo check --locked --workspace --all-targets -j 1
    cargo test --locked --workspace --all-targets -j 1
    cargo clippy --locked --workspace --all-targets -j 1 -- -D warnings
    cargo deny check
    cargo audit -D warnings
    metadata-driven cargo geiger checks for the workspace packages
    git diff --check

The focused Rust tests passed with two language tests and one runtime test;
the full workspace test run also passed the scaffold, 44 language unit tests,
72 parser-contract tests, four tokenizer-contract tests, 33 runtime unit
tests, and all runtime integration contracts. The geiger reports found no
first-party unsafe code in `tabdat-explore-rs`, `tabdat-language`, or
`tabdat-runtime`.

Final PR-head acceptance passed:

- [CI run 35543209210](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35543209210), including [policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35543209210/job/106164529713) and [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35543209210/job/106164529804).
- [Runtime boundary run 35543209204](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35543209204), including [Linux runtime](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35543209204/job/106164492762).

Post-merge `main` acceptance passed:

- [merge-head CI run 35544366067](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35544366067), including [policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35544366067/job/106167556327) and [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35544366067/job/106167556414).
- [merge-head runtime run 35544366084](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35544366084), including [Linux runtime](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35544366084/job/106167556432).

## Deviations and deferrals

The Rust AST intentionally owns numeric spelling instead of Python's eager
`float` value because the public Rust `Command` derives `Eq`; statistical
conversion is deferred and the accepted/rejected lexical forms remain oracle
driven. Full command parsing, varlists, general option/expression grammar,
conditions, relation/schema validation, inference, covariance, missingness,
post-estimation state, reporting, CLI/JSON/MCP surfaces, and broad Python
statistical parity remain deferred.
