# Runtime decode evidence and migration record

Status: accepted and verified on `main` at merge commit
[`0845e6c`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/0845e6cf37a50c317d7ee30acee2d85474c7bcd2).

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
    6 passed in 0.86s

The recovered contract includes parser-owned `DecodeCommand` values, attached
label lookup, mapped numeric-to-string CASE projection, NULL for unmatched
codes, and state preservation when no labels are attached. This Rust slice
intentionally narrows the attached-label source to the private map produced by
the bounded Rust `encode` command.

## Rust implementation evidence

Implementation checkpoint [`73be75e`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/73be75e)
contains:

- `tabdat-language::Command::Decode` and exact bounded parser diagnostics;
- `DecodeResult`, `ExecutionResult::Decode`, and typed runtime errors;
- a session-owned code-to-text map created only after successful encode
  publication;
- staged, quoted DuckDB decode projection with schema/count checks and atomic
  publication; and
- focused runtime coverage for no-active behavior, round-trip NULL handling,
  rename/projection provenance, quoted names, empty mappings, collisions, and
  backend-failure retry.

No dependency, native backend, FFI, unsafe-code, or lockfile change was made.

## Local verification

The locked repository baseline passed at this checkpoint:

    cargo fmt --all -- --check
    cargo check --locked --workspace --all-targets
    cargo test --locked --workspace --all-targets
    cargo clippy --locked --workspace --all-targets -- -D warnings
    git diff --check

The policy checks also passed:

    cargo deny check
    cargo audit -D warnings

The metadata-driven geiger loop reported exactly one first-party package for
each of `tabdat-explore-rs`, `tabdat-language`, and `tabdat-runtime`; each
forbade unsafe code and reported zero first-party unsafe counts. Each geiger
process exited zero.

## Hosted acceptance

PR [#54](https://github.com/SaehwanPark/tabdat-explore-rs/pull/54) was opened
as a draft at contract checkpoint [`5006f38`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/5006f38),
marked ready after the implementation/evidence checkpoints, and its final
head [`1ffb845`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/1ffb84553f0b098f0b9b25ac589cdfaf2baccba7) passed:

- [PR-head CI run 35479575291](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35479575291);
- [PR-head runtime run 35479575205](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35479575205).

The PR was squash-merged as
[`0845e6c`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/0845e6cf37a50c317d7ee30acee2d85474c7bcd2).
The merge-head workflows passed:

- [main CI run 35480547109](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35480547109), including [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35480547109/job/105997441966) and [dependency/unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35480547109/job/105997442091); and
- [main runtime run 35480547104](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35480547104), including its [Linux runtime job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35480547104/job/105997441940).

Documentation closeout commit
[`08dba48`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/08dba4868d70f44f50cce6a11392dc8118357cda) then passed:

- [final main CI run 35481514850](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35481514850), including [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35481514850/job/106000077668) and [dependency/unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35481514850/job/106000077736);
- [final ReadStat workflow 35481514778](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35481514778), with [ReadStat spike](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35481514778/job/106000077051) and [ReadStat Rust spike](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35481514778/job/106000077176);
- [final libgretl feasibility workflow 35481514780](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35481514780), including its [Linux spike job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35481514780/job/106000077223); and
- [final libgretl OLS workflow 35481514788](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35481514788), including its [Linux spike job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35481514788/job/106000077244).

The remote feature branch was deleted after merge and pruned locally.

## Deviations and deferrals

The Python oracle supports arbitrary label metadata, the generic `label`
command, DTA-imported
value labels, variable labels, and broader transform metadata. This branch
implements only encode-produced mappings in one eager session. Generic label
metadata, DTA ingestion, lazy/materialized execution, panel metadata,
last-operation state, formatting, output adapters, and broad transform
sequencing remain separate roadmap work.
