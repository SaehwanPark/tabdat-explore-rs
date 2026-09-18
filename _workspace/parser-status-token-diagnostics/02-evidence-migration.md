# `status` sign-token diagnostic migration evidence

Status: implementation and hosted checks passed; independent review and merge
closeout are pending.

Producer: task owner, with independent oracle and parser review

Consumers: reviewers and the next tokenizer maintainer

Boundary: pinned Python parser contract → Rust syntax-only diagnostic correction

## Authority and contract inputs

The Python oracle is the clean sibling checkout `../tabdat-explore` at pinned
commit `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, package `0.25.0`, and Python
`3.13.3`. Its `uv.lock` SHA-256 is
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`; the
checkout was clean and no dependency synchronization or source edits were
performed. The bounded contract is in `01-contract.md`.

## Revisions and changed paths

- `c188be9`: frozen contract and draft-PR handoff;
- `ce73613`: status sign/empty-condition parser correction, exact unit/public
  regressions, and MIG-0002 scope update.

Changed implementation paths:

- `crates/tabdat-language/src/lib.rs`: attached status sign dispatch,
  command-specific validation precedence, and unit coverage;
- `crates/tabdat-language/tests/parser_contract.rs`: public exact diagnostic
  coverage;
- `docs/migration/decisions.md`: records that the bounded sign forms now match
  while broader status tokenizer parity remains unresolved;
- `_workspace/parser-status-token-diagnostics/01-contract.md`: bounded scope.

No command enum, runtime/backend, session, relation, filesystem, dependency,
unsafe-code, serialization, CLI, or MCP surface changed.

## Pinned oracle probe

The focused command was rerun at the pinned revision:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py -k 'parse_status_command or invalid_commands'
```

Observed result: `419 passed, 70 deselected in 0.42s`.

The broader parser/script regression was also rerun:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py tests/test_script.py
```

Observed result: `516 passed in 0.45s`.

Representative pinned results:

```text
status -1   -> unsupported token in command: -
status +1   -> unsupported token in command: +
status-1    -> unsupported token in command: -
status+1    -> unsupported token in command: +
status -1,  -> unsupported token in command: -
status +1,  -> unsupported token in command: +
status if    -> missing expression after if
status if x  -> status does not accept arguments, if clauses, options, or assignment syntax
```

## Rust checks

At implementation revision `ce73613`, the required local checks passed:

```text
cargo fmt --all -- --check                              passed
cargo check --locked --workspace --all-targets          passed
cargo test --locked --workspace --all-targets           passed
  root scaffold: 1 test passed
  tabdat-language: 42 unit + 32 public integration tests passed
  tabdat-runtime: 2 unit + 13 integration tests passed
cargo clippy --locked --workspace --all-targets -- -D warnings
                                                         passed
git diff --check                                        passed
```

The policy checks also passed locally:

```text
cargo deny check                                        advisories, bans, licenses, sources ok
cargo audit -D warnings                                 passed; no vulnerabilities reported
metadata-driven cargo geiger (all workspace packages,
  locked/all-targets/all-dependencies JSON assertions)  passed; first-party
  crates reported forbid(unsafe_code) and zero first-party unsafe usage
```

## Hosted acceptance in progress

Draft PR #30 is
[`Parse status sign-token diagnostics`](https://github.com/SaehwanPark/tabdat-explore-rs/pull/30),
opened before implementation. Its current head is
`ce736138117305e25fc1ac6d8dc49822b15100e0`. All current-head hosted jobs
passed:

- [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35315903975/job/105507435747), 20m32s;
- [dependency and unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35315903975/job/105507435916), 19m45s;
- [tabdat-runtime](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35315903961/job/105507373668), 21m8s.

Independent review, ready-state promotion, squash merge, branch deletion, and
post-merge `main` verification remain to be recorded. Keep MIG-0002 unresolved
for the broader status tokenizer matrix and keep the roadmap's broad tokenizer
checkbox unchecked.
