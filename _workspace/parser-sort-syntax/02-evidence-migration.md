# `sort` syntax migration evidence

Status: accepted. Implementation, hosted acceptance, squash merge, branch
cleanup, post-merge verification, and documentation-only closeout checks are
recorded below.

Producer: task owner

Consumers: reviewers and the next maintainer

Boundary: pinned Python parser contract → Rust syntax-only parser

## Authority and contract inputs

The Python oracle is the clean sibling checkout `../tabdat-explore` at pinned
commit `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, package `0.25.0`, and Python
`3.13.3`. Its `uv.lock` SHA-256 is
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`; the
checkout is clean and no dependency synchronization or source edits were
performed.

The contract and source citations are frozen in
`_workspace/parser-sort-syntax/01-contract.md`.

## Revisions and implementation mapping

- `0e415f0`: froze the bounded Python contract and probe command;
- `c19ac0c`: added the owned `Command::Sort` variant, direct dispatch,
  syntax-only parser, public/unit coverage, runtime command-name mapping, and
  explicit unsupported-runtime regression;
- `d574ec2`: added Rust coverage for leading punctuation boundaries identified
  by independent review.

Changed implementation paths:

- `crates/tabdat-language/src/lib.rs`: owned command variant, `sort:` boundary,
  direct parser, and unit tests;
- `crates/tabdat-language/tests/parser_contract.rs`: public typed-command and
  exact-diagnostic coverage;
- `crates/tabdat-runtime/src/lib.rs`: exhaustive command-name mapping;
- `crates/tabdat-runtime/tests/use_contract.rs`: explicit runtime deferral.

No manifest, dependency, backend, unsafe code, filesystem, relation, session,
CLI, serialization, or MCP surface changed.

## Pinned oracle probe

The reproducible command is recorded in `01-contract.md`. At the pinned
revision it produced:

```text
'sort age' -> SortCommand(variables=('age',))
'SORT   age   label' -> SortCommand(variables=('age', 'label'))
'sort `a,b` "old name"' -> SortCommand(variables=('a,b', 'old name'))
'sort age age' -> SortCommand(variables=('age', 'age'))
'sort age label now' -> SortCommand(variables=('age', 'label', 'now'))
'sort' -> sort expects at least one variable
'sort age if age > 0' -> sort only accepts a variable list
'sort age, stable' -> sort only accepts a variable list
'sort age = x' -> sort only accepts a variable list
'sort = x' -> sort assignment requires a target before =
'sort age =' -> sort assignment requires an expression after =
'sort age,' -> comma must be followed by at least one option
'sort if' -> missing expression after if
'sort age==x' -> unsupported token in command: ==
'sort age-1' -> unsupported token in command: -
'sort age+1' -> unsupported token in command: +
'sort age!x' -> unsupported token in command: !
'sort age@x' -> unsupported token in command: @
'sort:age' -> unsupported token in command: :
'sort=age' -> sort assignment requires a target before =
'sort==age' -> unsupported token in command: ==
'sort age:label' -> unsupported token in command: :
'sort age/label' -> unsupported token in command: /
'sort age.label' -> unsupported token in command: .
'sort +age' -> unsupported token in command: +
'sort -age' -> unsupported token in command: -
'sort !age' -> unsupported token in command: !
'sort @age' -> unsupported token in command: @
'sort,' -> comma must be followed by at least one option
'sort if x > 0' -> sort only accepts a variable list
'sort AGE\\x1cLABEL' -> SortCommand(variables=('AGE', 'LABEL'))
'sort  age\\x1dlabel' -> SortCommand(variables=('age', 'label'))
'sort ``' -> quoted identifier cannot be empty
'sort "unterminated' -> unterminated quoted string
```

The focused and full pinned oracle commands and outputs were:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_sort.py tests/test_parser.py \
  -k 'test_parse_sort_commands or test_parse_invalid_commands'
419 passed, 77 deselected in 0.41s

PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py tests/test_script.py
516 passed in 0.46s
```

## Rust checks at current implementation head

Focused checks at `d574ec2` passed:

```text
cargo fmt --all -- --check
cargo test --locked -p tabdat-language -p tabdat-runtime --all-targets
  tabdat-language: 38 unit + 27 integration tests passed
  tabdat-runtime: 2 unit + 10 integration tests passed
