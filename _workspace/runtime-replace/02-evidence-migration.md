# Runtime `replace` evidence and migration record

Status: accepted and verified on `main` at `df2cad9`.

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
  -k "replace or quoted_identifiers_execute_with_exact_spelling or \
      rename_generate_and_replace_update_active_dataset or \
      phase_3_transformations_report_user_facing_errors"
24 passed, 393 deselected
```

Those results establish the broader Python contract and explicit Rust
deferrals; they are not claims of full Python/Rust parity.

## Rust implementation evidence

The implementation is the library-only eager local-Parquet path in
`crates/tabdat-runtime`:

- `ReplaceResult` and `ExecutionResult::Replace` own the post-transform
  metadata;
- target, identifier, domain, predicate, and unsupported-form validation run
  before staging;
- accepted expressions use quoted identifiers, escaped string literals, typed
  NULL casts, and the existing checked numeric SQL compiler;
- one source-order `SELECT` is staged in `__tabdat_next`, inspected, and
  transactionally published before `Session.active_dataset` changes; and
- validation, stage, schema/row-count, SQL, or publication failures preserve
  the previous metadata and private active relation.

Focused coverage is in
`crates/tabdat-runtime/tests/replace_contract.rs` (8 integration tests), the
private-backend no-active regression and mismatched-relation atomicity test in
`crates/tabdat-runtime/src/lib.rs`, and the updated no-active regression in
`tests/use_contract.rs`.

## Local verification

The final implementation head before hosted acceptance is `9f3e21a`; the final
PR head, including the review evidence, is `427055c`:

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

The focused runtime suite reported 8 passed integration tests and 27 passed
library tests after the final coverage additions. The geiger inventory reported
`forbid_unsafe=true` and zero first-party unsafe functions/expressions for all
three workspace packages; dependency inventory counts remain the policy
tool's separate report.

## Hosted PR-head evidence

Draft PR [#48](https://github.com/SaehwanPark/tabdat-explore-rs/pull/48) was
opened at the contract checkpoint and contains implementation commits
`a9ec68d`, `6bfb231`, `58fa7e8`, and `9f3e21a`, followed by the review evidence
commit `427055c`. Its final head passed:

- [CI workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35452188768),
  including [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35452188768/job/105921196645)
  and [dependency/unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35452188768/job/105921196520);
- [TabDat runtime boundary](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35452188801)
  ([Linux job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35452188801/job/105921195853)).

The PR was marked ready after the three-pass local review and squash-merged as
[`df2cad9`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/df2cad9e8f61479ad67f118441acbbeb0704c408)
with `--delete-branch`. The local and remote `feat/runtime-replace` refs were
absent after the merge.

## Merge-head evidence

The merge-head [runtime workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35453332473)
passed for `df2cad9` ([Linux job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35453332473/job/105924178833)).
The merge-head CI invocation was superseded by the documentation closeout push
under the repository's `cancel-in-progress` concurrency policy. The follow-on
main [CI workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35453397485)
for `84d6cb2` passed, including [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35453397485/job/105924386502)
and [dependency/unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35453397485/job/105924386301).

## Deviations and deferrals

The Rust runtime intentionally does not claim the Python executor's lazy or
materialized execution, function calls, boolean/other target domains, exact
integral overflow counts, broader non-finite/division diagnostics, labels or
panel metadata, `last_operation`, CLI/JSON/MCP surfaces, or broad transform
sequencing. These remain explicit future slices rather than hidden behavior.
