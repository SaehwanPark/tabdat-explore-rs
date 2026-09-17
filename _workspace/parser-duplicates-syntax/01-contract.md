# Contract: syntax-only `duplicates`

Status: accepted; merged in PR #20 (`5460c7b64ba852a969bafbd1551e893d40a31ac4`).

Selected skills: `tabdat-migration`, `simple-code-writer`

Rust base: `main` at `d5ebf94` (the merged `missing` syntax slice with green
hosted checks). Implementation and evidence revisions were merged as the
single squash commit above after all hosted gates passed; the temporary branch
was deleted.

## Scope

Add only direct `duplicates [report] [varlist]` parsing to the pure
`tabdat-language` crate. The slice returns an owned typed command containing
the ordered variable names; an optional leading, unquoted `report` token is a
syntax alias and is removed. It does not inspect an active relation, validate
variables, group rows, access schema metadata, or initialize DuckDB/another
backend. Duplicate-group counts, null-key semantics, schema order, `by:`
wrappers, conditions, options, execution, results, and rendering remain
deferred.

## Python contract

The pinned clean oracle is the sibling checkout `../tabdat-explore` at commit
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`). Authoritative paths are:

- `src/tabdat/models.py:186-190`: `DuplicatesCommand(variables: tuple[str, ...])`;
- `src/tabdat/parser.py:271-305,581-589`: direct routing and command builder;
- `src/tabdat/parser.py:3040-3158`: generic argument/condition/option/
  assignment diagnostics;
- `tests/test_duplicates.py:67-78`: focused parser forms;
- `docs/commands/duplicates.md:1-42`: public syntax and deferred report
  semantics.

The direct command is case-insensitive (`duplicates`/`DUPLICATES`) and accepts
surrounding Python-compatible separator whitespace, including TabDat control
whitespace. With no variables it returns an empty variable tuple. Otherwise
each whitespace-separated argument is retained in order, except a first
unquoted `report` token, which is stripped. Quoted strings are unwrapped and
also participate in this alias stripping; backtick identifiers are unwrapped
but a quoted `` `report` `` remains a variable because Python marks only
backtick identifiers as quoted for this command. For example:

```text
duplicates
duplicates report
duplicates id
duplicates report id label
duplicates id label
duplicates "report"
duplicates `report`
```

No existence, type, wildcard, range, or duplicate-group validation occurs in
parsing.

The exact direct-command diagnostics recovered from the pinned oracle include:

| Input shape | Diagnostic |
| --- | --- |
| `duplicates id if id > 0` or `duplicates id, missing` | `duplicates does not accept if clauses or options` |
| `duplicates id = other` | `duplicates does not accept assignment syntax` |
| `duplicates = id` | `duplicates assignment requires a target before =` |
| `duplicates id,` or `duplicates,` | `comma must be followed by at least one option` |
| `duplicates if` or `duplicates id if` | `missing expression after if` |
| `duplicates id==x` | `unsupported token in command: ==` |
| `duplicates id-1`, `duplicates id+1`, `duplicates id!x`, or `duplicates id@x` | corresponding `unsupported token in command` diagnostic |

The broader tokenizer and malformed-expression behavior are intentionally
deferred; this table is the frozen direct acceptance boundary for this slice.

The focused pinned oracle check is:

```sh
cd ../tabdat-explore
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_duplicates.py -k 'parse_duplicates_forms'
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
    Duplicates { variables: Vec<String> },
    // existing variants unchanged
}
```

The parser may reuse the bounded simple-body tokenizer used by the verified
`codebook` and `missing` forms. It must not introduce `PathBuf`, filesystem
access, relation or schema state, a backend dependency, or unsafe code to the
language/application boundary.

## Test contract

Unit and public integration tests will cover no-variable, optional leading
`report`, ordered variables, mixed-case/control-whitespace, quoted and
backtick-quoted names (including quoted `report`), assignment/condition/option
rejection, missing-condition and trailing-comma diagnostics, unsupported
punctuation, and the unchanged existing parser matrix. Tests pattern-match the
typed `variables` field. Generic expression/varlist parity and prefixed forms
are explicitly deferred.

## Implementation mapping and deferrals

- Native Rust: one typed command variant, direct dispatch, bounded argument
  parsing, optional unquoted `report` stripping, exact direct-command
  diagnostics, and focused tests.
- Deferred: active relation/schema lookup, duplicate grouping and counts,
  null-key semantics, variable existence, wildcard/range expansion,
  conditions and options, execution/results, serialization/reporting, `by:`
  wrappers, full tokenizer/varlist/option grammar, scripts, and backend
  capability initialization.

Acceptance requires focused/full oracle results, Rust fmt/check/test/Clippy,
policy scans, `git diff --check`, independent parser/contract/workspace review,
and all hosted CI checks. Stop if satisfying the contract requires execution or
broad expression grammar; record the conflict instead of silently expanding
the slice.
