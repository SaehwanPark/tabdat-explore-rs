# Runtime `generate` evidence and migration record

Status: accepted on `main` at `98979bc`.

## Authority and oracle evidence

The behavior authority is the pinned Python checkout recorded in
[`01-contract.md`](01-contract.md): `tabdat-explore` revision
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, Python `3.13.3`, and the recorded
`uv.lock` digest.

The read-only oracle reconnaissance covered the parser, executor, backend,
language-semantics documentation, focused executor/CLI/MCP tests, and the
target-collision, type, missing/non-finite arithmetic, exact-integral, and
atomic-failure behavior. The pinned focused runs reported:

```text
tests/test_executor.py focused generate/arithmetic/type/atomicity subset:
75 passed, 342 deselected
tests/test_cli.py focused JSON/output subset:
4 passed, 179 deselected
```

Those results establish the broader Python contract and the explicit Rust
deferrals; they are not claims of full Python/Rust parity.

## Rust implementation evidence

The accepted implementation is the library-only eager local-Parquet path in
`crates/tabdat-runtime`:

- `GenerateResult` and `ExecutionResult::Generate` own the post-transform
  metadata;
- target collisions, unknown identifiers, numeric-domain checks, and
  unsupported expression forms are validated before staging;
- numeric expressions are compiled through quoted identifiers and the existing
  checked numeric SQL helpers;
- `SELECT *, <expression> AS <target>` is staged in `__tabdat_next`, inspected,
  and transactionally published before `Session.active_dataset` changes;
- validation, SQL, inspection, or publication failures preserve the prior
  metadata and private active relation.

Focused Rust coverage is in
`crates/tabdat-runtime/tests/generate_contract.rs` (8 integration tests), the
internal mismatched-relation atomicity test in `src/lib.rs`, and the updated
no-active regression in `tests/use_contract.rs`.

## Local verification

On the final implementation head `861214f` before merge:

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

The focused runtime suite reported 8 passed tests, and the complete workspace
suite passed after the embedded-quote coverage was added.

## Hosted PR-head evidence

Draft PR [#46](https://github.com/SaehwanPark/tabdat-explore-rs/pull/46) was
opened at the contract commit before implementation, marked ready after local
review, and its final head `861214f` passed:

- [CI workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35442489756),
  including [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35442489756/job/105895653408)
  and [dependency/unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35442489756/job/105895653322);
- [TabDat runtime boundary](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35442489682)
  ([Linux job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35442489682/job/105895653129)).

## Merge and post-merge evidence

PR #46 was squash-merged as
[`98979bc`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/98979bce06f6a85932dd51a1b46421e72b542aa7)
with `--delete-branch`. The local and remote `feat/runtime-generate` refs were
absent after the merge. The merge-head workflows both passed:

- [post-merge CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35443571579),
  [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35443571579/job/105898548371),
  and [dependency/unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35443571579/job/105898548171);
- [post-merge runtime boundary](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35443571550)
  ([Linux job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35443571550/job/105898548252)).

The later docs closeout is intentionally a separate push so these merge-head
checks remain durable evidence for the implementation revision.

## Deviations and deferrals

The Rust runtime intentionally does not claim the Python executor’s strings,
booleans, NULL/comparison expressions, function calls, exact overflow counts,
non-finite/division normalization contract, lazy/materialized fallback,
labels/panel metadata, `last_operation`, CLI/JSON/MCP, or broad transformation
sequencing. These remain explicit future slices rather than hidden behavior.
