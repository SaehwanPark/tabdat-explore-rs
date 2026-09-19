# Bounded syntax-only `generate` contract

Status: accepted; implementation, hosted acceptance, merge, and branch cleanup
are complete.

Producer: task owner, using `tabdat-migration` and `simple-code-writer`, with
independent pinned-Python contract recovery and Rust boundary reconnaissance.
Consumer: the language-layer implementation and review for this loop.

## Scope

This loop adds the backend-independent typed syntax for:

```text
generate <new-variable> = <expression>
```

The parser must preserve command case normalization, exact target/reference
identifier spelling (including backtick-quoted identifiers and doubled
backticks), expression precedence, and owned expression structure. The
expression grammar includes identifiers, numeric/string/null literals, unary
minus, `+`, `-`, `*`, `/`, comparisons, parentheses, and function calls with
comma-separated arguments. Function names and calls are syntax only here.

No dataset, schema, session, DuckDB connection, filesystem, or output surface
may be touched by this language slice.

## Pinned Python authority

- repository: `/Volumes/research/gitrepos/tabdat-explore`;
- revision: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`;
- tree: `601b236788872323af9277d2276a236154a0f129`;
- Python: `3.13.3`;
- `uv.lock` SHA-256:
  `0f0e1dedbff49b4c77b470a75510b8809436c7d3dc77d1eac372ab9c7264d239`.

Authority paths:

- `src/tabdat/models.py:14-101,290-292` (expression and command models);
- `src/tabdat/parser.py:653-658,3327-3484` (command and expression parsing);
- `tests/test_parser.py:275-296,1334-1381` (generate, function, quote, and
  null examples);
- `tests/test_parser.py` invalid-command table (missing assignment/expression,
  duplicate `if`, options, and malformed-expression cases).

The focused pinned checks were:

```text
PYTHONDONTWRITEBYTECODE=1 uv run --no-sync pytest -q -p no:cacheprovider \
  tests/test_parser.py -k generate
3 passed, 486 deselected
```

The broader recovery check covering generate-related parser/runtime surfaces
also passed (`42 passed, 375 deselected`), but its execution behavior is
outside this syntax-only contract.

Recovered parser diagnostics include:

```text
generate expects syntax: generate new = expression
generate assignment requires an expression after =
incomplete expression after +
duplicate if clause
generate does not accept if clauses or options
unsupported token in expression: )
quoted identifier cannot be empty
unterminated quoted identifier
```

The pinned parser currently accepts a trailing comma after an expression; that
quirk is recorded for evidence and is not expanded into option parsing by this
slice.

## Rust acceptance contract

Add an owned typed `Command::Generate { variable, expression }` form and an
expression representation capable of retaining the grammar above, including
function-call nodes. Preserve exact target/reference names and nested argument
order. Keep the existing `AssertExpression` behavior and diagnostics stable;
any shared expression representation must not make `assert` claim function-call
runtime support.

The parser must reject missing/extra assignment structure, `if` clauses,
options, unsupported punctuation, malformed numbers, unbalanced parentheses,
empty/unterminated quoted identifiers, and incomplete operators with stable
errors. It must distinguish a quoted identifier named `if` from the clause
marker.

## Focused tests

- simple arithmetic, precedence, parentheses, string/null literals, and quoted
  identifiers;
- nested and multi-argument function calls, including exact function/argument
  spelling and order;
- command case normalization and target/reference names with spaces, commas,
  plus signs, and embedded backticks;
- exact parser diagnostics listed above plus unsupported symbols and malformed
  assignment forms;
- a regression proving parsing has no runtime/backend side effects.

## Explicit deferrals

This loop does not add `ExecutionResult`, runtime mutation, schema/type
validation, target-collision checks, unknown-variable checks, arithmetic
overflow/non-finite normalization, function evaluation, lazy/materialized
behavior, panel/label/`last_operation` state, CLI/REPL, JSON, MCP, or broader
expression/tokenizer parity. Those require a separate eager-runtime contract
with data-semantics evidence.

No new dependency, native backend, FFI, unsafe code, or ADR decision is needed.

Completion state: the bounded implementation, focused oracle/Rust checks,
independent review, and hosted acceptance are complete at implementation head
`772eb58`. PR #45 was marked ready, squash-merged as `63e65ec`, and its local
and remote temporary branch was deleted. Post-merge documentation and `main`
workflow verification are recorded in the companion evidence closeout.
