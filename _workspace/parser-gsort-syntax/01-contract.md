# Contract: syntax-only `gsort [+|-]varlist`

Status: accepted after PR #28 (`fd94133`) squash merge; runtime sorting remains
deferred.

Selected skills: `tabdat-migration`, `simple-code-writer`

Rust base: `c9da209` (`main` after the accepted syntax-only `sort` slice and
its final documentation-only CI run).

## Scope

Add only the direct `gsort [+|-]varlist` form to the pure `tabdat-language`
parser. The slice returns an owned typed command with ordered direction-aware
keys. It does not look up columns, sort rows, change an active relation,
mutate session state, initialize a backend, or add a runtime dependency.
Conditions, options, assignments, by-wrappers, CLI/JSON/MCP surfaces, and
relation execution remain deferred.

## Python contract

The authority is the clean sibling checkout `../tabdat-explore` at pinned commit
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, package `0.25.0`, and Python
`3.13.3`. Its `uv.lock` SHA-256 is
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`; the
checkout is clean and no dependency synchronization or source edits were
performed.

Authoritative paths:

- `src/tabdat/models.py:274-287`: `SortKey` and `GsortCommand`;
- `src/tabdat/parser.py:643-644,2983-3002`: direct dispatch and signed-key
  parsing;
- `src/tabdat/parser.py:3036-3123`: generic token and symbol handling;
- `tests/test_gsort.py:46-65`: focused parser coverage;
- `docs/commands/gsort.md:7-21,44-55`: public syntax and invalid forms;
- `src/tabdat/cli.py`: catalog/effect metadata.

Accepted forms preserve key order and variable spelling. Unprefixed and `+`
keys are ascending; `-` keys are descending. Direction prefixes are removed
only from unquoted keys, so a quoted variable named `-score` remains literal:

```text
gsort group_id -label       -> GsortCommand(keys=(SortKey("group_id"), SortKey("label", descending=True)))
gsort +group_id -label      -> same directions
gsort `-score`              -> SortKey("-score", descending=False)
gsort "old name"            -> SortKey("old name", descending=False)
gsort group_id group_id     -> duplicate keys preserved syntactically
```

The pinned tokenizer permits attached symbolic key text for this command, for
example `gsort+age`, `gsort-age`, `gsort:age`, `gsort age/label`, and
`gsort age==x`; symbols are part of the key unless they are a leading `+` or
`-` direction prefix. Unsupported symbols such as `@`, `!`, and `?` retain the
generic unsupported-token diagnostics.

Exact observed diagnostics:

| Input | Diagnostic |
| --- | --- |
| `gsort` | `gsort expects at least one variable` |
| `gsort group_id, stable`, `gsort group_id if x > 0`, or `gsort group_id = x` | `gsort only accepts a signed variable list` |
| `gsort = x` | `gsort assignment requires a target before =` |
| `gsort group_id =` | `gsort assignment requires an expression after =` |
| `gsort group_id,` or `gsort,` | `comma must be followed by at least one option` |
| `gsort if` | `missing expression after if` |
| `gsort --group_id`, `gsort ++label`, or `gsort +-label` | `gsort keys must use at most one + or - prefix` |
| `gsort -`, `gsort +`, or `gsort group_id -` | `gsort expects a variable after each direction prefix` |
| `gsort age!x` | `unsupported token in command: !` |
| `gsort age@x` | `unsupported token in command: @` |
| `gsort `` ` | `quoted identifier cannot be empty` |
| `gsort "unterminated` | `unterminated quoted string` |

## Reproducible oracle checks

The boundary probe used to freeze this contract is:

```sh
cd ../tabdat-explore
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync python - <<'PY'
from tabdat.parser import parse_command, ParseError

cases = (
    "gsort group_id -label", "GSORT   +group_id   -label", "gsort `-score`",
    'gsort "old name"', "gsort group_id group_id", "gsort", "gsort group_id, stable",
    "gsort group_id if x > 0", "gsort group_id = x", "gsort = x", "gsort group_id =",
    "gsort group_id,", "gsort,", "gsort if", "gsort --group_id", "gsort -", "gsort +",
    "gsort group_id ++label", "gsort group_id +-label", "gsort group_id -+label",
    "gsort group_id +", "gsort group_id -", "gsort age==x", "gsort age!x", "gsort age@x",
    "gsort:age", "gsort=age", "gsort+age", "gsort-age", "gsort age/label", "gsort age.label",
    "gsort ``", 'gsort "unterminated', "gsort AGE\x1cLABEL", "gsort  age\x1dlabel",
)
for text in cases:
    try:
        print(f"{text!r} -> {parse_command(text)!r}")
    except ParseError as exc:
        print(f"{text!r} -> {exc}")
PY
```

The focused and full pinned-oracle commands are:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_gsort.py tests/test_parser.py \
  -k 'test_parse_gsort_direction_forms or test_parse_invalid_commands'

PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py tests/test_script.py
```

The focused result and full parser/script result will be recorded in the
migration evidence ledger after the probe is re-run for implementation.

## Rust contract

Add owned types:

```rust
pub struct SortKey {
  pub variable: String,
  pub descending: bool,
}

Command::Gsort { keys: Vec<SortKey> }
```

Reuse `parse_simple_body(body, true)`, preserving quote/backtick provenance
when deciding whether to strip a leading `+` or `-`. Add a direct dispatch arm
and an exhaustive runtime `command_name` arm; execution remains an explicit
unsupported-command result.

## Test contract

Add focused unit and public integration coverage for ascending/descending and
explicit `+` keys, quoted/backtick punctuation, duplicate keys, arbitrary
varlists, attached symbols, missing keys, conditions/options/assignments,
trailing commas, repeated direction prefixes, unsupported punctuation, generic
quote errors, and runtime deferral. Preserve all existing command behavior.

## Deferrals

Deferred are active-schema lookup, unknown-variable checks, stable/null/order
semantics, lazy/backend execution, relation/session effects, `by:` wrappers,
wildcard/range expansion, CLI/JSON/MCP output, and full tokenizer/varlist
parity. The roadmap's `gsort` runtime transform item remains unchecked.

## Explicit tokenizer-parity deferrals

The bounded contract does not claim parity for tokenizer edge cases outside the
accepted forms above. Independent review recorded these pinned-oracle cases for
the future tokenizer/condition slice:

- Python's Unicode `str.isalnum()` boundary differs from Rust's standard
  character classification for U+0345 (`gsort U+0345age` is an unsupported
  token in Python, while the current Rust simple-body path preserves it as raw
  key text; attached `gsortU+0345age` is an unsupported token in Python and an
  unknown command in Rust).
- Attached reserved `if` forms are still handled by the simple-body path as
  literal symbolic keys (`if+score`, `if-score`, `if/score`, `if:score`, and
  `if==x`), while Python enters condition parsing and emits expression/list
  diagnostics.
- Early assignment/option delimiters stop Rust before later quote scanning
  (`'gsort=``'` and `'gsort,``'`), so Python's empty-quoted-identifier
  diagnostic is not yet reproduced there.

These cases remain explicit limitations, not acceptance claims; a later
tokenizer/condition milestone must add oracle probes and exact Rust tests before
their behavior is declared migrated.