git diff --check
```

The complete locked workspace baseline at `d574ec2` passed:

```text
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
  root scaffold: 1 passed
  tabdat-language: 38 unit + 27 integration passed
  tabdat-runtime: 2 unit + 10 integration passed
cargo clippy --locked --workspace --all-targets -- -D warnings
git diff --check
```

Separate policy checks at `d574ec2` passed:

```text
cargo deny check
  advisories ok, bans ok, licenses ok, sources ok
cargo audit -D warnings
  completed with no findings (170 locked dependencies)
metadata-driven cargo geiger --all-dependencies --all-targets --locked
  tabdat-explore-rs: forbids_unsafe=true, unsafe functions=0, unsafe expressions=0
  tabdat-language: forbids_unsafe=true, unsafe functions=0, unsafe expressions=0
  tabdat-runtime: forbids_unsafe=true, unsafe functions=0, unsafe expressions=0
  geiger exit=0 for each first-party package
```

## Hosted acceptance

The draft PR is [PR #27](https://github.com/SaehwanPark/tabdat-explore-rs/pull/27),
from `feat/parser-sort-syntax`. The latest implementation/evidence head
`dc82d92` passed its complete hosted set:

- [dependency and unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35295649241/jobs/105447578806)
  (passed, 19m39s);
- [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35295649241/jobs/105447578593)
  (passed, 20m42s);
- [tabdat-runtime on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35295649246/jobs/105447575638)
  (passed, 21m4s);
- [ReadStat Rust check on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35295649328/jobs/105447526788)
  (passed, 26s);
- [ReadStat feasibility on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35295649328/jobs/105447527043)
  (passed, 23s);
- [libgretl OLS Rust check on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35295649249/jobs/105447526747)
  (passed, 1m7s);
- [libgretl feasibility on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35295649276/jobs/105447526997)
  (passed, 1m9s).

PR #27 was marked ready and squash-merged as `7cf21ae65cb159199661e9ba98ea52c9ef94e0fb`.
The temporary `feat/parser-sort-syntax` branch was deleted locally and from
the remote. Merge-triggered checks for the merge commit passed:

- [dependency and unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35298683135/jobs/105456476684)
  (passed, 16m33s);
- [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35298683135/jobs/105456476690)
  (passed, 20m41s);
- [tabdat-runtime on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35298683143/jobs/105456476476)
  (passed, 20m27s);
- [ReadStat Rust check on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35298683093/jobs/105456476739)
  (passed, 56s);
- [ReadStat feasibility on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35298683093/jobs/105456476544)
  (passed, 26s);
- [libgretl OLS Rust check on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35298683095/jobs/105456476416)
  (passed, 1m5s);
- [libgretl feasibility on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35298683150/jobs/105456476403)
  (passed, 1m44s).

The accepted-state documentation-only closeout commit was `e0649298793d2581a426b26a39ced2a50cf0fbaf`.
Its hosted checks passed:

- [dependency and unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35300324221/jobs/105461408485)
  (passed, 17m15s);
- [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35300324221/jobs/105461408797)
  (passed, 20m56s);
- [ReadStat feasibility](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35300324230/jobs/105461408511)
  (passed, 21s);
- [ReadStat Rust check](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35300324230/jobs/105461408681)
  (passed, 25s);
- [libgretl feasibility](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35300324215/jobs/105461408556)
  (passed, 1m20s);
- [libgretl OLS Rust check](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35300324223/jobs/105461408495)
  (passed, 2m10s).

The runtime-boundary workflow was correctly not triggered by this
documentation-only commit. No implementation or dependency files changed.

## Supported and deferred behavior

Supported here is direct syntax only: one or more owned names, case/separator
normalization, generic quote/backtick unwrapping, duplicate names preserved as
syntax, exact condition/option/assignment validation, missing-assignment
diagnostics, and the recorded punctuation/quote boundaries. Runtime execution
returns `UnsupportedCommand { name: "sort" }`.

Deferred are active-schema lookup, wildcard/range expansion, stable sorting,
null ordering, descending/expression keys, deduplication, labels/panel
metadata, by-groups, full-varlist/tokenizer parity, CLI/JSON/MCP output, and
backend/session effects.
