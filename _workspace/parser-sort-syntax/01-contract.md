# Contract: syntax-only `sort <varlist>`

Status: frozen; implementation and acceptance are pending.

Selected skills: `tabdat-migration`, `simple-code-writer`

Rust base: `c49108c` (`main` after the accepted `select` syntax slice and its
final documentation-only CI run).

## Scope

Add only the direct `sort <varlist>` command to the pure `tabdat-language`
parser. The slice returns an owned typed command without looking up columns,
sorting rows, changing an active relation, mutating session state,
initializing a backend, or adding a runtime dependency. Existing commands and
diagnostics remain unchanged. Conditions, options, assignments, by-wrappers,
CLI/JSON/MCP surfaces, and relation execution remain deferred.

## Python contract

The authority is the clean sibling checkout `../tabdat-explore` at pinned commit
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, package `0.25.0`, and Python
`3.13.3`. Its `uv.lock` SHA-256 is
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`; the
checkout is clean and no dependency synchronization or source edits were
performed.

Authoritative paths:

- `src/tabdat/models.py:267-271`: `SortCommand(variables)`;
- `src/tabdat/parser.py:126,636-641,3036-3123,3327-3395`: command inventory,
  specialized direct branch, and generic token/boundary handling;
- `tests/test_sort.py:44-52`: focused parser coverage;
- `docs/commands/sort.md:1-24` and `src/tabdat/help/topics/sort.md:1-19`:
  public syntax and execution intent;
- `src/tabdat/cli.py:135,315-320`: catalog/effect and schema metadata.

Accepted direct forms preserve argument order and spelling while normalizing the
case-insensitive command name and separator/control whitespace:

```text
sort age                         -> SortCommand(variables=("age",))
SORT   age   label               -> SortCommand(variables=("age", "label"))
sort `a,b` "old name"            -> SortCommand(variables=("a,b", "old name"))
sort age age                     -> SortCommand(variables=("age", "age"))
sort age label now               -> SortCommand(variables=("age", "label", "now"))
```

Generic quote/backtick unwrapping is reused. Variable existence, wildcard or
range expansion, duplicate rejection, sort order/null semantics, and
active-relation effects are not parser responsibilities in this slice.

Exact observed diagnostics:

| Input | Diagnostic |
| --- | --- |
| `sort` | `sort expects at least one variable` |
| `sort age if age > 0`, `sort age, stable`, or `sort age = x` | `sort only accepts a variable list` |
| `sort = x` | `sort assignment requires a target before =` |
| `sort age =` | `sort assignment requires an expression after =` |
| `sort age,` or `sort,` | `comma must be followed by at least one option` |
| `sort if` | `missing expression after if` |
| `sort age==x` | `unsupported token in command: ==` |
| `sort age-1` | `unsupported token in command: -` |
| `sort age+1` | `unsupported token in command: +` |
| `sort age!x` | `unsupported token in command: !` |
| `sort age@x` | `unsupported token in command: @` |

Generic tokenizer boundaries observed at the pinned revision include
`unsupported token in command: :`, `/`, and `.` for attached punctuation,
unsupported `+`, `-`, `!`, and `@` prefixes, and the exact generic quote errors
`quoted identifier cannot be empty` and `unterminated quoted string`.

A reproducible pinned-oracle probe is:

```sh
cd ../tabdat-explore
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync python - <<'PY'
from tabdat.parser import parse_command, ParseError

cases = (
    "sort age", "SORT   age   label", "sort `a,b` \"old name\"",
    "sort age age", "sort age label now", "sort", "sort age if age > 0",
    "sort age, stable", "sort age = x", "sort = x", "sort age =",
    "sort age,", "sort if", "sort age==x", "sort age-1", "sort age+1",
    "sort age!x", "sort age@x", "sort:age", "sort=age", "sort==age",
    "sort age:label", "sort age/label", "sort age.label", "sort +age",
    "sort -age", "sort !age", "sort @age", "sort,", "sort if x > 0",
    "sort AGE\x1cLABEL", "sort  age\x1dlabel", "sort ``",
    'sort "unterminated',
)
for text in cases:
    try:
        print(f"{text!r} -> {parse_command(text)!r}")
    except ParseError as exc:
        print(f"{text!r} -> {exc}")
PY
```

The pinned focused parser selection and full parser/script regression were
re-run before freezing this contract:

```text
tests/test_sort.py tests/test_parser.py -k 'test_parse_sort_commands or test_parse_invalid_commands'
419 passed, 77 deselected in 0.41s

tests/test_parser.py tests/test_script.py
516 passed in 0.46s
```

## Rust contract

Add the owned command variant:

```rust
Command::Sort { variables: Vec<String> }
```

Reuse `parse_simple_body(body, false)`, preserving its quote handling and
boundary diagnostics. Require at least one argument, reject conditions,
options, and assignment syntax with the exact diagnostics above, and add an
exhaustive runtime `command_name` arm that leaves execution unsupported.

## Test contract

Add focused unit and public integration coverage for canonical and case/space
variants, quoted/backtick names (including punctuation and spaces), duplicate
names, arbitrary-length varlists, missing varlists, condition/options/
assignment rejection, trailing commas, unsupported punctuation, generic quote
errors, missing assignment expressions, and runtime deferral. Re-run the
pinned focused sort/invalid selection and the full parser/script oracle suite.

## Implementation mapping and deferrals

The implementation is pure Rust in `crates/tabdat-language`; no DuckDB, native
backend, filesystem, relation, or session capability is needed. Runtime
execution returns `UnsupportedCommand { name: "sort" }`.

Deferred are active-schema lookup, wildcard/range expansion, stable sorting,
null ordering, descending/expression keys, deduplication, labels/panel
metadata, by-groups, full-varlist/tokenizer parity, CLI/JSON/MCP output, and
backend/session effects.
