# Bounded runtime `duplicates` migration evidence

Status: contract recovered; implementation and hosted acceptance pending

Producer: task owner, with pinned oracle evidence and independent runtime review

Consumers: reviewers and the next runtime maintainer

Boundary: pinned Python execution contract → Rust-owned eager-session duplicate
reports

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
`head`/`tail` in PRs #33/#34, eager numeric `summarize` in PR #35, eager
column profiles in PR #36, and SQL-NULL missingness in PR #37. This slice
consumes their owned active metadata and private DuckDB relation.

## Recovery evidence

Focused Python validation at the pinned revision:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_duplicates.py
14 passed in 0.84s
```

The implementation evidence, local checks, review disposition, hosted links,
and temporary branch cleanup will be appended as the draft PR advances.

## Deferred scope

Lazy/materialized execution, `last_operation`, labels, wildcard/range
expansion, formatting, CLI, JSON, MCP, and the unchecked `isid`/other
inspection surfaces remain explicit deferrals. No second data engine,
dependency, unsafe code, relation writer, serializer, or public value lifetime
is introduced by this bounded slice.
