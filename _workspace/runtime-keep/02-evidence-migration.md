# Bounded eager-runtime `keep` migration evidence

Status: accepted bounded slice; PR #42, temporary-branch cleanup, and the
post-merge matrix are complete on `main` at `37f0ab6`.

Producer: task owner, using `tabdat-migration`, `tabdat-data-semantics`, and
`simple-code-writer`, with independent runtime review

Consumers: reviewers and the next runtime maintainer

Boundary: pinned Python projection contract → Rust-owned eager DuckDB
projection over the active local-Parquet relation

## Authority and contract inputs

The pinned authority is the clean sibling checkout `../tabdat-explore` at
commit `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, Python `3.13.3`, and `uv.lock`
SHA-256 `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.
The recovered contract is in [`01-contract.md`](01-contract.md).

Authoritative Python paths are `src/tabdat/models.py:244-247,1262-1265`,
`src/tabdat/parser.py:623-624,1026-1035,3036-3101`,
`src/tabdat/executor.py:993-1000,1336-1348,752-812`,
`src/tabdat/backend.py:1152-1168,1247-1278,2518-2522,2303-2395`,
`docs/commands/keep.md`, `docs/language-semantics.md:42-45,91-96,100-122,151-160`,
and the parser/executor selections in `tests/test_parser.py` and
`tests/test_executor.py`.

## Oracle recovery evidence

The focused oracle selections passed at the pinned revision:

```text
tests/test_parser.py -k keep:    2 passed, 487 deselected
tests/test_executor.py -k keep: 16 passed, 401 deselected
```

The projection contract preserves requested column and row order, validates all
unknown variables before querying, and reports `Kept selected columns` on
success. A direct oracle check confirmed duplicate projection names are retained
as `age`, `age_1`. The full Python `keep if <expression>` form remains recovered
but intentionally deferred in this bounded Rust slice.

## Implementation evidence

The implementation commits on `feat/runtime-keep` are:

- `dab8a01` — recover the pinned `keep` contract and bounded scope;
- `bcbaa98` — add typed parser/runtime projection, quoted DuckDB staging, and
  atomic publication;
- `7acbb7b` — cover the backend-initialization boundary;
- `18971ee` — record duplicate projection behavior;
- `3cfbce9` — align assignment diagnostics and add dropped-relation failure
  atomicity coverage;
- `0233457` — align mixed-condition option diagnostics;
- `a910119` — align trailing-comma condition diagnostics and regression tests.

The bounded parser accepts case-insensitive `keep <explicit-varlist>` with
quoted/backtick identifiers and exact assignment, option, condition, and empty
form diagnostics. The runtime validates the active schema before backend work,
quotes identifiers, preserves duplicate requests and row order, stages the
projection in `__tabdat_next`, refreshes owned schema/row-count metadata, and
publishes only after successful transactional replacement. Unknown variables,
missing active relations, schema/count failures, and publication failures leave
the published dataset metadata unchanged.

Focused Rust evidence:

```text
cargo test --locked -p tabdat-language --test parser_contract keep_preserves_bounded_projection_diagnostics
1 passed
cargo test --locked -p tabdat-runtime --test keep_contract
4 passed
cargo test --locked -p tabdat-runtime tests::failed_keep_keeps_the_published_dataset_metadata
1 passed
```

Final local workspace gates at `a910119` all passed:

```text
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
git diff --check
cargo deny check
cargo audit -D warnings
```

The workspace test totals were root 1, language unit 42, parser contract 36,
runtime unit 21, assert contract 7, datasignature contract 6, keep contract 4,
and use contract 63, with no failures. The metadata-driven all-package
`cargo geiger` scan reported clean first-party unsafe usage for
`tabdat-explore-rs`, `tabdat-language`, and `tabdat-runtime`.

## Hosted acceptance, merge, and cleanup

Draft PR [#42](https://github.com/SaehwanPark/tabdat-explore-rs/pull/42) was
opened before implementation. The final code head was `a910119`; PR-head
acceptance is green: [CI run
35422022624](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35422022624)
passed its [Rust baseline job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35422022624/job/105841375575)
and [dependency/unsafe-code policy job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35422022624/job/105841375667),
and [runtime boundary run
35422022659](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35422022659)
passed its [Linux runtime job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35422022659/job/105841374154).
PR #42 was marked ready after those checks and squash-merged as [`d43c923`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/d43c923f2b599a8ad7ebe7a3b5b0af44c1f77e00).
The temporary `feat/runtime-keep` branch was removed locally and remotely. The
docs-inclusive PR head was `b490c61`; its green [CI run
35422981525](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35422981525)
and [runtime boundary run
35422981533](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35422981533)
confirmed the evidence-only closeout before merge. The final main
documentation closeout `37f0ab6` passed the complete matrix:

- [CI run 35424054768](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35424054768), with [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35424054768/job/105846859357) and [dependency/unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35424054768/job/105846859448);
- [TabDat runtime boundary](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35424054718), [Linux job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35424054718/job/105846858331);
- [ReadStat feasibility](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35424054733), [spike job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35424054733/job/105846827011), and [Rust job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35424054733/job/105846827226);
- [libgretl feasibility](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35424054778), [Linux job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35424054778/job/105846826961); and
- [libgretl OLS](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35424054698), [Linux job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35424054698/job/105846826653).

## Deferred scope

`keep if <expression>` remains deferred, including boolean/null filtering,
expression functions, arithmetic overflow reporting, and row-level transform
semantics. Lazy/materialized execution, wildcard/range expansion, options,
labels/panel metadata, `last_operation`, CLI/JSON/MCP surfaces, formatting, and
broader transformation sequencing remain out of scope. This artifact records a
bounded eager projection and does not claim full Python `keep` parity.
