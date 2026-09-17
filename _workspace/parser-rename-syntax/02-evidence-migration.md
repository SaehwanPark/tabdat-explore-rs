# `rename` syntax migration evidence

Status: implementation complete; independent review and hosted acceptance are
pending.

Producer: task owner

Consumer: reviewers and the next maintainer

Boundary: pinned Python parser contract → Rust syntax-only parser

## Authority and contract inputs

The Python oracle is the clean sibling checkout `../tabdat-explore` at commit
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, package `0.25.0`, and Python
`3.13.3`. Its `uv.lock` SHA-256 is
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`; the
checkout is clean and no dependency synchronization or source edits were
performed.

The contract and implementation cite these pinned paths:

- `src/tabdat/models.py:262-265` (`RenameCommand(old_name, new_name)`);
- `src/tabdat/parser.py:127,252-306,646-651,3036-3123,3327-3395`
  (command inventory, generic dispatch, the specialized branch, and generic
  token/boundary diagnostics);
- `tests/test_parser.py:254-273,1490-1497` (positive and invalid forms);
- `docs/commands/rename.md:1-18` and `src/tabdat/help/topics/rename.md`
  (public syntax and execution description);
- `src/tabdat/cli.py:35,126,329` (catalog/effect metadata).

The Rust contract intentionally maps only the direct two-argument syntax to
owned strings. Generic quote removal, exact arity, condition/option/assignment
rejection, and the listed punctuation diagnostics are preserved; active-schema
lookup, collision checks, relation mutation, session effects, and all execution
surfaces remain deferred.

## Rust implementation revisions

- `bb20b02`: froze the bounded contract in `01-contract.md`;
- `7e7320e`: added `Command::Rename`, direct dispatch, the explicit colon
  boundary, the reusable simple-body parser path, runtime command-name mapping,
  and unit/public/runtime deferral tests.

Changed implementation paths:

- `crates/tabdat-language/src/lib.rs`: owned command variant, direct dispatch,
  rename parser, colon boundary, and unit tests;
- `crates/tabdat-language/tests/parser_contract.rs`: public typed-command and
  exact-diagnostic coverage;
- `crates/tabdat-runtime/src/lib.rs`: exhaustive command-name mapping;
- `crates/tabdat-runtime/tests/use_contract.rs`: explicit unsupported runtime
  regression.

No manifest, dependency, backend, unsafe code, filesystem, relation, session,
CLI, serialization, or MCP surface changed.

## Oracle checks

Focused transformation/invalid-command selection:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py -k 'test_parse_phase_3_transformation_commands or test_parse_invalid_commands'
419 passed, 70 deselected in 0.42s
```

Full pinned parser/script regression:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py tests/test_script.py
516 passed in 0.45s
```

The independent pinned probe used the same heredoc command recorded in
`01-contract.md` and produced the exact accepted values and diagnostics listed
there, including `rename if`/`rename old if`/`rename old new if` as `missing
expression after if`, the exact two-variable diagnostic for attached
conditions/options, `rename=old new` as an assignment-target diagnostic,
`rename:old new` as an unsupported colon, and `rename old-new new` as an
unsupported hyphen:

```text
'rename sex gender' -> RenameCommand(old_name='sex', new_name='gender')
'RENAME   sex   gender' -> RenameCommand(old_name='sex', new_name='gender')
'rename `old-name` `new-name`' -> RenameCommand(old_name='old-name', new_name='new-name')
'rename "old" "new"' -> RenameCommand(old_name='old', new_name='new')
'rename old old' -> RenameCommand(old_name='old', new_name='old')
'rename' -> rename expects exactly two variables: rename old new
'rename old' -> rename expects exactly two variables: rename old new
'rename old new now' -> rename expects exactly two variables: rename old new
'rename if' -> missing expression after if
'rename old if' -> missing expression after if
'rename old new if' -> missing expression after if
'rename old if x > 0' -> rename expects exactly two variables: rename old new
'rename old new if x > 0' -> rename expects exactly two variables: rename old new
'rename old new, replace' -> rename expects exactly two variables: rename old new
'rename old new,' -> comma must be followed by at least one option
'rename=old new' -> rename assignment requires a target before =
'rename = old' -> rename assignment requires a target before =
'rename==old new' -> unsupported token in command: ==
'rename:old new' -> unsupported token in command: :
'rename old-new new' -> unsupported token in command: -
```

## Rust checks

Focused checks passed:

```text
cargo test --locked -p tabdat-language -p tabdat-runtime --all-targets
tabdat-language: 34 unit + 23 integration tests passed
tabdat-runtime: 2 unit + 8 integration tests passed
```

The complete locked workspace baseline passed:

```text
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
  root scaffold: 1 passed
  tabdat-language: 34 unit + 23 integration passed
  tabdat-runtime: 2 unit + 8 integration passed
cargo clippy --locked --workspace --all-targets -- -D warnings
git diff --check
```

Separate policy checks passed:

```text
cargo deny check
  advisories ok, bans ok, licenses ok, sources ok
cargo audit -D warnings
  completed with no findings
metadata-driven cargo geiger
  first-party package reports satisfied forbid(unsafe_code) and zero
  first-party unsafe functions/expressions for all workspace packages
```

## Hosted acceptance

PR #25 (`https://github.com/SaehwanPark/tabdat-explore-rs/pull/25`) is open as
a draft from `feat/parser-rename-syntax`. The code-bearing evidence head is
`f376c84`; its complete hosted set passed:

- [dependency/unsafe policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35272514214/jobs/105375258694) (passed, 19m49s);
- [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35272514214/jobs/105375258289) (passed, 21m16s);
- [tabdat-runtime on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35272514305/jobs/105375164796) (passed, 20m35s);
- [ReadStat feasibility on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35272514171/jobs/105375066782) (passed, 47s);
- [ReadStat Rust check on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35272514171/jobs/105375067197) (passed, 24s);
- [libgretl feasibility on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35272514225/jobs/105375066463) (passed, 1m10s);
- [libgretl OLS Rust check on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35272514149/jobs/105375066433) (passed, 1m1s).

The later review/evidence closeout is documentation-only and does not change
the code-bearing revision or these seven results. Its current-head CI status
is tracked in the PR and review record; path-filtered native checks remain
anchored to `f376c84`.

The branch must not be marked ready until the independent parser, contract, and
workspace reviews approve the current head, all required hosted checks pass,
the PR is squash-merged, the temporary branch is deleted locally/remotely, and
the post-merge `main` checks pass.

## Supported and deferred behavior

Supported here is direct syntax only: two owned names, case/separator
normalization, generic quote/backtick unwrapping, duplicate names preserved as
syntax, exact arity/condition/option/assignment validation, and the recorded
boundary diagnostics. Runtime execution returns
`UnsupportedCommand { name: "rename" }`.

Deferred are active-schema lookup, collision and same-name validation, relation
transformation, panel metadata, by-groups, full varlist/tokenizer parity,
conditions/options/expressions, CLI/JSON/MCP output, and backend/session effects.
