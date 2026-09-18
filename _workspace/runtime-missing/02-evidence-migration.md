# Bounded runtime `missing` migration evidence

Status: implementation and review pending; PR #37 is draft

Producer: task owner, with pinned oracle evidence and independent runtime review

Consumers: reviewers and the next runtime maintainer

Boundary: pinned Python execution contract → Rust-owned eager-session missingness reports

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
`head`/`tail` previews in PRs #33/#34, eager numeric `summarize` in PR #35,
and eager column profiles in PR #36. This slice consumes their owned active
metadata and private DuckDB relation; it does not add lazy materialization,
labels, transforms, or presentation surfaces.

## Revisions and changed paths

- `49a9555`: recovered the missingness contract and opened draft PR #37;
- `6558268`: added owned missingness results, eager aggregate execution, exact
  diagnostics, and read-only state handling; and
- `00000f8`: added unit/integration coverage for active-state errors, counts,
  percentages, order, duplicates, unknown variables, empty/all-null relations,
  quoted identifiers, container columns, and failed replacement.

PR #37 ([Add bounded eager missingness report](https://github.com/SaehwanPark/tabdat-explore-rs/pull/37))
was opened as a draft before implementation.

Changed implementation and evidence paths:

- `crates/tabdat-runtime/src/lib.rs`: owned `MissingRow`/`MissingResult`,
  `ExecutionResult::Missing`, typed diagnostics, and one quoted DuckDB count
  aggregate;
- `crates/tabdat-runtime/tests/use_contract.rs`: fresh-session, parser-to-
  session, explicit/default/duplicate order, exact count/percentage/type,
  null/empty, unknown, quoted/container, repeated-read, and failed-replacement
  tests;
- `crates/tabdat-language/src/lib.rs`: the `Missing` command comment now
  reflects runtime support; and
- `_workspace/runtime-missing/`: contract, migration evidence, and review record.

No dependency, unsafe-code, relation-writer, filesystem-writer,
serialization, CLI, or MCP surface changed.

## Pinned oracle probe

The focused command was run at the pinned revision:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_missing.py
```

Observed result:

```text
9 passed in 0.36s
```

The authoritative no-active error is exactly:
`missing requires an active dataset; run use <path> first`.

The fixture evidence covers requested and default schema order, explicit
duplicates, SQL-NULL counts, zero-row percentage, all-null values, unknown
variable validation, quoted identifiers, and container-valued columns whose
values are never copied into the Rust result.

## Rust and policy checks

At implementation/test head `00000f8`, the required local checks passed:

```text
cargo fmt --all -- --check                              passed
git diff --check                                        passed
cargo check --locked --workspace --all-targets          passed
cargo test --locked --workspace --all-targets           passed
  root scaffold: 1 test passed
  tabdat-language: 42 unit + 32 public integration tests passed
  tabdat-runtime: 11 unit + 54 integration tests passed
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
dependency inventory warnings as non-first-party findings. It completed
successfully for the root, language, and runtime first-party packages.

## State and parity evidence

The implementation is read-only after `use` publishes an eager relation:

| Before | Input | Result | After |
| --- | --- | --- | --- |
| no active dataset | `missing` | typed `NoActiveDataset` with exact text | unchanged; backend remains uninitialized |
| active eager local-Parquet dataset | explicit variables | owned rows in requested order, including duplicates | unchanged |
| active eager local-Parquet dataset | empty variable list | every schema column in schema order | unchanged |
| active eager local-Parquet dataset | null/empty values | exact counts and percentages; only SQL NULL is missing | unchanged |
| unknown variables | `missing` | typed exact diagnostic before query | unchanged |
| missing or dropped active relation | `missing` | typed `MissingFailed` displayed as `missing failed` | active metadata remains exactly as before |

Counts use one SQL aggregate with `count(*)` and a quoted `count(column)` for
each requested position. Missing counts are checked as `total - nonmissing`;
percentage is zero for an empty relation and otherwise
`100.0 * missing / total`. Empty strings, sentinel codes, and NaN are not
treated as SQL NULL. No backend values are converted, so container columns can
be counted even though the preview/codebook `CellValue` boundary intentionally
does not own their values.

Lazy/materialized execution, labels, wildcard/range expansion, formatting,
CLI, JSON, and MCP remain explicit deferrals. The Python runtime records a
`last_operation` value and supports lazy engines; the bounded Rust session has
no such state field and intentionally preserves only its current active
metadata/relation contract.

## Hosted acceptance and completion state

Independent review, PR-head workflows, post-merge workflows, and temporary
branch cleanup are pending. The acceptance record will be updated with exact
hosted run/job links after PR #37 is reviewed, marked ready, squash-merged,
and the resulting `main` workflows pass.
