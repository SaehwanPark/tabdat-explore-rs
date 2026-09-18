# `gsort` syntax migration evidence

Status: implementation complete at `85b1a92`; local and oracle evidence are
recorded below. Independent review, hosted checks, readiness, merge, branch
cleanup, and post-merge verification are pending.

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

Focused locked checks passed at `85b1a92`:

```text
cargo fmt --all -- --check
cargo test --locked -p tabdat-language -p tabdat-runtime --all-targets
  tabdat-language: 40 unit + 29 integration tests passed
  tabdat-runtime: 2 unit + 11 integration tests passed
git diff --check
```

The runtime tests include `leaves_gsort_execution_deferred`, which returns the
owned `UnsupportedCommand { name: "gsort" }` error without initializing or
mutating a backend/session relation.

The syntax-only slice intentionally does not claim full tokenizer parity. The
pinned Python parser rejects malformed numeric-looking keys such as `1.2.3`,
where this owned simple-body path preserves raw symbolic key text; that remains
deferred with the roadmap's tokenizer/varlist work. Attached command splitting
uses a character-aware boundary so non-ASCII identifier continuations do not
become accidental `gsort` keys.

## Hosted acceptance

Draft PR #28 is [open](https://github.com/SaehwanPark/tabdat-explore-rs/pull/28)
from `feat/parser-gsort-syntax`. Its implementation/evidence head and complete
hosted check set will be linked here after independent review and readiness.

## Supported and deferred behavior

Supported here is direct syntax only: ordered owned keys, unprefixed/`+`
ascending and `-` descending directions, quoted/backtick provenance, duplicate
keys, attached symbolic key text, exact signed-list/assignment/condition/
option/quote diagnostics, control-whitespace separators, and explicit runtime
deferral.

Deferred are active-schema lookup, unknown-variable checks, stable/null/order
semantics, lazy/backend execution, relation/session effects, `by:` wrappers,
wildcard/range expansion, CLI/JSON/MCP output, malformed-number/tokenizer
parity, and the roadmap's Phase 6.3 `gsort` transform behavior.
