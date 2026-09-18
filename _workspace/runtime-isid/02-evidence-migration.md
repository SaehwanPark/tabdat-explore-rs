# Bounded runtime `isid` migration evidence

Status: contract recovered; implementation and hosted acceptance pending

Producer: task owner, with pinned oracle evidence and independent runtime review

Consumers: reviewers and the next runtime maintainer

Boundary: pinned Python execution contract → Rust-owned eager-session key
uniqueness result

## Authority and contract inputs

The pinned authority is `../tabdat-explore` at commit
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, package `0.25.0`, Python `3.13.3`,
and `uv.lock` SHA-256
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`. The
checkout was clean and no dependency synchronization or source edits were
performed. The bounded contract is in `01-contract.md`.

The accepted Rust prerequisites are the eager local-Parquet `use` session in
PR #22, read-only `describe` in PR #31, cached `count` in PR #32, owned
`head`/`tail` in PRs #33/#34, eager numeric `summarize` in PR #35, eager column
profiles in PR #36, SQL-NULL missingness in PR #37, and duplicate-key grouping
in PR #38. This slice consumes their owned active metadata and private DuckDB
relation.

## Recovery evidence

Focused Python validation at the pinned revision:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_isid.py
20 passed in 0.54s
```

The implementation evidence, local checks, review disposition, hosted links,
and temporary branch cleanup will be appended as the draft PR advances.

## Planned implementation and state checks

The bounded implementation will add an owned `IsidResult`, typed unknown,
semantic-failure, and backend-failure diagnostics, eager session dispatch, and
one quoted DuckDB grouped aggregate. It must preserve request order and
duplicates, count NULL-containing groups correctly, reject duplicates even
with `missok`, and leave active metadata unchanged on every failure.

Required local and hosted evidence will be recorded after implementation:

- `cargo fmt --all -- --check` and `git diff --check`;
- locked workspace check/test/Clippy and policy scans;
- metadata-driven first-party unsafe inventory;
- focused runtime/parser tests; and
- all required PR-head and post-merge workflows.

## Deferred scope

Lazy/materialized execution, `last_operation`, labels/panel metadata,
wildcard/range expansion, formatting, CLI, JSON, MCP, and the unchecked
`datasignature`/`assert`/other inspection surfaces remain explicit deferrals.
