# Bounded runtime `datasignature` migration evidence

Status: WIP — implementation complete; hosted acceptance pending

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
the pinned exact fixture digest. Independent rerun at the pinned checkout:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_datasignature.py
11 passed in 0.44s
```

The oracle produced `0b61cef05ab04301df893652f666b6ed974668f0cd214ebae17cfff02a6b3aad`
for the three-row/four-column fixture,
`c2d343670c50720e4e4360322c980346b3eea1c0984aba4acb437aced8c816ea` for the
complex temporal/nonfinite/decimal/list fixture, and
`9ee1723d85281a5b7423fe90999069a22ac2a4be650ba1a2b39d94fdff23d295` for the
empty one-column schema.

The edge fixtures also match the independently generated oracle digests:
`2d1c67af3c53e41de8eaecf6a39af59ff4c4cdb73ad50242747e869be8e7710a` for a
nanosecond timestamp, `708fdda786f82d09019158fb402f4ae2cd6ea9b5bf785453c6fa77a6bac446bc`
for a nested timezone list, `3d1041974ea09a192c5ebd736d68bdd1f81d9f9a395407257a7ad772b46aab6b`
for a nested timezone struct, `7a5a285fcec0d99773a3adc71fea4a065e12d37dc28f32b250fd5d6e11832552`
for a nested timezone map, and `82372379c40bbea52435f328e4139e5567af9a6f1788b607d092e3fc56898765`
for an interval value.

## Implementation evidence

Implementation commit `0acfd7f` (`runtime: add bounded eager datasignature`)
adds the owned result/error boundary, safe `sha2` protocol encoder, eager
DuckDB scan, and state-preservation tests. Focused runtime evidence on the
branch:

```text
cargo test --locked -p tabdat-runtime --all-targets
18 unit tests + 6 datasignature-contract tests + 63 existing integration tests passed
cargo check --locked --workspace --all-targets
passed
cargo clippy --locked -p tabdat-runtime --all-targets -- -D warnings
passed
cargo fmt --all -- --check
passed
git diff --check
passed
```

The Rust contract tests assert the three exact oracle digests, no-active and
dropped-relation diagnostics, deterministic repeats, schema/row-order
sensitivity, empty relations, parser dispatch, state preservation, and
nanosecond/nested-timezone/interval encodings. The independent review in
`03-review.md` found and then verified fixes for the three parity edge cases.

## Hosted acceptance and cleanup

The WIP PR is #40. It will be marked ready only after the dependency/unsafe
policy, Rust baseline, and runtime-boundary PR-head checks pass, after which the
post-merge workflow matrix and deletion of the temporary local and remote branch
will be recorded here.

## Deferred scope

Lazy/materialized execution, `last_operation`, labels/panel metadata, `by`
syntax, formatting, CLI/REPL, JSON/MCP, and the unchecked `assert`/other
inspection surfaces remain explicit deferrals.
