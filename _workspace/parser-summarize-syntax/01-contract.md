# Contract: syntax-only `summarize`

Status: draft; contract recovery for the next bounded parser slice.

Selected skills: `tabdat-migration`, `simple-code-writer`

Rust base: `main` at `1a5eaf0` (the merged `duplicates` syntax slice and
roadmap-history correction with green hosted checks).

## Scope

Add only direct `summarize [varlist]` parsing to the pure `tabdat-language`
crate. The slice returns an owned typed command containing the ordered variable
names. It does not inspect an active relation, validate numeric columns, access
schema metadata, compute statistics, or initialize DuckDB/another backend.
Structured `if` clauses and options, duplicate-key or missingness semantics,
`by:` wrappers, execution, results, and rendering remain deferred.

## Python contract

The pinned clean oracle is the sibling checkout `../tabdat-explore` at commit
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`). Authoritative paths are:

- `src/tabdat/models.py:157-165`: `SummarizeCommand(variables: tuple[str, ...])`;
- `src/tabdat/parser.py:271-305,555-572`: direct routing and command builder;
- `src/tabdat/parser.py:3040-3158`: generic argument/condition/option/
  assignment diagnostics;
- `tests/test_parser.py:191-202`: direct summarize parser cases;
- `tests/test_parser.py:218-238`: structured `if`/option forms deliberately
  outside this slice;
- `docs/commands/summarize.md:1-25`: public syntax and examples;
- `src/tabdat/help/topics/summarize.md:1-17`: in-app syntax/help text.

The direct command is case-insensitive (`summarize`/`SUMMARIZE`) and accepts
surrounding Python-compatible separator whitespace, including TabDat control
whitespace. With no variables it returns an empty variable tuple. Otherwise
each whitespace-separated argument is retained in order. Quoted strings and
backtick identifiers are unwrapped; doubled backticks reduce to one backtick.
For example:

```text
summarize
summarize age bmi
summarize `bmi-zscore` `cost.2024` `x/y`
summarize "value col"
```

The exact direct-command diagnostics recovered from the pinned oracle include:

| Input shape | Diagnostic |
| --- | --- |
| `summarize age = 1` | `summarize does not accept assignment syntax` |
| `summarize = 1` | `summarize assignment requires a target before =` |
| `summarize age,` or `summarize,` | `comma must be followed by at least one option` |
| `summarize if` or `summarize age if` | `missing expression after if` |
| `summarize age==x` | `unsupported token in command: ==` |
| `summarize age-1`, `summarize age+1`, `summarize age!x`, or `summarize age@x` | corresponding `unsupported token in command` diagnostic |

The pinned parser intentionally accepts structured forms such as
`summarize age if age >= 18` and `summarize age bmi, detail limit=10` as a
different `ParsedCommand` family. This Rust slice must reject those forms with
the bounded direct-command diagnostics rather than pretending to provide their
expression/option AST.

The broader tokenizer and malformed-expression behavior are intentionally
deferred; this table is the frozen direct acceptance boundary for this slice.

The focused pinned oracle check is:

```sh
cd ../tabdat-explore
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py::test_parse_summarize_command_with_variables \
  tests/test_parser.py::test_parse_summarize_preserves_punctuated_variable_names \
  tests/test_parser.py::test_parse_summarize_command_without_variables
```

The broader pinned parser/script regression remains:

```sh
cd ../tabdat-explore
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py tests/test_script.py
```

Both commands must run against the recorded commit/tree without modifying the
oracle checkout.

## Rust contract

Expose one owned syntax-only variant:

```rust
pub enum Command {
    Summarize { variables: Vec<String> },
    // existing variants unchanged
}
```

The parser may reuse the bounded simple-body tokenizer used by the verified
`codebook`, `missing`, and `duplicates` forms. It must not introduce
`PathBuf`, filesystem access, relation or schema state, a backend dependency,
or unsafe code to the language/application boundary.

## Test contract

Unit and public integration tests will cover no-variable, ordered multi-variable,
mixed-case/control-whitespace, quoted and backtick-quoted names, assignment/
condition/option rejection, missing-condition and trailing-comma diagnostics,
unsupported punctuation, and the unchanged existing parser matrix. Tests
pattern-match the typed `variables` field. Generic expression/option/varlist
parity and prefixed forms are explicitly deferred.

## Implementation mapping and deferrals

- Native Rust: one typed command variant, direct dispatch, bounded argument
  parsing, exact direct-command diagnostics, and focused tests.
- Deferred: active relation/schema lookup, numeric-type validation, summary
  statistics, missingness rules, conditions and options, `by:` wrappers,
  full tokenizer/varlist/option and expression grammar, scripts,
  execution/results, serialization/reporting, and backend capability
  initialization.

Acceptance requires focused/full oracle results, Rust fmt/check/test/Clippy,
policy scans, `git diff --check`, independent parser/contract/workspace review,
and all hosted CI checks. Stop if satisfying the contract requires execution or
broad expression grammar; record the conflict instead of silently expanding the
slice.
