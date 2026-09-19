# Runtime encode evidence and migration record

Status: implementation checkpoint; hosted acceptance is pending for the current
PR head.

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
- `915b6ba` — parser diagnostic parity hardening.

## Local verification

The locked baseline and policy checks passed after the final implementation
checkpoint:

    cargo fmt --all -- --check
    cargo check --locked --workspace --all-targets
    cargo test --locked --workspace --all-targets
    cargo clippy --locked --workspace --all-targets -- -D warnings
    git diff --check
    cargo deny check
    cargo audit -D warnings

The metadata-driven geiger loop also passed for `tabdat-explore-rs`,
`tabdat-language`, and `tabdat-runtime`: each report contained exactly one
first-party package, `forbids_unsafe` was true, first-party unsafe counts were
zero, and each geiger process exited zero.

## Hosted acceptance

Draft PR [#53](https://github.com/SaehwanPark/tabdat-explore-rs/pull/53) was
opened at the contract checkpoint. The current implementation head is
`915b6ba`; its hosted baseline, policy, and runtime checks are pending. The
final accepted record will add the documentation-head and merge-head workflow
links after GitHub reports green conclusions.

## Deviations and deferrals

This record does not claim value-label creation/attachment, `decode`, a usable
TabDat CLI, Python/R runtime dependency parity, lazy or materialized execution,
panel metadata, last-operation state, formatting, output adapters, or broad
transform sequencing. Those remain separate roadmap slices.
