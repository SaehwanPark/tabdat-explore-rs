# `gsort` syntax migration evidence

Status: accepted after PR #28 (`fd94133`) squash merge. Oracle, local, policy,
independent-review, hosted, merge, branch-cleanup, and post-merge evidence are
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

The frozen contract and source citations are in
`_workspace/parser-gsort-syntax/01-contract.md`.

## Revisions and implementation mapping

- `05566ca`: froze the bounded Python contract and reproducible probe command;
- `85b1a92`: added owned `SortKey`/`Command::Gsort`, direct dispatch including
  attached symbolic command suffixes, signed-key parsing, public/unit coverage,
  runtime command-name mapping, and explicit unsupported-runtime regression.
- `44ac3ca`: collapsed the gsort direction-prefix condition so the required
  clippy gate passes at the final implementation head.

Changed implementation paths:

- `crates/tabdat-language/src/lib.rs`: owned direction-aware key type, command
  variant, attached-boundary handling, parser, and unit tests;
- `crates/tabdat-language/tests/parser_contract.rs`: public typed-command and
  exact-diagnostic coverage;
- `crates/tabdat-runtime/src/lib.rs`: exhaustive command-name mapping;
- `crates/tabdat-runtime/tests/use_contract.rs`: explicit runtime deferral.

No manifest, dependency, backend, unsafe code, filesystem, relation, session,
CLI, serialization, or MCP surface changed.

## Pinned oracle probe

The reproducible probe from `01-contract.md` was re-run at the pinned revision:

```text
'gsort group_id -label' -> GsortCommand(keys=(SortKey(variable='group_id', descending=False), SortKey(variable='label', descending=True)))
'GSORT   +group_id   -label' -> GsortCommand(keys=(SortKey(variable='group_id', descending=False), SortKey(variable='label', descending=True)))
'gsort `-score`' -> GsortCommand(keys=(SortKey(variable='-score', descending=False),))
'gsort "old name"' -> GsortCommand(keys=(SortKey(variable='old name', descending=False),))
'gsort group_id group_id' -> GsortCommand(keys=(SortKey(variable='group_id', descending=False), SortKey(variable='group_id', descending=False)))
'gsort' -> gsort expects at least one variable
'gsort group_id, stable' -> gsort only accepts a signed variable list
'gsort group_id if x > 0' -> gsort only accepts a signed variable list
'gsort group_id = x' -> gsort only accepts a signed variable list
'gsort = x' -> gsort assignment requires a target before =
'gsort group_id =' -> gsort assignment requires an expression after =
'gsort group_id,' -> comma must be followed by at least one option
'gsort,' -> comma must be followed by at least one option
'gsort if' -> missing expression after if
'gsort --group_id' -> gsort keys must use at most one + or - prefix
'gsort -' -> gsort expects a variable after each direction prefix
'gsort +' -> gsort expects a variable after each direction prefix
'gsort group_id ++label' -> gsort keys must use at most one + or - prefix
'gsort group_id +-label' -> gsort keys must use at most one + or - prefix
'gsort group_id -+label' -> gsort keys must use at most one + or - prefix
'gsort group_id +' -> gsort expects a variable after each direction prefix
'gsort group_id -' -> gsort expects a variable after each direction prefix
'gsort age==x' -> GsortCommand(keys=(SortKey(variable='age==x', descending=False),))
'gsort age!x' -> unsupported token in command: !
'gsort age@x' -> unsupported token in command: @
'gsort:age' -> GsortCommand(keys=(SortKey(variable=':age', descending=False),))
'gsort=age' -> gsort assignment requires a target before =
'gsort+age' -> GsortCommand(keys=(SortKey(variable='age', descending=False),))
'gsort-age' -> GsortCommand(keys=(SortKey(variable='age', descending=True),))
'gsort age/label' -> GsortCommand(keys=(SortKey(variable='age/label', descending=False),))
'gsort age.label' -> GsortCommand(keys=(SortKey(variable='age.label', descending=False),))
'gsort ``' -> quoted identifier cannot be empty
'gsort "unterminated' -> unterminated quoted string
'gsort AGE\\x1cLABEL' -> GsortCommand(keys=(SortKey(variable='AGE', descending=False), SortKey(variable='LABEL', descending=False)))
'gsort  age\\x1dlabel' -> GsortCommand(keys=(SortKey(variable='age', descending=False), SortKey(variable='label', descending=False)))
```

Oracle test commands and results:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_gsort.py tests/test_parser.py \
  -k 'test_parse_gsort_direction_forms or test_parse_invalid_commands'
419 passed, 78 deselected in 0.59s

PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py tests/test_script.py
516 passed in 0.44s

PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_gsort.py
8 passed in 0.55s
```

## Rust checks at implementation head

Focused locked checks passed at `44ac3ca`:

```text
cargo fmt --all -- --check
cargo test --locked -p tabdat-language -p tabdat-runtime --all-targets
  tabdat-language: 40 unit + 29 integration tests passed
  tabdat-runtime: 2 unit + 11 integration tests passed
