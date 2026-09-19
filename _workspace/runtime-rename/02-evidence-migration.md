# Runtime `rename` evidence and migration record

Status: implementation and local review complete; hosted acceptance is pending.

## Authority and oracle evidence

The behavior authority is the pinned Python checkout recorded in
[`01-contract.md`](01-contract.md): `tabdat-explore` revision
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, Python `3.13.3`, and the recorded
`uv.lock` SHA-256
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.

The recovered oracle was checked out in the isolated sibling clone
`C:\Users\saehwan\repos\tabdat-python-oracle` with the pinned revision and
clean working tree. The focused executor probe reported:

```text
uv run --no-sync pytest -q -p no:cacheprovider tests/test_executor.py \
  -k "rename or quoted_identifiers_execute_with_exact_spelling or \
      rename_generate_and_replace_update_active_dataset or \
      phase_3_transformations_report_user_facing_errors"
3 passed, 414 deselected
```

## Rust implementation evidence

The implementation is the library-only eager local-Parquet path in
`crates/tabdat-runtime`:

- `RenameResult` and `ExecutionResult::Rename` own the post-transform metadata;
- source/target validation occurs before backend staging;
- source-order SQL aliases only the renamed column through the existing quoted
  identifier helper, including embedded double quotes;
- staged schema and row count are inspected before the shared transaction
  publishes the relation; and
- validation, stage, schema/row-count, or publication failures leave the
  previously published metadata and private active relation unchanged.

Focused coverage is in
`crates/tabdat-runtime/tests/rename_contract.rs` (4 integration tests), the
private-backend no-active regression and mismatched-relation atomicity test in
`crates/tabdat-runtime/src/lib.rs`, and the updated no-active regression in
`crates/tabdat-runtime/tests/use_contract.rs`.

The contract checkpoint is commit `3c847f0`; the implementation commit is
`9e298fb`; and the final PR-head documentation correction is `e650128`.

## Local verification

At implementation commit `9e298fb`:

```text
cargo fmt --all -- --check                         passed
cargo check --locked --workspace --all-targets    passed
cargo test --locked --workspace --all-targets     passed
cargo clippy --locked --workspace --all-targets -- -D warnings
                                                     passed
git diff --check                                  passed
cargo deny check                                  passed
cargo audit -D warnings                           passed
```

The focused runtime suite reported 4 passed rename integration tests and 29
passed runtime library tests; the complete workspace test suite also passed.
The metadata-driven geiger loop passed for `tabdat-explore-rs`,
`tabdat-language`, and `tabdat-runtime`, with `forbids_unsafe=true` and zero
first-party unsafe counts for each package. Dependency inventory warnings were
not present in this run.

The final PR head `e650128` changes only the stale public `Command::Rename`
documentation comment found during review; `cargo fmt --all -- --check` passed
after that correction.

## Hosted acceptance

Draft PR [#49](https://github.com/SaehwanPark/tabdat-explore-rs/pull/49) was
opened at the contract checkpoint and contains the bounded contract,
implementation, tests, and review evidence. The PR must be marked ready only
after its final-head CI and runtime-boundary jobs pass. The final hosted run
links and squash-merge commit will be recorded here before closeout.

## Deviations and deferrals

The Rust runtime intentionally does not claim the Python executor's panel or
label metadata updates, lazy or materialized execution, wildcard or
multi-column forms, `last_operation`, formatting, CLI/JSON/MCP surfaces, or
broad transform sequencing. These remain explicit future slices rather than
hidden behavior.
