# Bounded runtime `datasignature` migration evidence

Status: accepted bounded eager-runtime slice

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
for an interval value. Bare struct fields, temporal map keys, and an escaped
struct field name also match the oracle: `80ea4f18ca0d129649989fc28dcff1ec6d3758aa08390161f2c41fba499c6599`,
`1019fb05c4a0ae3614b19ff5a736d372e5dd43efe3157c60bc167c5a560dffdb`,
`56a02953bcca1f09a4a63743e766fddf9f0eb48bf19c5d8c4fbf3d3e725ec07f`, and
`5b331a8cd85c3a489eb42e9b69d9937f930ff05b0875873ea5c691a28870fd54`.

## Implementation evidence

Implementation commit `0acfd7f` (`runtime: add bounded eager datasignature`)
adds the owned result/error boundary, safe `sha2` protocol encoder, eager
DuckDB scan, and state-preservation tests. Corrections in `3512a46` propagate
raw nested type hints (including temporal map keys and bare struct fields), and
`22d6b36` handles doubled quotes in nested struct names. Focused runtime
evidence on the branch:

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

The Rust contract tests assert the exact oracle digests, no-active and
dropped-relation diagnostics, deterministic repeats, schema/row-order
sensitivity, empty relations, parser dispatch, state preservation, and
nanosecond/nested-timezone/interval/escaped-name encodings. The independent
review in `03-review.md` found and then verified fixes for all four parity edge
cases. Dependency policy evidence is also green: `cargo deny check`,
`cargo audit -D warnings`, and metadata-driven `cargo geiger` report clean
first-party packages.

## Hosted acceptance and cleanup

PR #40 was marked ready after all PR-head gates passed for commit `24822ad`:

- [CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35411047370),
  including [dependency/unsafe policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35411047370/job/105810650144)
  and [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35411047370/job/105810650340);
- [TabDat runtime boundary](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35411047371)
  ([Linux job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35411047371/job/105810648318)).

PR #40 was squash-merged as
[`9a141da`](https://github.com/SaehwanPark/tabdat-explore-rs/commit/9a141daa547571ad5e0e75efef08b6c842c99923).
The temporary `feat/runtime-datasignature` branch was deleted locally and
remotely. The post-merge workflows for the squash commit are:

- [CI](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35412237819),
  including [dependency/unsafe policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35412237819/job/105813973474)
  and [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35412237819/job/105813973742);
- [TabDat runtime boundary](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35412237838)
  ([Linux job](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35412237838/job/105813973082)).

Both post-merge workflows completed successfully on the squash commit before
the main-branch roadmap closeout: the generic CI finished in 20m0s and the
runtime boundary finished in 19m33s.

## Deferred scope

Lazy/materialized execution, `last_operation`, labels/panel metadata, `by`
syntax, formatting, CLI/REPL, JSON/MCP, and the unchecked `assert`/other
inspection surfaces remain explicit deferrals.
