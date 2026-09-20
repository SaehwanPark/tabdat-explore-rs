# Runtime by evidence and migration record

Status: accepted and verified on main at merge commit
[cc818e5](https://github.com/SaehwanPark/tabdat-explore-rs/commit/cc818e5e5e395d9b0f8c86a0dee451417ca98002), via
[PR #58](https://github.com/SaehwanPark/tabdat-explore-rs/pull/58).

## Authority and oracle evidence

The behavior authority is the pinned Python checkout recorded by
[01-contract.md](01-contract.md): tabdat-explore revision
16b45d9b66b0d80f32d4d220e84d81bc5180bdbe, tree
601b236788872323af9277d2276a236154a0f129, and the recorded uv.lock
SHA-256 0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239.

The isolated oracle checkout at
C:/Users/saehwan/repos/tabdat-python-oracle was clean at the pinned
revision. Focused recovery reported:

    uv run --no-sync pytest -q -p no:cacheprovider tests/test_parser.py -k by
    7 passed, 482 deselected in 0.53s

    uv run --no-sync pytest -q -p no:cacheprovider tests/test_executor.py -k 'by_summarize_and_count or unsupported_by_command'
    2 passed, 415 deselected in 2.07s

The executor selection for by_tabulate collected no tests and was not used
as passing evidence. Grouped tabulate execution is an explicit Rust deferral
in this slice.

## Rust implementation evidence

The contract checkpoint was
[ef3e60b](https://github.com/SaehwanPark/tabdat-explore-rs/commit/ef3e60b2c9d80a3dc7ef8fc6c4c3b81c7470338),
the parser checkpoint was
[004e2f2](https://github.com/SaehwanPark/tabdat-explore-rs/commit/004e2f2b762822f72f5b68c3ce4e0314a101d82c),
and the runtime checkpoint was
[f58e652](https://github.com/SaehwanPark/tabdat-explore-rs/commit/f58e6523d468409f8e4aaf706858183f8995456a).
They were squash-merged by
[PR #58](https://github.com/SaehwanPark/tabdat-explore-rs/pull/58) as
[cc818e5](https://github.com/SaehwanPark/tabdat-explore-rs/commit/cc818e5e5e395d9b0f8c86a0dee451417ca98002).

The implementation contains:

- an owned recursive ByCommand that preserves parsed grouping variables and
  child-command boundaries, supports quoted identifiers, and rejects nested,
  help, status, and doctor children with command-specific diagnostics;
- an owned ByResult table for grouped summarize and count, with default
  numeric selection excluding group variables and explicit unknown/non-numeric
  validation;
- eager local-Parquet DuckDB grouping with SQL NULL groups, ascending
  NULL-last ordering, AVG means, and COUNT(*) row counts; and
- read-only execution that preserves the active relation, dataset metadata,
  and session-local labels while returning copied scalar cells.

Focused Rust coverage includes 44 language unit tests, 54 parser-contract
tests, and six by_contract runtime tests. The full workspace targets also
passed.

## Local verification

The locked repository baseline passed after the final implementation change:

    cargo fmt --all -- --check
    cargo check --locked --workspace --all-targets
    cargo test --locked --workspace --all-targets
    cargo clippy --locked --workspace --all-targets -- -D warnings
    git diff --check

The exact parallel workspace test command passed after its clean rebuild; the
single-job workspace test command also passed during focused verification.

The policy checks also passed:

    cargo deny check
    cargo audit -D warnings

The metadata-driven geiger assertions reported exactly one first-party package
for each of tabdat-explore-rs, tabdat-language, and tabdat-runtime; each
package forbids unsafe code and reported zero first-party unsafe counts. The
PowerShell invocation reproduced the repository's metadata-driven assertions
because jq was unavailable in the local shell.

## Hosted acceptance

PR [#58](https://github.com/SaehwanPark/tabdat-explore-rs/pull/58) was opened
as a draft at the contract checkpoint, received parser and runtime
checkpoints, and was marked ready only after review and local verification.
Its final head [f58e652](https://github.com/SaehwanPark/tabdat-explore-rs/commit/f58e6523d468409f8e4aaf706858183f8995456a)
passed:

- [PR-head CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35496489763),
  including [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35496489763/job/106040329833)
  and [dependency/unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35496489763/job/106040329949); and
- [PR-head runtime](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35496489761),
  including its [Linux runtime job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35496489761/job/106040326388).

The squash merge head [cc818e5](https://github.com/SaehwanPark/tabdat-explore-rs/commit/cc818e5e5e395d9b0f8c86a0dee451417ca98002) passed:

- [main CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35497533142),
  including [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35497533142/job/106043183870)
  and [dependency/unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35497533142/job/106043184011); and
- [main runtime](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35497533149),
  including its [Linux runtime job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35497533149/job/106043183837).

The documentation-closeout commit and its hosted verification will be appended
after the closeout documentation is committed.

## Deviations and deferrals

The Python oracle also supports grouped tabulate, conditions, weights, named
tables, lazy and materialized modes, panel propagation, persistence, formatting,
CLI, JSON, MCP, and broader by surfaces. This Rust slice intentionally
defers those forms. Parsed children outside grouped summarize and count return
an explicit bounded capability error and do not mutate state.
