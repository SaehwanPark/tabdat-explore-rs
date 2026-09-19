# Bounded eager-runtime `drop` contract

Status: contract recovery in progress; implementation has not started.

Producer: task owner with parallel Python-contract, Rust-runtime, and scope
review investigations
Consumer: bounded `drop` implementation and independent review on
`feat/runtime-drop`

## Recovery authority

The pinned Python authority is the clean sibling checkout
`/Volumes/research/gitrepos/tabdat-explore` at revision
`16b45d9b66b0d80d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, Python `3.13.3`, with `uv.lock`
SHA-256 `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.

The Python model/parser/executor/backend paths, focused oracle results, exact
diagnostics, and the bounded Rust recommendation will be recorded here before
any runtime code is changed.

## Pending decisions

- whether the first useful slice is explicit-varlist projection only or also
  predicate filtering;
- exact duplicate-variable and row/schema publication behavior;
- failure-atomic session and private-relation invariants;
- explicit deferrals for lazy/materialized execution, expression functions,
  metadata side effects, and broader transform sequencing.

Completion state: recovery pending; no Rust behavior is claimed by this artifact.
