# Contract: syntax-only `missing`

Status: draft; contract recovery for the next bounded parser slice.

Selected skills: `tabdat-migration`, `simple-code-writer`

Rust base: `main` at `709a5dc` (the merged `codebook` syntax slice with green
hosted checks).

## Scope

Add only direct `missing [varlist]` parsing to the pure `tabdat-language` crate.
The slice returns an owned typed command containing the ordered variable names;
it does not inspect an active relation, count nulls, access schema metadata, or
initialize DuckDB/another backend. Existing commands and diagnostics stay
unchanged. Missingness counts/percentages, schema order, unknown-variable errors,
`by:` wrappers, conditions, options, execution, results, and rendering remain
deferred.

## Python contract

The pinned clean oracle is the sibling checkout `../tabdat-explore` at commit
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`). Authoritative paths are:

- `src/tabdat/models.py:179-183`: `MissingCommand(variables: tuple[str, ...])`;
- `src/tabdat/parser.py:271-305,574-579`: direct routing and command builder;
- `src/tabdat/parser.py:3040-3158`: generic argument/condition/option/
  assignment diagnostics;
- `tests/test_missing.py:16-28`: focused parser cases;
- `docs/commands/missing.md:1-30`: public syntax and deferred report semantics.

The direct command is case-insensitive (`missing`/`MISSING`) and accepts
surrounding Python-compatible separator whitespace, including TabDat control
whitespace. With no arguments it returns an empty variable tuple. Otherwise each
whitespace-separated argument is retained in order. Quoted strings and backtick
identifiers are unwrapped; doubled backticks reduce to one backtick. For example:

```text
missing
missing cost age
missing `bmi-zscore` `cost.2024` `x/y`
missing "value col"
```

No existence, type, wildcard, range, or missingness validation occurs in parsing.

The exact direct-command diagnostics recovered from the pinned oracle include:

| Input shape | Diagnostic |
| --- | --- |
| `missing age if age > 0` or `missing age, foo` | `missing does not accept if clauses or options` |
| `missing age = other` | `missing does not accept assignment syntax` |
| `missing = age` | `missing assignment requires a target before =` |
| `missing age,` or `missing,` | `comma must be followed by at least one option` |
| `missing if`, `missing if, foo`, or `missing age if` | `missing expression after if` |
| `missing age==x` | `unsupported token in command: ==` |
| `missing age-1`, `missing age+1`, `missing age!x`, or `missing age@x` | corresponding `unsupported token in command` diagnostic |

The broader tokenizer and malformed-expression behavior are intentionally
deferred; this table is the frozen direct acceptance boundary for this slice.

The focused pinned oracle check is:

```sh
cd ../tabdat-explore
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_missing.py -k 'parse_missing'
```

The broader pinned parser/script regression remains:

```sh
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py tests/test_script.py
```

Both commands must run against the recorded commit/tree without modifying the
oracle checkout.

## Rust contract

Expose one owned syntax-only variant:

```rust
pub enum Command {
    Missing { variables: Vec<String> },
    // existing variants unchanged
}
```

The parser may reuse the bounded simple-body tokenizer used by the verified
`codebook` form. It must not introduce `PathBuf`, filesystem access, relation or
schema state, a backend dependency, or unsafe code to the language/application
boundary.

## Test contract

Unit and public integration tests will cover no-variable, ordered multi-variable,
mixed-case/control-whitespace, quoted and backtick-quoted names,
assignment/condition/option rejection, missing-condition and trailing-comma
diagnostics, unsupported punctuation, and the unchanged existing parser matrix.
Tests pattern-match the typed `variables` field. Generic expression/varlist
parity and prefixed forms are explicitly deferred.

## Implementation mapping and deferrals

- Native Rust: one typed command variant, direct dispatch, bounded argument
  parsing, exact direct-command diagnostics, and focused tests.
- Deferred: active relation/schema lookup, null-count and percentage semantics,
  schema-order rules, unknown-variable errors, wildcard/range expansion,
  conditions and options, execution/results, serialization/reporting, `by:`
  wrappers, full tokenizer/varlist/option grammar, scripts, and backend
  capability initialization.

Acceptance requires focused/full oracle results, Rust fmt/check/test/Clippy,
policy scans, `git diff --check`, independent parser/contract/workspace review,
and all hosted CI checks. Stop if satisfying the contract requires execution or
broad expression grammar; record the conflict instead of silently expanding the
slice.
