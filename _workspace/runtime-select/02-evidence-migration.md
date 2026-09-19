# Bounded eager-runtime `select` migration evidence

Status: accepted and merged on `main` at `228fa50`; the temporary
`feat/runtime-select` branch was removed locally and remotely.

Boundary: pinned Python explicit-varlist `select` contract → Rust-owned eager
DuckDB requested-order projection over the active local-Parquet relation.

## Authority and oracle evidence

The clean sibling checkout `/Volumes/research/gitrepo/tabdat-explore` is pinned
to revision `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, Python `3.13.3`, with `uv.lock`
SHA-256 `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.
The recovered paths, diagnostics, implementation leakage, and bounded
deviations are recorded in [`01-contract.md`](01-contract.md).

Focused pinned-oracle selections passed:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider tests/test_parser.py -k select
16 passed, 473 deselected

PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider tests/test_executor.py -k select
3 passed, 414 deselected
```

## Implementation evidence

The branch commits are:

- `1801f3d` — recover and record the pinned select contract;
- `d21788d` — add typed eager select projection, parser parity diagnostics,
  and runtime contract tests.

The runtime validates all names before backend work, rejects direct empty typed
requests, projects in requested order with duplicate naming, quotes identifiers
through the shared `project_columns` path, stages and inspects the relation,
and publishes metadata only after successful transactional publication. The
parser preserves the existing syntax-only boundary while adding the recovered
duplicate-`if` and incomplete-operator diagnostics.

Focused Rust evidence passed:

```text
cargo test --locked -p tabdat-language --test parser_contract select
2 passed
cargo test --locked -p tabdat-runtime --test select_contract
8 passed
cargo test --locked -p tabdat-runtime tests::failed_select_keeps_the_published_dataset_metadata
1 passed
```

The implementation head also passed the local baseline:

```text
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
git diff --check
```

## Hosted implementation-head acceptance

Draft PR [#44](https://github.com/SaehwanPark/tabdat-explore-rs/pull/44) was
opened before implementation. At head `d21788d`, all required hosted checks
passed:

- [CI run](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35430633835),
  [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35430633835/job/105864465410),
  and [dependency/unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35430633835/job/105864465483);
- [TabDat runtime boundary](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35430633830),
  [Linux job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35430633830/job/105864435701).

The docs/evidence head `f1db889` passed the same required hosted checks:

- [CI run](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35431743740),
  [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35431743740/job/105867469973),
  and [dependency/unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35431743740/job/105867470021);
- [TabDat runtime boundary](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35431743753),
  [Linux job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35431743753/job/105867469887).

The PR was squash-merged as
[`228fa505dfeba3fb027590a2d430a28fa45e718f`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/228fa505dfeba3fb027590a2d430a28fa45e718f),
and `feat/runtime-select` was deleted locally and remotely. The post-merge
code-head checks also passed:

- [CI run](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35432672560),
  [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35432672560/job/105869917534),
  and [dependency/unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35432672560/job/105869917620);
- [TabDat runtime boundary](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35432672544),
  [Linux job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35432672544/job/105869917478).

The main docs closeout and native feasibility matrix will be appended after the
roadmap/SPEC/ADR records are updated.

## Deferred scope

Predicate filtering, expression execution, lazy/materialized behavior,
wildcard/range expansion, label/panel metadata, `last_operation`, formatting,
CLI/REPL, JSON/MCP, and broad transform sequencing remain deferred. This
evidence does not claim full Python `select` parity or broad Phase 4 completion.
