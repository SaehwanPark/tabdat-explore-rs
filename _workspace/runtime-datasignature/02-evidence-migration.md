# Bounded runtime `datasignature` migration evidence

Status: WIP — contract recovered; implementation pending

Producer: task owner, with pinned oracle evidence and independent runtime review

Consumers: reviewers and the next runtime maintainer

Boundary: pinned Python execution contract → Rust-owned eager-session
reproducibility fingerprint

## Authority and contract inputs

The pinned authority is `../tabdat-explore` at commit
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, package `0.25.0`, Python `3.13.3`,
and `uv.lock` SHA-256
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`. The
checkout was clean and no dependency synchronization or source edits were
performed. The bounded contract is in `01-contract.md`.

## Recovery evidence

The recovery pass identified the exact length-framed SHA-256 protocol,
canonical type aliases, recursive value encodings, schema/row-order rules, and
the pinned exact fixture digest. Focused oracle commands and observed results
are recorded in `01-contract.md`; they will be rerun independently before the
implementation is accepted.

## Implementation evidence

Pending. This section will record the implementation commit, focused runtime
tests, workspace checks, policy scans, and the independent review.

## Hosted acceptance and cleanup

Pending. The WIP PR will be marked ready only after all PR-head checks pass;
after merge, the post-merge workflow matrix and deletion of the temporary local
and remote branch will be recorded here.

## Deferred scope

Lazy/materialized execution, `last_operation`, labels/panel metadata, `by`
syntax, formatting, CLI/REPL, JSON/MCP, and the unchecked `assert`/other
inspection surfaces remain explicit deferrals.
