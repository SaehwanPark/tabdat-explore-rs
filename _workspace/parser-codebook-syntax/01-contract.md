# Contract: syntax-only `codebook`

Status: draft; contract recovery for the next bounded parser slice.

Selected skills: `tabdat-migration`, `simple-code-writer`

Rust base: `main` at `bd0d54a` (the merged `use` syntax slice with its
post-merge documentation update and green hosted checks).

## Scope

Add only direct `codebook [varlist]` parsing to the pure `tabdat-language`
crate. The slice returns an owned typed command containing the ordered variable
names without looking up an active dataset, inspecting schema, counting rows,
or initializing DuckDB/another backend. Existing commands and diagnostics stay
unchanged. General varlist expansion, ranges, wildcards, `by:` wrappers,
conditions, options, execution, results, and rendering remain deferred.

## Python contract

The pinned clean oracle is the sibling checkout `../tabdat-explore` at commit
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`). Authoritative paths are:

- `src/tabdat/models.py:168-176`: `CodebookCommand(variables: tuple[str, ...])`;
- `src/tabdat/parser.py:271-305,567-572`: direct routing and command builder;
- `src/tabdat/parser.py:3040-3158`: generic tokenization, arguments,
  conditions, options, and assignment diagnostics;
- `tests/test_parser.py:205-207,1476-1477`: accepted and rejected parser
  forms;
- `docs/commands/codebook.md:1-22` and
  `src/tabdat/help/topics/codebook.md:1-14`: public syntax and examples.

The direct command is case-insensitive (`codebook`/`CODEBOOK`) and accepts
surrounding Python-compatible separator whitespace. With no arguments it
returns an empty variable tuple. Otherwise each whitespace-separated argument
is retained in order. Quoted strings are unwrapped; backtick-quoted identifiers
are unwrapped with doubled backticks reduced to one backtick. For example:

```text
codebook
codebook age sex
codebook `bmi-zscore` `cost.2024` `x/y`
codebook `x y`
```

These produce `CodebookCommand(variables=())`, `("age", "sex")`, and the
corresponding owned names. No existence, type, wildcard, or delimiter
validation occurs in parsing.

The exact direct-command diagnostics recovered from the pinned oracle include:

| Input shape | Diagnostic |
| --- | --- |
| `codebook age if age > 18` or `codebook age, detail` | `codebook does not accept if clauses or options` |
| `codebook age = 1` | `codebook does not accept assignment syntax` |
| `codebook = 1` | `codebook assignment requires a target before =` |
| `codebook age,` | `comma must be followed by at least one option` |
| `codebook if` | `missing expression after if` |
| unsupported punctuation such as `codebook -1` | `unsupported token in command: -` |

When generic tokenization reaches a missing assignment expression or malformed
expression, Python may emit a more specific expression diagnostic (for example,
`codebook age =` → `codebook assignment requires an expression after =`). That
broader expression grammar is intentionally deferred; the direct forms above
are the frozen acceptance boundary for this slice.

The focused pinned oracle check is:

```sh
cd ../tabdat-explore
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py::test_parse_phase_3_inspection_commands
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
    Codebook { variables: Vec<String> },
    // existing variants unchanged
}
```

The parser may reuse the existing bounded simple-body tokenizer because its
quoted argument handling already matches the recovered direct forms. It must
not introduce `PathBuf`, filesystem access, schema/session state, a backend
dependency, or unsafe code to the language/application boundary.

## Test contract

Unit and public integration tests will cover no-variable, ordered
multi-variable, mixed-case/whitespace, quoted and backtick-quoted names,
assignment/condition/option rejection, missing-condition and trailing-comma
diagnostics, unsupported punctuation, and the unchanged existing parser
matrix. Tests pattern-match the typed `variables` field rather than relying on
debug formatting. Generic expression/varlist parity and prefixed forms are
explicitly deferred.

## Implementation mapping and deferrals

- Native Rust: one typed command variant, direct dispatch, reuse of the
  backend-independent simple argument parser, exact bounded diagnostics, and
  focused tests.
- Deferred: active dataset/schema lookup, variable existence and type checks,
  wildcard/range expansion, conditions and options, execution/results,
  serialization/reporting, `by:` wrappers, full tokenizer/varlist/option
  grammar, scripts, and backend capability initialization.

Acceptance requires focused/full oracle results, Rust fmt/check/test/Clippy,
policy scans, `git diff --check`, independent parser/contract/workspace review,
and all hosted CI checks. Stop if satisfying the contract requires execution or
broad expression grammar; record the conflict instead of silently expanding the
slice.
