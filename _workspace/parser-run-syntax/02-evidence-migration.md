# `run` syntax migration evidence

Status: accepted bounded slice; PR #24 is merged and its independent review,
hosted checks, branch cleanup, and post-merge evidence are recorded below.

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
- `src/tabdat/parser.py:146-147,252-306,303-304,518-523,1002-1009,3036-3123,3327-3395`
  (command inventory, generic dispatch/tokenization, direct routing,
  `_parse_run`, and the generic boundary-diagnostic paths);
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
- `030842d`: recorded the initial migration evidence and current-state notes;
- `7a9d9b5`: corrected hosted-workflow enumeration and SPEC wording;
- `fc61dfa`: added the independent review artifact and requested current-head
  review;
- `ea112b1`: strengthened the generic dispatch/tokenizer citations, recorded
  the reproducible boundary probe, and corrected the revision/provenance
  wording; this is the source revision used for the final review.
- `2857af6`: recorded the complete passing PR-head hosted set and readiness
  evidence; PR #24 was then marked ready and squash-merged as
  `77f4754b4b0875b5e22e32c09d4b4854bb3427bb`.

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

The boundary rows were reproduced independently from the pinned checkout with:

```sh
cd ../tabdat-explore
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync python -c 'from tabdat.parser import parse_command, ParseError
for text in ("run,foo", "run=foo", "run==foo", "run:foo"):
    try:
        print(f"{text!r} -> {parse_command(text)!r}")
    except ParseError as exc:
        print(f"{text!r} -> {exc}")'
```

Output:

```text
'run,foo' -> unknown command: run
'run=foo' -> run assignment requires a target before =
'run==foo' -> unsupported token in command: ==
'run:foo' -> unsupported token in command: :
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

At the documentation/evidence head `030842d`, PR #24 triggered seven hosted
jobs. The two native feasibility jobs and their Rust checks had already passed;
the baseline, policy, and runtime jobs were superseded by later documentation
revisions. Those historical links are retained below only to show the original
workflow coverage:

- [dependency/unsafe policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35259128197/jobs/105330439157) (cancelled/superseded);
- [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35259128197/jobs/105330439687) (cancelled/superseded);
- [tabdat-runtime on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35259128204/jobs/105330217820) (cancelled/superseded);
- [ReadStat feasibility on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35259128214/jobs/105330126436) (passed);
- [ReadStat Rust check on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35259128214/jobs/105330126736) (passed);
- [libgretl feasibility on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35259128229/jobs/105330126269) (passed);
- [libgretl OLS Rust check on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35259128224/jobs/105330126491) (passed).

The current implementation/evidence head `ea112b1` triggered a fresh set of
the same seven workflows, and all seven passed:

- [dependency/unsafe policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35260502595/jobs/105335000437) (passed);
- [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35260502595/jobs/105335000028) (passed);
- [tabdat-runtime on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35260502772/jobs/105334893584) (passed);
- [ReadStat feasibility on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35260502718/jobs/105334725169) (passed);
- [ReadStat Rust check on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35260502718/jobs/105334725502) (passed);
- [libgretl feasibility on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35260502465/jobs/105334724465) (passed);
- [libgretl OLS Rust check on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35260502633/jobs/105334725947) (passed).

The final PR head, ready transition, squash merge, deleted-branch verification,
and post-merge `main` checks are recorded below; all required jobs passed.

Final PR head `2857af6` (all seven jobs passed):

- [dependency/unsafe policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35262741549/jobs/105342236031) (passed);
- [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35262741549/jobs/105342236642) (passed);
- [tabdat-runtime on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35262741298/jobs/105342235136) (passed);
- [ReadStat feasibility on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35262741652/jobs/105342236266) (passed);
- [ReadStat Rust check on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35262741652/jobs/105342236655) (passed);
- [libgretl feasibility on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35262741520/jobs/105342236030) (passed);
- [libgretl OLS Rust check on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35262741521/jobs/105342236085) (passed).

Post-merge `main` commit `77f4754b4b0875b5e22e32c09d4b4854bb3427bb` (all seven
jobs passed):

- [dependency/unsafe policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35264950502/jobs/105349674847) (passed);
- [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35264950502/jobs/105349674597) (passed);
- [tabdat-runtime on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35264950528/jobs/105349674756) (passed);
- [ReadStat feasibility on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35264950617/jobs/105349675094) (passed);
- [ReadStat Rust check on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35264950617/jobs/105349674755) (passed);
- [libgretl feasibility on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35264950587/jobs/105349674990) (passed);
- [libgretl OLS Rust check on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35264950477/jobs/105349674804) (passed).

`feat/parser-run-syntax` was deleted locally and from `origin` after the
squash merge; `main` is synchronized at the merge SHA.

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
