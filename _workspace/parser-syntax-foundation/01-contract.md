# Parser syntax foundation contract

Status: bounded implementation slice, pending validation
Producer: task owner
Consumer: implementer/reviewer
Rust base: `main` at `c5f3ee8`
Python oracle: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`)

## Scope

Implement a syntax-only Rust language crate for the first command forms that do
not require a data or statistical backend:

- `help` with zero or one topic;
- `?` as the help alias;
- `status` with no arguments;
- `exit` and `quit` as equivalent session-termination commands.

The parser accepts leading/trailing whitespace and matches unquoted command names
case-insensitively. It returns owned typed commands and deterministic parse errors.
It must not initialize a backend or execute a command.

## Python contract

Authoritative paths at the pinned revision:

- `src/tabdat/parser.py`: `parse_command`, `_parse_help`, and structured command
  validation for `status`, `exit`, and `quit`;
- `src/tabdat/models.py`: `HelpCommand`, `StatusCommand`, `ExitCommand`;
- `tests/test_parser.py`: `test_parse_help_command`,
  `test_parse_status_command`, `test_parse_exit_aliases`, and invalid-command
  cases.

Observed oracle behavior (Python 3.13.3, existing pinned environment):

| Input | Result |
| --- | --- |
| `help` / `?` | `HelpCommand(topic=None)` |
| `help summarize` / `? summarize` | `HelpCommand(topic="summarize")` |
| `?foo` | `HelpCommand(topic="foo")` |
| `HELP SUMMARIZE` | `HelpCommand(topic="summarize")` |
| `status` | `StatusCommand()` |
| `exit` / `quit` | `ExitCommand()` |
| empty or whitespace-only input | `ParseError("empty command")` |
| unknown command `unknown` | `ParseError("unknown command: unknown")` |
| `help a b` or `? a b` | `ParseError("help expects at most one command name: help <command>")` |
| `status now` or `status, verbose` | `ParseError("status does not accept arguments, if clauses, options, or assignment syntax")` |
| `exit foo` or `quit, now` | command-specific `does not accept arguments, if clauses, or options` error |
| `status=now` | `ParseError("status assignment requires a target before =")` |
| `exit=now` or `quit=now` | command-specific `assignment requires a target before =` error |
| `help,verbose` | `ParseError("unknown command: help")` |

Quoted command names are rejected. Quoting a help topic is outside this slice's
normal syntax and is preserved as literal topic text only when the parser's simple
topic form receives it; no tokenizer or expression grammar is introduced here.

## Rust contract

Expose a backend-independent `parse_command(&str) -> Result<Command, ParseError>`
from `tabdat-language`.

- `Command` is an enum with `Help { topic: Option<String> }`, `Status`, and `Exit`.
- `ParseError` owns a stable human-readable message and implements `Display` and
  `Error`.
- `quit` maps to the same `Command::Exit` value as `exit`.
- Parsing is pure: no I/O, global state, clocks, randomness, or backend handles.

## Test contract

Add focused unit/integration tests for every positive and negative case above,
including whitespace and case normalization. Run the bounded Python parser tests
from the pinned sibling checkout as oracle evidence, then run all required Rust
workspace checks locally and in hosted CI.

## Implementation mapping

- Native Rust: lexical command boundary and typed syntax AST/error.
- DuckDB/native/statistics: not used.
- Deferred: all other command forms, tokenizer/expression AST, scripts, execution,
  reporting, JSON/MCP surfaces, and Python/Rust differential harness automation.

## Acceptance and stop conditions

Acceptance requires source/tests for this slice, `#![forbid(unsafe_code)]` on the
new crate, no root runtime dependency changes beyond the workspace member, and
passing `fmt`, `check`, `test`, `clippy`, and `git diff --check`. Stop if preserving
the observed behavior requires introducing backend/runtime code or if the pinned
oracle cannot be reverified.

Implement exactly this contract. Do not broaden scope; report conflicts instead of
inventing syntax or execution behavior.