git diff --check
```

The full workspace and policy gates were then run at `44ac3ca`:

```text
cargo fmt --all -- --check                              passed
cargo check --locked --workspace --all-targets          passed
cargo test --locked --workspace --all-targets           passed
cargo clippy --locked --workspace --all-targets -- -D warnings
                                                         passed
git diff --check                                        passed
cargo deny check                                        passed
cargo audit -D warnings                                 passed
metadata-driven cargo geiger (all workspace packages,
  locked/all-targets/all-dependencies JSON assertions)  passed
```

The first clippy attempt at the preceding documentation head `c7be0a2`
reported `clippy::collapsible-if`; `44ac3ca` contains that mechanical fix and
the final clippy result above is from the exact implementation head. The
geiger loop followed `CONTRIBUTING.md`: it asserted one report per first-party
package, `forbid(unsafe_code)`, and zero first-party unsafe usage, while keeping
dependency-inventory warnings distinct from first-party safety.

The runtime tests include `leaves_gsort_execution_deferred`, which returns the
owned `UnsupportedCommand { name: "gsort" }` error without initializing or
mutating a backend/session relation.

The syntax-only slice intentionally does not claim full tokenizer parity. The
pinned Python parser rejects malformed numeric-looking keys such as `1.2.3`,
where this owned simple-body path preserves raw symbolic key text; that remains
deferred with the roadmap's tokenizer/varlist work. Attached command splitting
uses a character-aware boundary; exact Python Unicode classification remains
deferred as recorded below.

## Hosted acceptance and merge

PR #28 was marked ready after independent review and merged as `fd94133` on
2026-09-18. The exact pushed head was `5a837437d86cfa30e7dbc81dcea6f646eee75ec8`.
All required PR-head jobs passed:

- [Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35305066498/job/105475507432), 20m38s;
- [dependency and unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35305066498/job/105475507654), 19m08s;
- [tabdat-runtime](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35305066456/job/105475486979), 20m50s;
- [ReadStat Rust spike](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35305066442/job/105475461084), 28s;
- [ReadStat spike](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35305066442/job/105475461192), 20s;
- [libgretl OLS Rust spike](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35305066459/job/105475461176), 59s;
- [libgretl feasibility spike](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35305066468/job/105475461309), 58s.

The temporary `feat/parser-gsort-syntax` branch was deleted locally and on the
remote after the squash merge.

The documentation closeout commit `1ed5343d5cb9a49dd956ab0aa4211fbf10b2d401`
was pushed to `main`; its post-merge checks also passed:

- [main dependency and unsafe-code policy](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35306616500/job/105480061171), 19m19s;
- [main Rust baseline](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35306616500/job/105480061522), 20m36s;
- [main ReadStat spike](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35306616617/job/105480011349), 22s;
- [main ReadStat Rust spike](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35306616617/job/105480011442), 25s;
- [main libgretl OLS Rust spike](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35306616543/job/105480011059), 59s;
- [main libgretl feasibility spike](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/35306616511/job/105480011280), 1m06s.

Final cleanup verification: `main` is synchronized with `origin/main`, the
working tree is clean, and `git ls-remote --heads origin feat/parser-gsort-syntax`
returns no branch.

## Supported and deferred behavior

Supported here is direct syntax only: ordered owned keys, unprefixed/`+`
ascending and `-` descending directions, quoted/backtick provenance, duplicate
keys, attached symbolic key text, exact signed-list/assignment/condition/
option/quote diagnostics, control-whitespace separators, and explicit runtime
deferral.

Deferred are active-schema lookup, unknown-variable checks, stable/null/order
semantics, lazy/backend execution, relation/session effects, `by:` wrappers,
wildcard/range expansion, CLI/JSON/MCP output, malformed-number/tokenizer
parity, Unicode classification parity, attached-`if` condition parsing, quote
diagnostics after early assignment/option delimiters, and the roadmap's Phase
6.3 `gsort` transform behavior.

Independent parser review also recorded these explicit out-of-contract probes:

| Probe family | Pinned Python result | Current Rust result | Disposition |
| --- | --- | --- | --- |
| `gsort U+0345age` / attached `gsortU+0345age` | unsupported token in command | raw key / unknown command | deferred Unicode tokenizer parity |
| `gsort if+score`, `if/score`, `if:score`, `if==x` | expression-token diagnostics | literal symbolic key | deferred condition/expression parsing |
| `gsort if-score` | signed-list diagnostic | literal symbolic key | deferred condition/expression parsing |
| `'gsort=``'` and `'gsort,``'` | empty quoted-identifier diagnostic | assignment/comma diagnostic | deferred quote scanning after early delimiters |

The already-recorded `gsort 1.2.3` malformed-number difference has the same
tokenizer-parity disposition. None of these probes expands the accepted
contract or is presented as migrated behavior.
