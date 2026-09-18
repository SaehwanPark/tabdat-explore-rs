# Contract: syntax-only `select <varlist>`

Status: accepted; implementation, independent review, hosted checks, merge,
branch cleanup, and post-merge verification are complete or recorded below.

Selected skills: `tabdat-migration`, `simple-code-writer`

Rust base: `e1881a5` (`main` after the accepted `rename` syntax slice and its
post-merge documentation-only CI run).

## Scope

Add only the direct `select <varlist>` command to the pure `tabdat-language`
parser. The slice returns an owned typed command without looking up columns,
changing an active relation, mutating session state, initializing a backend, or
adding a runtime dependency. Existing commands and diagnostics remain
unchanged. Conditions, options, assignments, by-wrappers, CLI/JSON/MCP
surfaces, and relation execution remain deferred.

## Python contract

The authority is the clean sibling checkout `../tabdat-explore` at pinned commit
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, package `0.25.0`, Python `3.13.3`,
and `uv.lock` SHA-256
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.

Authoritative paths:

- `src/tabdat/models.py:256-259`: `SelectCommand(variables)`;
- `src/tabdat/parser.py:126,629-634,3036-3123,3327-3395`: command inventory,
  specialized direct branch, and generic token/boundary handling;
- `tests/test_parser.py:254-272,1334-1343,1490-1495`: positive, quoted-name,
  and invalid-command coverage;
- `docs/commands/select.md:1-24` and `src/tabdat/help/topics/select.md:1-14`:
  public syntax and intent;
- `src/tabdat/cli.py:133,308-313`: catalog/effect metadata.

Accepted direct forms preserve argument order and spelling while normalizing the
case-insensitive command name and separator whitespace:

```text
select age sex       -> SelectCommand(variables=("age", "sex"))
SELECT   age   sex   -> SelectCommand(variables=("age", "sex"))
select `a,b`         -> SelectCommand(variables=("a,b",))
select "old name" age -> SelectCommand(variables=("old name", "age"))
select age age       -> SelectCommand(variables=("age", "age"))
```

Generic quote/backtick unwrapping is reused. Variable existence, wildcard or
range expansion, duplicate rejection, and active-relation effects are not
parser responsibilities in this slice.

Exact observed diagnostics:

| Input | Diagnostic |
| --- | --- |
| `select` | `select expects at least one variable` |
| `select age if age > 0`, `select age, stable`, or `select age = x` | `select only accepts a variable list` |
| `select = x` | `select assignment requires a target before =` |
| `select age =` | `select assignment requires an expression after =` |
| `select age,` | `comma must be followed by at least one option` |
| `select if` | `missing expression after if` |
| `select age==x` | `unsupported token in command: ==` |
| `select age-1` | `unsupported token in command: -` |
| `select age+1` | `unsupported token in command: +` |
| `select age!x` | `unsupported token in command: !` |
| `select age@x` | `unsupported token in command: @` |

The generic parser accepts additional variable tokens (for example,
`select age sex now`); this is a varlist, not an exact-arity command.

A reproducible pinned-oracle probe is:

```sh
cd ../tabdat-explore
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync python - <<'PY'
from tabdat.parser import parse_command, ParseError

cases = ("select age sex", "SELECT   age   sex", "select `a,b`", 'select "old name" age', "select age age", "select", "select age if age > 0", "select age, stable", "select age = x", "select = x", "select age =", "select age,", "select if", "select age==x", "select age-1", "select age+1", "select age!x", "select age@x", "select age sex now")
for text in cases:
    try:
        print(f"{text!r} -> {parse_command(text)!r}")
    except ParseError as exc:
        print(f"{text!r} -> {exc}")
PY
```

At the pinned revision it prints:

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

## Rust contract

Add the owned command variant:

```rust
Command::Select { variables: Vec<String> }
```

Reuse `parse_simple_body(body, false)`, preserving its quote handling and
boundary diagnostics. Require at least one argument, reject conditions,
options, and assignment syntax with the exact diagnostics above, and add an
exhaustive runtime `command_name` arm that leaves execution unsupported.

## Test contract

Add focused unit and public integration coverage for canonical and case/space
variants, quoted/backtick names (including punctuation and spaces), duplicate
names, missing varlists, condition/options/assignment rejection, trailing
commas, unsupported punctuation, and runtime deferral. Re-run the pinned focused
transformation/invalid selection and the full parser/script oracle suite.

## Implementation mapping and deferrals

The implementation is pure Rust in `crates/tabdat-language`; no DuckDB, native
backend, filesystem, relation, or session capability is needed. Runtime
execution returns `UnsupportedCommand { name: "select" }`.

Deferred are active-schema lookup, wildcard/range expansion, selection mutation,
ordering/labels/panel metadata, `if` expressions, options, by-groups,
full-varlist/tokenizer parity, CLI/JSON/MCP output, and backend/session effects.
