# Runtime tabulate evidence and migration record

Status: accepted and verified on main at merge commit
[24405a6](https://github.com/SaehwanPark/tabdat-explore-rs/commit/24405a6878034506e18086d5092d665b4dd25159).

## Authority and oracle evidence

The behavior authority is the pinned Python checkout recorded in
[01-contract.md](01-contract.md): tabdat-explore revision
16b45d9b66b0d80f32d4d220e84d81bc5180bdbe, tree
601b236788872323af9277d2276a236154a0f129, and the recorded uv.lock
SHA-256 0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239.

The isolated oracle checkout at
C:/Users/saehwan/repos/tabdat-python-oracle was clean at the pinned
revision. Focused recovery reported:

    uv run --no-sync pytest -q -p no:cacheprovider tests/test_tabulate_labels.py
    6 passed in 0.55s

    uv run --no-sync pytest -q -p no:cacheprovider tests/test_executor.py -k "tabulate_one_way_and_two_way or tabulate_missing_option_controls_missing_categories"
    2 passed, 418 deselected in 3.78s

These checks cover the bounded parser flags, one-way and two-way frequencies,
percentage columns, missing-category inclusion, and attached value-label
display. Broader aggregation, condition, prefix, output, and persistence
surfaces remain outside this slice.

## Rust implementation evidence

The contract checkpoint was [98021ee](https://github.com/SaehwanPark/tabdat-explore-rs/commit/98021eea39c8692eac29bfd688d0178e709ba4e9).
The parser checkpoint was
[dd403a6](https://github.com/SaehwanPark/tabdat-explore-rs/commit/dd403a6c238b1f3fcbd6a90d20f458e55d6ef4d7),
and the implementation checkpoint was
[cb463cd](https://github.com/SaehwanPark/tabdat-explore-rs/commit/cb463cd9fe2afebbc3eae51ec8e8358a9b6a99e2).
They were squash-merged by PR #56 as
[24405a6](https://github.com/SaehwanPark/tabdat-explore-rs/commit/24405a6878034506e18086d5092d665b4dd25159).

The implementation contains:

- typed direct one-way and two-way parser forms with bounded flag
  diagnostics;
- an owned TabulateResult with no DuckDB row or statement lifetimes;
- eager local-Parquet grouped counts with default missing exclusion and
  explicit missing inclusion;
- deterministic category order, zero-filled two-way cells, row/column
  percentages, and session-local value-label rendering; and
- read-only execution that preserves the active relation and label metadata.

Focused Rust coverage contains 52 parser-contract tests and five tabulate
runtime contract tests.

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
each of tabdat-explore-rs, tabdat-language, and tabdat-runtime; each package
forbids unsafe code and reported zero first-party unsafe counts.

## Hosted acceptance

PR [#56](https://github.com/SaehwanPark/tabdat-explore-rs/pull/56) was opened
as a draft at the contract checkpoint and received the parser and runtime
implementation checkpoints. Its final head cb463cd passed:

- [PR-head CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35486898316),
  including [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35486898316/job/106014856696)
  and [dependency/unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35486898316/job/106014856664); and
- [PR-head runtime](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35486898348),
  including its [Linux runtime job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35486898348/job/106014855794).

The squash merge head 24405a6 was then verified by:

- [main CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35487873526),
  including [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35487873526/job/106017458148)
  and [dependency/unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35487873526/job/106017458222); and
- [main runtime](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35487873479),
  including its [Linux runtime job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35487873479/job/106017458008).

The documentation-closeout workflows are recorded after their completion.

## Deviations and deferrals

The Python oracle also supports values/stat aggregation, if predicates,
by-prefixes, multiple row/column dimensions, named-table execution, lazy and
materialized modes, formatting, persistence, and multiple output surfaces.
This Rust slice intentionally defers those forms, as well as CLI, JSON, MCP,
and broad Python tabulate parity.
