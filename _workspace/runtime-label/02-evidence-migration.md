# Runtime label evidence and migration record

Status: accepted and verified on `main` at merge commit
[`70b9745`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/70b9745ae7e22855c763bcb9e3ed40332646723c).

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

    uv run --no-sync pytest -q -p no:cacheprovider tests/test_labels.py
    6 passed in 0.55s

    uv run --no-sync pytest -q -p no:cacheprovider tests/test_encode_decode.py
    6 passed in 0.86s

The probes confirmed variable-label set/clear, integer/string/numeric label
values, duplicate-definition and option diagnostics, list/drop behavior, and
encode/decode provenance. JSON label persistence was intentionally excluded
from this bounded Rust slice.

## Rust implementation evidence

The contract checkpoint was [`75b0a80`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/75b0a801bb82af6df65cac73c4120c808ef47ab2).
The implementation checkpoint was
[`aa6f952`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/aa6f952f859630a8c77b24887f7b96a012f9d24f),
which was squash-merged as
[`70b9745`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/70b9745ae7e22855c763bcb9e3ed40332646723c).
It contains:

- typed parser forms for variable labels, definitions, attachments, listing,
  and dropping;
- owned `LabelMetadata`, `ValueLabelSet`, and `LabelResult` runtime values;
- atomic validation and session-local updates with deterministic metadata
  normalization;
- encode named/default value-label set publication and decode through an
  attached integer set; and
- reconciliation across `use`, rename, projections, value-changing replace,
  and in-place recode.

Focused Rust coverage includes 50 parser-contract tests, five label runtime
tests, and the existing encode/decode suites. No dependency, native backend,
FFI, unsafe-code, or lockfile change was made.

## Local verification

The locked repository baseline passed after the implementation checkpoint:

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
forbade unsafe code and reported zero first-party unsafe counts.

## Hosted acceptance

PR [#55](https://github.com/SaehwanPark/tabdat-explore-rs/pull/55) was opened
as a draft at contract checkpoint [`75b0a80`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/75b0a801bb82af6df65cac73c4120c808ef47ab2),
received the implementation checkpoint, and was marked ready after review.
Its final head [`aa6f952`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/aa6f952f859630a8c77b24887f7b96a012f9d24f) passed:

- [PR-head CI run 35483588143](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35483588143), including [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35483588143/job/106005729492) and [dependency/unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35483588143/job/106005729445); and
- [PR-head runtime run 35483588144](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35483588144), including its [Linux runtime job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35483588144/job/106005696691).

The squash merge head was then verified by:

- [main CI run 35484575373](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35484575373), including [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35484575373/job/106008480712) and [dependency/unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35484575373/job/106008480569); and
- [main runtime run 35484575369](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35484575369), including its [Linux runtime job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35484575369/job/106008480665).

The documentation-closeout commit `3a0ab00` then passed:

- [final main CI run 35485577830](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35485577830), including [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35485577830/job/106011208017) and [dependency/unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35485577830/job/106011207859);
- [final ReadStat workflow 35485577864](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35485577864), including [facade job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35485577864/job/106011208002) and [upstream build job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35485577864/job/106011208037);
- [final libgretl feasibility workflow 35485577816](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35485577816), including its [Linux spike job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35485577816/job/106011208154); and
- [final libgretl OLS workflow 35485577786](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35485577786), including its [Linux OLS job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35485577786/job/106011207790).

## Deviations and deferrals

The Python oracle supports JSON label dictionaries, DTA-imported labels,
inspection/reporting rendering, and broader transform/output surfaces. This
Rust slice keeps metadata in the eager session and deliberately defers
`label save`, `label use`, DTA ingestion, lazy/materialized execution, panel
metadata, formatting, CLI, JSON/MCP adapters, and broad transform sequencing.
