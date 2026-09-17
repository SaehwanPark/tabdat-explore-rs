# Describe-command syntax contract

Status: bounded implementation slice, pending validation  
Producer: task owner  
Consumer: implementer/reviewer  
Selected skills: `tabdat-migration`, `simple-code-writer`  
Rust base: `main` at `79ae1d947e58f8017c2409172f48a7892dda98c3`  
Python oracle: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`)

## Scope

Extend the backend-independent `tabdat-language` parser with the single
zero-argument inspection command `describe`. It returns a typed syntax value
only. It must not inspect a dataset, access session state, initialize DuckDB,
execute a command, or add a runtime/backend dependency. Existing command forms
and diagnostics remain unchanged.

## Python contract

Authoritative paths at the pinned revision:

- `src/tabdat/models.py:136-140`: empty frozen `DescribeCommand` model;
- `src/tabdat/parser.py:525-529`: `describe` validation and dispatch;
- `tests/test_parser.py:187-188`: positive parser case;
- `tests/test_parser.py:1464-1465` and nearby invalid cases: arguments and
  `if` rejection;
- `docs/commands/describe.md:5-9`: public syntax (`describe` only).

Observed oracle behavior (Python 3.13.3, existing pinned environment):

| Input | Result |
| --- | --- |
| `describe` (any command case, surrounding whitespace) | `DescribeCommand()` |
| `describe age` / `describe if age > 18` | `describe does not accept arguments, if clauses, or options` |
| `describe,` / `describe age,` | `comma must be followed by at least one option` |
| `describe, detail` | `describe does not accept arguments, if clauses, or options` |
| `describe=now` | `describe assignment requires a target before =` |
| `describe == now` | `unsupported token in command: ==` |
| `describe -1` | `unsupported token in command: -` |

The full tokenizer's punctuation, expression, option, and malformed-token
diagnostics are outside this bounded syntax-only command dispatch. Preserve the
existing parser's explicit no-general-tokenizer boundary rather than inventing
broader grammar here.

## Rust contract

Extend `crates/tabdat-language/src/lib.rs` with `Command::Describe`. A bare
`describe` (case-insensitive, surrounding command whitespace ignored) maps to
that unit variant. Reuse the existing owned `ParseError` and deterministic
diagnostics. Keep `#![forbid(unsafe_code)]`, standard-library-only dependencies,
and pure parsing.

## Test contract

Add focused unit/integration tests for the valid form, case/whitespace
normalization, every exact invalid example above, and the unchanged existing
command matrix. Re-run the pinned parser oracle subset, then run all required
Rust workspace checks locally and in hosted CI.

## Implementation mapping

- Native Rust: one typed command variant and its no-argument validation.
- No data/session/result model, relation, execution, serialization, terminal,
  JSON, MCP, DuckDB, statistics, or native backend code.
- Deferred: active-dataset schema metadata and `DescribeResult`, which belong to
  the later Phase 4 data-runtime slice.

## Acceptance and stop conditions

Acceptance requires source/tests for exactly this syntax slice, current-state and
roadmap wording that does not claim a usable CLI or data runtime, and passing
`fmt`, locked workspace `check`/`test`, `clippy -D warnings`, `git diff --check`,
pinned policy scans, and hosted CI. Stop and report a conflict if preserving the
oracle requires execution/runtime code or a general tokenizer.

Implement exactly this contract. Do not broaden the parser into varlists,
options, expressions, or `describe` execution.
