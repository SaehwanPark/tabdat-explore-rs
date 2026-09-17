# `run` syntax migration evidence

Status: draft; implementation is pushed in PR #24, while independent review
and hosted acceptance remain pending.

Producer: task owner

Consumer: reviewers and the next maintainer

Boundary: pinned Python parser contract → Rust syntax-only parser

## Authority and contract inputs

The Python oracle is the clean sibling checkout `../tabdat-explore` at commit
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, package `0.25.0`, and Python
`3.13.3`. Its `uv.lock` SHA-256 is
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`; the
checkout was clean and no dependency synchronization or source edits were
performed.

The contract and implementation cite these pinned paths:

- `src/tabdat/models.py:438-440` (`RunCommand(path: Path)`);
- `src/tabdat/parser.py:146-147,303-304,518-523,1002-1009` (command
  inventory, dispatch, and `_parse_run`);
- `tests/test_parser.py:370-371,1539-1544` (positive and invalid forms);
- `docs/commands/run.md:1-25` and `src/tabdat/help/topics/run.md:1-25`
  (public syntax and execution description).

Python strips the command prefix, splits the remainder with ordinary
`str.split()`, requires exactly one token, and wraps that token in `Path`.
Quotes are not unescaped. A comma is part of a path token unless attached to
the command name, where the generic command boundary reports the recorded
diagnostic. The Rust contract intentionally keeps the exact one-token text as
`Command::Run { path: String }`; path normalization, `~` expansion, file
existence, and script loading are deferred effects rather than parser behavior.

## Rust implementation revisions

- `c9efafc`: froze the contract in `01-contract.md`;
- `173a4ee`: added `Command::Run { path: String }`, direct one-token parsing,
  the frozen command-boundary diagnostics, runtime command-name mapping, and
  unit/public/runtime deferral tests;
- documentation/evidence revisions are pending in this branch before PR #24
  is marked ready.

Changed implementation paths:

- `crates/tabdat-language/src/lib.rs`: owned command variant, direct dispatch,
  run-specific comma/equal/colon boundary handling, parser, and unit tests;
- `crates/tabdat-language/tests/parser_contract.rs`: public typed-command and
  exact-diagnostic coverage;
- `crates/tabdat-runtime/src/lib.rs`: exhaustive command-name mapping;
- `crates/tabdat-runtime/tests/use_contract.rs`: explicit unsupported runtime
  regression;
- `README.md`, `SPEC.md`, `ARCHITECTURE.md`, and
  `docs/TABDAT_RUST_PORT_ROADMAP.md`: current-state and pending-scope notes.

No manifest, dependency, backend, unsafe code, filesystem, session, CLI,
serialization, or script-engine surface changed.

## Oracle checks

Focused parser/invalid-command selection:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py -k 'test_parse_phase_8_run_command or test_parse_invalid_commands'
419 passed, 70 deselected in 0.44s
```

Full pinned parser/script regression:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py tests/test_script.py
516 passed in 0.78s
```

Probes at the same revision matched the frozen direct forms and diagnostics:

```text
run analysis.td                         -> RunCommand(path=Path("analysis.td"))
RUN   analysis.td                       -> the same command
run "analysis.td"                      -> Path('"analysis.td"')
run analysis.td,                        -> Path('analysis.td,')
run                                      -> run expects exactly one path: run <script>
run a.td b.td                            -> run expects exactly one path: run <script>
run "a b.td"                             -> run expects exactly one path: run <script>
run,                                     -> comma must be followed by at least one option
run,foo                                  -> unknown command: run
run=foo                                  -> run assignment requires a target before =
run==foo                                 -> unsupported token in command: ==
run:foo                                  -> unsupported token in command: :
```

## Rust checks

The changed language/runtime tests passed before the implementation commit:

```text
cargo fmt --all -- --check
cargo test --locked -p tabdat-language --all-targets
  32 unit tests, 21 public integration tests passed
cargo test --locked -p tabdat-runtime --all-targets
  2 unit tests, 7 runtime integration tests passed
```

The complete locked workspace baseline then passed:

```text
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
  root scaffold: 1 passed
  tabdat-language: 32 unit + 21 integration passed
  tabdat-runtime: 2 unit + 7 integration passed
cargo clippy --locked --workspace --all-targets -- -D warnings
git diff --check
```

Separate policy checks passed:

```text
cargo deny check
  advisories ok, bans ok, licenses ok, sources ok
cargo audit -D warnings
  completed with no findings
```

The metadata-driven `cargo geiger` scan passed for every first-party package
with `forbids_unsafe=true`, zero first-party unsafe functions, and zero
first-party unsafe expressions. Its report totals were:

- `tabdat-explore-rs`: `unscanned_files=0`, geiger exit `0`;
- `tabdat-language`: `unscanned_files=0`, geiger exit `0`;
- `tabdat-runtime`: `unscanned_files=33`, geiger exit `0` (the expected
  transitive dependency inventory warning described by ADR 0007).

## Hosted acceptance

At the current documentation/evidence head `030842d`, PR #24 triggered seven
hosted jobs. The two native feasibility jobs and their Rust checks have already
passed; the baseline, policy, and runtime jobs are still pending:

- [dependency/unsafe policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35259128197/jobs/105330439157) (pending);
- [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35259128197/jobs/105330439687) (pending);
- [tabdat-runtime on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35259128204/jobs/105330217820) (pending);
- [ReadStat feasibility on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35259128214/jobs/105330126436) (passed);
- [ReadStat Rust check on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35259128214/jobs/105330126736) (passed);
- [libgretl feasibility on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35259128229/jobs/105330126269) (passed);
- [libgretl OLS Rust check on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35259128224/jobs/105330126491) (passed).

The final head, all hosted results, ready transition, squash merge, and
post-merge main checks must be appended before this artifact changes to
accepted.

## Supported and deferred behavior

Supported here is direct syntax only: one owned raw path token, exact arity
validation, case/separator normalization, retained quote/punctuation text, and
the recorded command-boundary diagnostics. Runtime execution returns the
existing typed `UnsupportedCommand { name: "run" }` error.

Deferred are path normalization and expansion, file reads, line-oriented script
execution, comments, multiline SQL, `seed`, `let`, macro expansion,
`if`/`else`/`end`, nested/recursive `run`, file/line diagnostics, full
tokenizer/varlist/option/expression parity, CLI/JSON/MCP output, and backend
capability initialization.
