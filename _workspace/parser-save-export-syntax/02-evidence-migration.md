# `save`/`export` syntax migration evidence

Status: accepted after PR #29 (`8b16223`) squash merge. Oracle, local, policy,
independent-review, hosted, merge, and branch-cleanup evidence are recorded
below; post-merge main verification is pending this documentation closeout.

Producer: task owner, with independent parser review

Consumers: reviewers and the next maintainer

Boundary: pinned Python parser contract → Rust syntax-only parser/runtime
deferral

## Authority and contract inputs

The Python oracle is the clean sibling checkout `../tabdat-explore` at pinned
commit `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, package `0.25.0`, and Python
`3.13.3`. Its `uv.lock` SHA-256 is
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`; the
checkout was clean and no dependency synchronization or source edits were
performed. The frozen contract is in `01-contract.md`.

## Revisions and changed paths

- `beafafc`: frozen contract and draft-PR handoff;
- `e014705`: owned `Save`/`Export` variants, attached symbolic command
  boundaries, path/option parser, public/unit coverage, runtime mappings, and
  explicit unsupported-runtime regressions;
- `0afb8c5`: current-state and roadmap documentation for the bounded syntax
  scope.
- `8b16223`: squash merge of PR #29 to `main`.

Changed implementation paths:

- `crates/tabdat-language/src/lib.rs`: owned commands, direct dispatch, symbol-
  aware path/option parsing, deterministic diagnostics, and unit tests;
- `crates/tabdat-language/tests/parser_contract.rs`: public typed-command and
  exact-diagnostic coverage;
- `crates/tabdat-runtime/src/lib.rs`: exhaustive command-name mappings;
- `crates/tabdat-runtime/tests/use_contract.rs`: explicit save/export runtime
  deferral tests;
- `README.md`, `SPEC.md`, `ARCHITECTURE.md`, and
  `docs/TABDAT_RUST_PORT_ROADMAP.md`: current syntax-only status and deferred
  persistence boundary.

No manifest, dependency, backend, unsafe code, filesystem writer, relation,
session mutation, CLI, serialization, or MCP surface changed.

## Pinned oracle probe

The focused command was rerun at the pinned revision:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py -k 'phase_9_configuration_and_persistence or invalid_commands'
```

Observed result: `419 passed, 70 deselected in 0.41s`.

The broader parser/script regression was also rerun:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py tests/test_script.py
```

Observed result: `516 passed in 0.47s`.

The slice probe produced these representative outputs:

```text
'save output.parquet' -> SaveCommand(path=PosixPath('output.parquet'), replace=False)
'export "my output.parquet", replace' -> ExportCommand(path=PosixPath('my output.parquet'), replace=True)
'save `a,b`, replace replace' -> SaveCommand(path=PosixPath('a,b'), replace=True)
'save:out.parquet' -> SaveCommand(path=PosixPath(':out.parquet'), replace=False)
'export/out.csv' -> ExportCommand(path=PosixPath('/out.csv'), replace=False)
'save a==b' -> SaveCommand(path=PosixPath('a==b'), replace=False)
'save out, force' -> save unsupported option: force
'save out, replace=true' -> save option replace does not accept a value
'save out if x > 0' -> save does not accept if clauses or assignment syntax
'save out =' -> save assignment requires an expression after =
'save out,' -> comma must be followed by at least one option
```

## Rust checks

At implementation revision `e014705` (the documentation-only revision
`0afb8c5` does not alter code), the focused and workspace checks passed:

```text
cargo fmt --all -- --check                              passed
cargo check --locked --workspace --all-targets          passed
cargo test --locked --workspace --all-targets           passed
  root scaffold: 1 test passed
  tabdat-language: 42 unit + 31 public integration tests passed
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

The geiger loop followed `CONTRIBUTING.md`; dependency inventory warnings, if
reported by transitive DuckDB dependencies, are not first-party unsafe findings.

## Independent review

The read-only parser review compared `e014705` with `01-contract.md` and the
pinned oracle. It found no P0/P1 correctness or scope findings. It confirmed
path extraction/quote removal, symbolic and attached paths, repeated and
case-sensitive `replace`, option ordering/diagnostics, and runtime deferral.
The review retained these known tokenizer limitations as explicit deferrals:

| Probe | Pinned Python | Current bounded Rust | Disposition |
| --- | --- | --- | --- |
| `save 1.2.3` | `malformed number: 1.2.3` | raw lexical path text | deferred malformed-number tokenizer parity |
| `save if/foo` | `unsupported token in expression: /` | raw lexical path text | deferred attached-`if` condition parsing |

These cases are outside the accepted direct-path contract and are not presented
as migrated tokenizer behavior.

## Hosted acceptance and completion state

PR #29 (`https://github.com/SaehwanPark/tabdat-explore-rs/pull/29`) was opened
as a draft before implementation, promoted to ready after review, and squash-
merged as `8b162232cb31df75dbd96f4ae3c75842533904b4` on 2026-09-18. All
required PR-head jobs passed:

- [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35310703919/job/105492018946), 21m45s;
- [dependency and unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35310703919/job/105492019104), 19m19s;
- [tabdat-runtime](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35310703932/job/105492019610), 20m39s;
- [ReadStat Rust](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35310703944/job/105491967338), 26s;
- [ReadStat](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35310703944/job/105491967136), 25s;
- [libgretl OLS Rust](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35310703923/job/105491966960), 1m14s;
- [libgretl feasibility](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35310703934/job/105491966968), 1m8s.

The temporary `feat/parser-save-export-syntax` branch was deleted locally and
on the remote after merge. The closeout commit is now on `main`; its post-merge
checks are pending below. The migration state is `accepted` for the bounded
syntax-only contract.

Post-merge main verification will be appended after this documentation update
is pushed.

No backend, filesystem, active-dataset, overwrite, output-format, reporting, or
serialization claim is made here. Phase 6.5 persistence/output work remains
unchecked.
