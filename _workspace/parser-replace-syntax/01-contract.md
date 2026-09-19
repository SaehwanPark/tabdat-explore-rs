# Bounded syntax-only `replace` contract

Status: accepted and verified on `main` at `87ec017`.

Producer: task owner, using `tabdat-migration` and `simple-code-writer`, with
pinned Python parser reconnaissance.
Consumer: the pure `tabdat-language` parser and the runtime's explicit
unsupported-command boundary.

## Scope

Add the direct parsed form
`replace <target> = <expression> [if <condition>]` to the Rust language layer.
The command owns the target name, the full existing typed expression AST for
the replacement expression, and an optional expression AST for the condition.
Parsing remains pure and backend-independent: it does not inspect a schema,
read a relation, evaluate an expression, mutate session state, initialize
DuckDB, or produce CLI/JSON/MCP output.

The parser preserves the current Python expression surface in the existing
`GenerateExpression` tree: identifiers (including backtick-quoted names),
numeric and string literals, `null`, unary minus, arithmetic/comparison
operators, parentheses, and function calls. The runtime continues to reject
`replace` explicitly; relation mutation, type checks, predicate truthiness,
missing/non-finite arithmetic, overflow accounting, panel/label metadata,
lazy/materialized execution, output surfaces, and transform sequencing remain
deferred.

## Python contract

Pinned authority:

- repository: `/Volumes/research/gitrepo/tabdat-explore`;
- revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`;
- tree: `601b236788872323af9277d2276a236154a0f129`;
- Python: `3.13.3`;
- `uv.lock` SHA-256:
  `0f0e1dedbff49b4c77b470a75510b8809436c7d3dc77d1eac372ab9c7264d239`.

Authority paths:

- `src/tabdat/models.py:262-264,296-300` (`RenameCommand`,
  `GenerateCommand`, and `ReplaceCommand`);
- `src/tabdat/parser.py:127-131,646-668,3073-3147` (command inventory,
  direct command dispatch, assignment parsing, and the top-level `if` split);
- `tests/test_parser.py:275-292,1490-1503` (positive generate/replace AST
  retention and invalid-command coverage);
- `docs/commands/replace.md` and `src/tabdat/help/topics/replace.md` (public
  command syntax and execution intent).

The pinned parser reconnaissance reported `419 passed, 70 deselected` for
`tests/test_parser.py -k
'test_parse_phase_3_generate_and_replace_commands or test_parse_invalid_commands'`.
The positive oracle form is:

```text
replace cost = cost * 2 if sex == 'F'
```

which returns a `ReplaceCommand` with a binary arithmetic replacement and a
binary comparison condition. Direct `replace <target> = <expression>` without
an `if` returns `condition=None`; target and condition expressions retain
quoted names, strings, `null`, comparisons, and function-call nodes.

The pinned parser also establishes these command-boundary diagnostics:

| Input | Diagnostic |
| --- | --- |
| `replace` or `replace cost` | `replace expects syntax: replace existing = expression` |
| `replace = cost` | `replace assignment requires a target before =` |
| `replace cost =` | `replace assignment requires an expression after =` |
| `replace cost = cost, force` | `replace does not accept options` |
| `replace cost = cost if sex >` | `incomplete expression after >` |
| `replace cost = cost if sex == 'F' if x` | `duplicate if clause` |
| `replace cost == 1` | `unsupported token in command: ==` |
| `replace cost + 1` | `unsupported token in command: +` |
| `replace if = 1` | `unsupported token in expression: =` |

The exact parser boundary and punctuation behavior outside this bounded matrix
remain deferred rather than being inferred from the Python module layout.

## Rust contract

Extend the owned command enum with:

```rust
pub enum Command {
    Replace {
        variable: String,
        expression: GenerateExpression,
        condition: Option<GenerateExpression>,
    },
    // existing variants unchanged
}
```

The parser must:

1. recognize `replace` case-insensitively through the existing command boundary;
2. require one identifier target followed by `=` and a non-empty expression;
3. split an optional unquoted top-level `if` from the replacement expression,
   while leaving nested `if` identifiers and function/parenthesized content in
   the expression parser;
4. reject options and preserve the diagnostics above;
5. reuse `GenerateExpressionParser` so no second expression grammar or runtime
   dependency is introduced;
6. keep runtime behavior explicit as
   `RuntimeError::UnsupportedCommand { name: "replace" }` without backend
   initialization.

The existing `GenerateExpression` name is retained for compatibility with the
already accepted generate syntax/runtime boundary; its documentation may state
that it is shared by syntax-only generate and replace commands.

## Test contract

Focused Rust coverage must include:

- parsed replacement with arithmetic and optional comparison condition;
- no-condition replacement and case-insensitive command spelling;
- quoted target/source/condition identifiers, strings, `null`, and function
  calls retained in the AST;
- nested parentheses/function commas not mistaken for the top-level `if` or
  option boundary;
- exact missing-target, missing-expression, option, duplicate-`if`, and
  unsupported-command diagnostics;
- runtime deferral through `UnsupportedCommand { name: "replace" }` with no
  active dataset or backend side effect;
- unchanged generate, rename, select, sort, and existing runtime tests.

No new dependency, native backend, FFI boundary, unsafe code, or ADR decision is
required. The accepted syntax does not claim replace execution parity.

## Completion state

Contract recovery, the bounded implementation, focused Rust tests, independent
review, hosted acceptance, squash merge, and temporary-branch cleanup are
complete. PR #47 was merged as `87ec017`; the companion evidence, review, and
summary records document the checks and explicit runtime deferrals.
