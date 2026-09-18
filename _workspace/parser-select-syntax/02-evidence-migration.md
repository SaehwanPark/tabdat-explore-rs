# `select` syntax migration evidence

Status: accepted; implementation, independent review, hosted checks, merge,
branch cleanup, and post-merge verification are complete or recorded below.

Producer: task owner

Consumer: reviewers and the next maintainer

Boundary: pinned Python parser contract → Rust syntax-only parser

## Authority and contract inputs

The Python oracle is the clean sibling checkout `../tabdat-explore` at pinned
commit `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, package `0.25.0`, and Python
`3.13.3`. Its `uv.lock` SHA-256 is
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`; the
checkout is clean and no dependency synchronization or source edits were
performed.

The contract cites these pinned paths:

- `src/tabdat/models.py:256-259` (`SelectCommand(variables)`);
- `src/tabdat/parser.py:126,629-634,3036-3123,3327-3395` (command inventory,
  specialized direct branch, and generic token/boundary handling);
- `tests/test_parser.py:254-272,1334-1343,1490-1495` (positive, quoted-name,
  and invalid-command coverage);
- `docs/commands/select.md:1-24` and `src/tabdat/help/topics/select.md:1-14`
  (public syntax and intent);
- `src/tabdat/cli.py:133,308-313` (catalog/effect metadata).

The pinned probe observed direct parsing of ordinary, case-insensitive,
separator-whitespace, quoted/backtick, duplicate, and arbitrary-length
varlists. It also recorded the exact empty-varlist, condition/option,
assignment, missing-expression, trailing-comma, and punctuation diagnostics in
`01-contract.md`.

## Rust implementation revisions

- `0f0897d`: froze the bounded contract in `01-contract.md`;
- `6f4bd37`: added the owned `Command::Select` variant, direct dispatch,
  syntax-only parser, public/unit coverage, runtime command-name mapping, and
  explicit unsupported-runtime regression.
- `90f4c41`: matched Python's missing-assignment-expression diagnostic and
  added its unit/public regression coverage plus reproducible boundary-probe
  evidence.

Changed implementation paths:

- `crates/tabdat-language/src/lib.rs`: owned command variant, dispatch,
  direct parser, colon boundary, and unit tests;
- `crates/tabdat-language/tests/parser_contract.rs`: public typed-command and
  exact-diagnostic coverage;
- `crates/tabdat-runtime/src/lib.rs`: exhaustive command-name mapping;
- `crates/tabdat-runtime/tests/use_contract.rs`: explicit runtime deferral.

No manifest, dependency, backend, unsafe code, filesystem, relation, session,
CLI, serialization, or MCP surface changed.

## Oracle checks

Focused transformation/invalid-command selection:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py -k 'test_parse_phase_3_transformation_commands or test_parse_invalid_commands'
419 passed, 70 deselected in 0.43s
```

Full pinned parser/script regression:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py tests/test_script.py
516 passed in 0.46s
```

The exact direct probe recorded in the contract produced these representative
results:

```text
'select age sex' -> SelectCommand(variables=('age', 'sex'))
'SELECT   age   sex' -> SelectCommand(variables=('age', 'sex'))
'select `a,b`' -> SelectCommand(variables=('a,b',))
'select "old name" age' -> SelectCommand(variables=('old name', 'age'))
'select age age' -> SelectCommand(variables=('age', 'age'))
'select' -> select expects at least one variable
'select age if age > 0' -> select only accepts a variable list
'select age, stable' -> select only accepts a variable list
'select age = x' -> select only accepts a variable list
'select = x' -> select assignment requires a target before =
'select age =' -> select assignment requires an expression after =
'select age,' -> comma must be followed by at least one option
'select if' -> missing expression after if
'select age==x' -> unsupported token in command: ==
'select age-1' -> unsupported token in command: -
'select age+1' -> unsupported token in command: +
'select age!x' -> unsupported token in command: !
'select age@x' -> unsupported token in command: @
'select age sex now' -> SelectCommand(variables=('age', 'sex', 'now'))
```

The reproducible boundary-probe command was:

```sh
cd ../tabdat-explore
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync python - <<'PY'
from tabdat.parser import parse_command, ParseError

cases = ('select:age', 'select=age', 'select==age', 'select age:sex', 'select age/sex', 'select age.age', 'select +age', 'select -age', 'select !age', 'select @age', 'select age,', 'select,', 'select if x > 0', 'select AGE\x1cSEX', 'select  age\x1dsex')
for text in cases:
    try:
        print(f'{text!r} -> {parse_command(text)!r}')
    except ParseError as exc:
        print(f'{text!r} -> {exc}')
PY
```

At the pinned revision it prints:

```text
'select:age' -> unsupported token in command: :
'select=age' -> select assignment requires a target before =
'select==age' -> unsupported token in command: ==
'select age:sex' -> unsupported token in command: :
'select age/sex' -> unsupported token in command: /
'select age.age' -> unsupported token in command: .
'select +age' -> unsupported token in command: +
'select -age' -> unsupported token in command: -
'select !age' -> unsupported token in command: !
'select @age' -> unsupported token in command: @
'select age,' -> comma must be followed by at least one option
'select,' -> comma must be followed by at least one option
'select if x > 0' -> select only accepts a variable list
'select AGE\\x1cSEX' -> SelectCommand(variables=('AGE', 'SEX'))
'select  age\\x1dsex' -> SelectCommand(variables=('age', 'sex'))
```

## Rust checks

Focused checks passed:

```text
cargo fmt --all -- --check
cargo test --locked -p tabdat-language -p tabdat-runtime --all-targets
  tabdat-language: 36 unit + 25 integration tests passed
  tabdat-runtime: 2 unit + 9 integration tests passed
```

The complete locked workspace baseline passed locally:

```text
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
  root scaffold: 1 passed
  tabdat-language: 36 unit + 25 integration passed
  tabdat-runtime: 2 unit + 9 integration passed
cargo clippy --locked --workspace --all-targets -- -D warnings
git diff --check
```

Separate policy checks passed locally:

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

The draft PR is [PR #26](https://github.com/SaehwanPark/tabdat-explore-rs/pull/26),
from `feat/parser-select-syntax`. The pre-fix implementation baseline
`6f4bd37` had a complete hosted set that passed:

- [dependency and unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35282974595/jobs/105408968850)
  (passed, 19m30s);
- [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35282974595/jobs/105408969000)
  (passed, 20m28s);
- [tabdat-runtime on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35282974607/jobs/105408902345)
  (passed, 21m3s).

The contract-only superseded run `35282809276` was cancelled and is not an
acceptance result. Corrected implementation head `90f4c41` adds the missing
assignment-expression diagnostic and the boundary-probe evidence; its complete
current-head hosted set passed:

- [dependency and unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35285785033/jobs/105417739211)
  (passed, 19m35s);
- [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35285785033/jobs/105417739546)
  (passed, 20m15s);
- [tabdat-runtime on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35285785025/jobs/105417741372)
  (passed, 19m26s);
- [ReadStat feasibility on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35285784979/jobs/105417672663)
  (passed, 23s);
- [ReadStat Rust check on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35285784979/jobs/105417672541)
  (passed, 27s);
- [libgretl feasibility on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35285785044/jobs/105417673068)
  (passed, 1m7s);
- [libgretl OLS Rust check on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35285785077/jobs/105417672916)
  (passed, 1m21s).

Final evidence/documentation head `51e227f` also passed its complete
current-head hosted set:

- [dependency and unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35287430064/jobs/105422766354)
  (passed, 14m24s);
- [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35287430064/jobs/105422766141)
  (passed, 15m59s);
- [tabdat-runtime on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35287430076/jobs/105422766304)
  (passed, 20m47s);
- [ReadStat feasibility on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35287430049/jobs/105422766612)
  (passed, 25s);
- [ReadStat Rust check on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35287430049/jobs/105422766277)
  (passed, 25s);
- [libgretl feasibility on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35287430101/jobs/105422766469)
  (passed, 1m20s);
- [libgretl OLS Rust check on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35287430056/jobs/105422766245)
  (passed, 1m1s).

The temporary branch must be deleted locally and remotely after squash merge.
Merge-triggered `main` checks for merge commit `5735b43` passed:

- [dependency and unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35289043865/jobs/105427675690)
  (passed, 17m37s);
- [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35289043865/jobs/105427675919)
  (passed, 21m3s);
- [tabdat-runtime on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35289043839/jobs/105427675521)
  (passed, 21m31s);
- [ReadStat feasibility on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35289043859/jobs/105427675810)
  (passed, 20s);
- [ReadStat Rust check on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35289043859/jobs/105427675612)
  (passed, 21s);
- [libgretl feasibility on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35289043847/jobs/105427675236)
  (passed, 1m15s);
- [libgretl OLS Rust check on Linux](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35289043951/jobs/105427675822)
  (passed, 1m12s).

The final documentation-only closeout run for the accepted state will be
recorded after this commit.

## Supported and deferred behavior

Supported here is direct syntax only: one or more owned names, case/separator
normalization, generic quote/backtick unwrapping, duplicate names preserved as
syntax, exact condition/option/assignment validation, and the recorded boundary
diagnostics. Runtime execution returns
`UnsupportedCommand { name: "select" }`.

Deferred are active-schema lookup, wildcard/range expansion, selection mutation,
ordering/labels/panel metadata, `if` expressions, options, by-groups,
full-varlist/tokenizer parity, CLI/JSON/MCP output, and backend/session effects.
