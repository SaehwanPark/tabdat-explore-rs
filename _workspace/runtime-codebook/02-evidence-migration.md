# Bounded runtime `codebook` migration evidence

Status: implementation and local validation complete; acceptance pending
independent review, PR-head checks, merge, and post-merge hosted evidence.

Producer: task owner, with pinned oracle evidence and independent runtime review

Consumers: reviewers and the next runtime maintainer

Boundary: pinned Python execution contract → Rust-owned eager-session codebook profiles

## Authority and contract inputs

The Python oracle is the clean sibling checkout `../tabdat-explore` at pinned
commit `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, package `0.25.0`, and Python
`3.13.3`. Its `uv.lock` SHA-256 is
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`; the
checkout was clean and no dependency synchronization or source edits were
performed. The bounded contract is in `01-contract.md`.

The accepted Rust prerequisites are the eager local-Parquet `use` session in
PR #22, read-only `describe` in PR #31, cached `count` in PR #32, owned
`head`/`tail` previews in PRs #33/#34, and eager numeric `summarize` in PR #35.
This slice consumes their owned active metadata and private DuckDB relation; it
does not add lazy materialization, labels, transforms, or presentation surfaces.

## Revisions and changed paths

- `e41667f`: recovered the codebook contract and opened draft PR #36;
- `dacecc3`: added owned codebook results, eager DuckDB profiles, typed
  diagnostics, and read-only state handling; and
- `c53c7b8`: added unit/integration coverage for counts, types, examples,
  ordering, duplicates, null/empty relations, unsupported values, and failures.

PR #36 ([Add bounded eager codebook execution](https://github.com/SaehwanPark/tabdat-explore-rs/pull/36))
was opened as a draft before implementation.

Changed implementation and evidence paths:

- `crates/tabdat-runtime/src/lib.rs`: `ExecutionResult::Codebook`, owned
  `CodebookRow`/`CodebookResult`, exact diagnostics, eager aggregate/example
  queries, and backend-independent value ownership; and
- `crates/tabdat-runtime/tests/use_contract.rs`: fresh-session, parser-to-
  session, explicit/default/duplicate order, type/count/example, null/empty,
  unsupported-value, validation, repeated-read, and failed-replacement tests;
- `_workspace/runtime-codebook/`: contract, migration evidence, and review record.

No dependency, unsafe-code, relation-writer, filesystem-writer,
serialization, CLI, or MCP surface changed.

## Pinned oracle probe

The focused command was run at the pinned revision:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py tests/test_executor.py tests/test_labels.py \
  -k 'codebook or phase_3_inspection_commands_require_active_dataset'
```

Observed result:

```text
12 passed, 900 deselected in 1.00s
```

The authoritative no-active error is exactly:
`codebook requires an active dataset; run use <path> first`.

The fixture evidence covers requested and default schema order, explicit
duplicates, schema types, nonmissing/missing/distinct counts, first-three
non-null examples including repeated values, all-null and empty columns,
unknown-variable validation, unsupported list ownership, repeated reads, and
read-only active metadata.

## Rust and policy checks

At implementation head `c53c7b8`, the required local checks passed:

```text
cargo fmt --all -- --check                              passed
git diff --check                                        passed
cargo check --locked --workspace --all-targets          passed
cargo test --locked --workspace --all-targets           passed
  root scaffold: 1 test passed
  tabdat-language: 42 unit + 32 public integration tests passed
  tabdat-runtime: 9 unit + 47 integration tests passed
cargo clippy --locked --workspace --all-targets -- -D warnings
                                                         passed
```

The policy checks passed locally:

```text
cargo deny check                                        advisories, bans, licenses, sources ok
cargo audit -D warnings                                 passed; no vulnerabilities reported
metadata-driven cargo geiger                            passed; first-party packages
                                                         forbid unsafe and use zero unsafe
```

The geiger loop follows the all-package assertion in `CONTRIBUTING.md`: it
resolves every workspace package from locked metadata, checks
`forbid(unsafe_code)` and zero first-party unsafe usage, and treats transitive
dependency inventory warnings as non-first-party findings. The loop completed
successfully for every first-party workspace package.

## State and parity evidence

The implementation is read-only after `use` publishes an eager relation:

| Before | Input | Result | After |
| --- | --- | --- | --- |
| no active dataset | `codebook` | typed `NoActiveDataset` with exact text | unchanged; backend remains uninitialized |
| active eager local-Parquet dataset | explicit variables | owned rows in requested order, including duplicates | unchanged |
| active eager local-Parquet dataset | empty variable list | every schema column in schema order | unchanged |
| active eager local-Parquet dataset | null/empty values | exact counts and empty examples where appropriate | unchanged |
| unknown variables | `codebook` | typed exact diagnostic before query | unchanged |
| unsupported example conversion or backend failure | `codebook` | typed `CodebookFailed` displayed as `codebook failed for variable: <name>` | active metadata remains exactly as before |

Counts use SQL `count(column)`, `count(*) - count(column)`, and
`count(distinct column)`, which exclude NULL from nonmissing, distinct, and
examples. Examples are copied into the existing owned `CellValue` boundary and
are not deduplicated. Variable labels are intentionally omitted because the
current Rust session has no label metadata; lazy/materialized execution,
unsupported logical/container coercion, formatting, CLI, JSON, and MCP remain
explicit deferrals.

## Hosted acceptance and completion state

Independent review is complete with no actionable findings. PR-head checks for
`c53c7b8` are green: [CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35374637669)
([policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35374637669/job/105696799992),
[Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35374637669/job/105696800355))
and [runtime](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35374637696/job/105696326339).

After this evidence commit is validated, PR #36 will be marked ready,
squash-merged, and its temporary branch deleted locally and remotely. The
roadmap checkbox and current-state wording will be updated only after the
post-merge main workflows and docs-inclusive validation are green.
