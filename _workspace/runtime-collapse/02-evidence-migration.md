# Runtime collapse evidence and migration record

Status: accepted and verified on main at merge commit
[94391af](https://github.com/SaehwanPark/tabdat-explore-rs/commit/94391afe84ab1b1c8c3c5006a55911426d6767b1).

## Authority and oracle evidence

The behavior authority is the pinned Python checkout recorded in
[01-contract.md](01-contract.md): tabdat-explore revision
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, and the recorded `uv.lock`
SHA-256 `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.

The isolated oracle checkout at
`C:/Users/saehwan/repos/tabdat-python-oracle` was clean at the pinned
revision. Focused recovery reported:

    uv run --no-sync pytest -q -p no:cacheprovider tests/test_parser.py -k collapse
    3 passed, 486 deselected in 0.61s

    uv run --no-sync pytest -q -p no:cacheprovider tests/test_executor.py -k collapse
    1 passed, 416 deselected in 1.90s

These checks cover the direct parser form, required `by(...)` grouping, the
supported statistic set, and replacement of the active dataset with grouped
aggregate columns.

## Rust implementation evidence

The contract checkpoint was [a45dd6f](https://github.com/SaehwanPark/tabdat-explore-rs/commit/a45dd6f94eb449de7fd2fdb4c3b9aea7e583511c),
the parser checkpoint was
[90f4263](https://github.com/SaehwanPark/tabdat-explore-rs/commit/90f42632f2178a3593abed33547ca87c6603a859),
and the runtime checkpoint was
[e23d97b](https://github.com/SaehwanPark/tabdat-explore-rs/commit/e23d97b5d49383d8c0c6c268711afb2e5e86a48c).
The bounded empty-`by()` diagnostic and contract correction were recorded in
[e7f2540](https://github.com/SaehwanPark/tabdat-explore-rs/commit/e7f25409c7d60543f1b6e8ec56f024511ffb69ea).
They were squash-merged by
[PR #57](https://github.com/SaehwanPark/tabdat-explore-rs/pull/57) as
[94391af](https://github.com/SaehwanPark/tabdat-explore-rs/commit/94391afe84ab1b1c8c3c5006a55911426d6767b1).

The implementation contains:

- an owned typed command for `count`, `mean`, `sum`, `min`, and `max` over an
  ordered aggregate variable list and non-empty grouping list;
- parser diagnostics for missing, repeated, empty, or additional `by(...)`
  options, unsupported statistics, conditions, and assignment syntax;
- eager local-Parquet DuckDB grouping with SQL NULL groups, NULL-last
  deterministic ordering, non-NULL `count(variable)`, and numeric validation
  for the other statistics;
- staged atomic active-relation replacement with an owned `CollapseResult` and
  preserved source/eager metadata; and
- session-local pruning of variable labels and value-label attachments to
  surviving group columns, without synthesizing aggregate labels.

Focused Rust coverage contains 54 parser-contract tests and four collapse
runtime contract tests.

## Local verification

The locked repository baseline passed after the final implementation change:

    cargo fmt --all -- --check
    cargo check --locked --workspace --all-targets
    cargo test --jobs 1 --locked --workspace --all-targets
    cargo clippy --locked --workspace --all-targets -- -D warnings
    git diff --check

The first exact parallel Windows `cargo test --locked --workspace
--all-targets` attempt hit a compiler memory-allocation/stack-buffer failure;
the single-job rerun passed every workspace target. Hosted CI ran the exact
baseline test command successfully.

The policy checks also passed:

    cargo deny check
    cargo audit -D warnings

The metadata-driven geiger assertions reported exactly one first-party package
for each of `tabdat-explore-rs`, `tabdat-language`, and `tabdat-runtime`; each
package forbids unsafe code and reported zero first-party unsafe counts. The
PowerShell invocation reproduced the repository's metadata-driven assertions
because `jq` was unavailable in the local shell.

## Hosted acceptance

PR [#57](https://github.com/SaehwanPark/tabdat-explore-rs/pull/57) was opened
as a draft at the contract checkpoint, received parser/runtime checkpoints, and
was marked ready only after review and local verification. Its final head
[e7f2540](https://github.com/SaehwanPark/tabdat-explore-rs/commit/e7f25409c7d60543f1b6e8ec56f024511ffb69ea) passed:

- [PR-head CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35491989391),
  including [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35491989391/job/106028427849)
  and [dependency/unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35491989391/job/106028427992); and
- [PR-head runtime](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35491989384),
  including its [Linux runtime job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35491989384/job/106028429092).

The squash merge head [94391af](https://github.com/SaehwanPark/tabdat-explore-rs/commit/94391afe84ab1b1c8c3c5006a55911426d6767b1) passed:

- [main CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35492943464),
  including [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35492943464/job/106030887960)
  and [dependency/unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35492943464/job/106030888019); and
- [main runtime](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35492943539),
  including its [Linux runtime job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35492943539/job/106030888090).

The documentation-closeout commit [02e1554](https://github.com/SaehwanPark/tabdat-explore-rs/commit/02e1554b3853bc48d83521a050e389dadd3c518d) passed:

- [main CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35494006705),
  including [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35494006705/job/106033677232)
  and [dependency/unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35494006705/job/106033677331);
- [ReadStat feasibility](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35494006702),
  including [ReadStat spike](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35494006702/job/106033677280)
  and [ReadStat Rust spike](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35494006702/job/106033677364);
- [libgretl feasibility](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35494006655),
  including its [Linux spike job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35494006655/job/106033677058); and
- [libgretl OLS Rust spike](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35494006711),
  including its [Linux spike job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35494006711/job/106033677325).

## Deviations and deferrals

The Python oracle also supports conditions, weights, named tables, lazy and
materialized modes, panel propagation, persistence, formatting, CLI, JSON, MCP,
and broader `collapse`/aggregation surfaces. This Rust slice intentionally
defers those forms and broad Python parity.
