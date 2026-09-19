# Runtime encode evidence and migration record

Status: accepted and verified on `main` at merge commit
[`af3e3b2`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/af3e3b2726778af5c5f3b5c13c4ba84e5291da61).

## Authority and oracle evidence

The behavior authority is the pinned Python checkout recorded in
[01-contract.md](01-contract.md): tabdat-explore revision
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, Python 3.13.3, and the recorded
`uv.lock` SHA-256
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.

The isolated oracle checkout at
`C:\Users\saehwan\repos\tabdat-python-oracle` was clean at the pinned
revision. Focused recovery reported:

    uv run --no-sync pytest -q -p no:cacheprovider tests/test_encode_decode.py
    6 passed in 0.96s

An additional isolated DuckDB probe confirmed that source rows `b, a, NULL, b`
produce codes `2, 1, NULL, 2`, with an `INTEGER` generated column. The
implementation also records the oracle's empty-relation typed-NULL behavior.

## Rust implementation evidence

The implementation is the library-only eager local-Parquet path:

- `crates/tabdat-language` owns the typed source, generated target, and optional
  label name;
- `crates/tabdat-runtime` validates the active schema and explicit label
  boundary, discovers sorted nonmissing values, quotes identifiers/literals,
  stages the CASE projection, checks schema/count, and publishes atomically;
- `EncodeResult` and `ExecutionResult::Encode` report the transformed dataset;
  and
- label metadata, decode, lazy/materialized execution, panel metadata,
  last-operation state, and output adapters remain deferred.

Focused Rust coverage is in
`crates/tabdat-language/tests/parser_contract.rs` and
`crates/tabdat-runtime/tests/encode_contract.rs`. It covers no-active behavior,
sorted 1-based codes, duplicates and NULLs, quoted and empty relations,
repeated success, label/type/source/target validation, and a backend failure
that preserves the previously published relation and metadata.

The implementation checkpoint commits are:

- `2d9f075` — contract;
- `ecb5b38` — language/runtime implementation;
- `d3895c0` — focused runtime tests; and
- `1f67205` — validation-order and overflow hardening; and
- `915b6ba` — parser diagnostic parity hardening; and
- `4bf0941` — case-sensitive option-name parity; and
- `8c80894` — final parser checkpoint and documentation evidence.

## Local verification

The locked baseline checks passed at the final PR head:

    cargo fmt --all -- --check
    cargo check --locked --workspace --all-targets
    cargo test --locked --workspace --all-targets
    cargo clippy --locked --workspace --all-targets -- -D warnings
    git diff --check

Dependency-policy, advisory-audit, and metadata-driven geiger checks also
passed for the implementation branch; the final parser/documentation checkpoint
changed no dependencies or unsafe boundaries.

    cargo deny check
    cargo audit -D warnings

The metadata-driven geiger loop also passed for `tabdat-explore-rs`,
`tabdat-language`, and `tabdat-runtime`: each report contained exactly one
first-party package, `forbids_unsafe` was true, first-party unsafe counts were
zero, and each geiger process exited zero.

## Hosted acceptance

Draft PR [#53](https://github.com/SaehwanPark/tabdat-explore-rs/pull/53) was
opened at the contract checkpoint and marked ready after the final checks. Its
final PR head [`8c80894`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/8c80894b21042479be1b250fbc84c066579fc0d1)
passed:

- [CI run 35474913800](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35474913800), including [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35474913800/job/105982427299) and [dependency/unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35474913800/job/105982427443); and
- [tabdat-runtime run 35474913819](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35474913819), including its [Linux runtime job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35474913819/job/105982428529).

The PR was squash-merged as
[`af3e3b2`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/af3e3b2726778af5c5f3b5c13c4ba84e5291da61).
The merge-head passed:

- [main CI run 35476036980](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35476036980), including [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35476036980/job/105985334039) and [dependency/unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35476036980/job/105985334268); and
- [main runtime run 35476036994](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35476036994), including its [Linux runtime job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35476036994/job/105985334401).

Documentation-closeout workflow evidence will be added after the main-branch
documentation commit.

## Deviations and deferrals

This record does not claim value-label creation/attachment, `decode`, a usable
TabDat CLI, Python/R runtime dependency parity, lazy or materialized execution,
panel metadata, last-operation state, formatting, output adapters, or broad
transform sequencing. Those remain separate roadmap slices.
