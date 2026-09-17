# Inspection-command syntax contract

Status: bounded implementation slice, pending validation  
Producer: task owner  
Consumer: implementer/reviewer  
Selected skills: `tabdat-migration`, `simple-code-writer`  
Rust base: `main` at `33ddc55ff025957e3e2fbc69b5ea47dcc79cc263`  
Python oracle: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`)

## Scope

Extend the backend-independent `tabdat-language` parser with the three bounded
inspection command forms that need no active dataset at parse time:

- `count` with no arguments;
- `head [n]`, defaulting to five rows;
- `tail [n]`, defaulting to five rows.

The parser returns typed syntax only. It must not execute a command, inspect
session state, initialize DuckDB, or add a runtime/backend dependency. Existing
`help`/`?`, `status`, `exit`, and `quit` behavior remains unchanged.

## Python contract

Authoritative paths at the pinned revision:

- `src/tabdat/models.py:215-241`: `CountCommand`, `HeadCommand`, and
  `TailCommand` model shapes;
- `src/tabdat/parser.py:606-621`: command dispatch;
- `src/tabdat/parser.py:3018-3033`: preview-limit validation and defaults;
- `tests/test_parser.py:205-215`: positive inspection-command cases;
- `tests/test_parser.py:1478-1489` and nearby invalid-command cases: exact
  diagnostics;
- `docs/commands/count.md`, `docs/commands/head.md`, and
  `docs/commands/tail.md`: public syntax and deferred execution semantics.

Observed oracle behavior (Python 3.13.3, existing pinned environment):

| Input | Result |
| --- | --- |
| `count` (any command case, surrounding whitespace) | `CountCommand()` |
| `head` / `tail` | `HeadCommand(5)` / `TailCommand(5)` |
| `head 10` / `tail 2` | corresponding limit |
| `head 01` / `tail 000` | corresponding canonical numeric value `1` / `0` |
| `head 0` / `tail 0` | corresponding zero limit |
| quoted numeric (`"10"`, `'10'`, `` `10` ``) | corresponding numeric limit |
| arbitrarily large ASCII decimal text | accepted without a parser range error |
| `count` with an argument, `if`, option, or assignment | `count does not accept arguments, if clauses, options, or assignment syntax` |
| `count = 1` | `count assignment requires a target before =` |
| `count == 1` | `unsupported token in command: ==` |
| `count -1` | `unsupported token in command: -` |
| `head 5 6` / `tail 5 6` | command-specific `accepts at most one row limit` |
| invalid limit (`1.0`, `1e2`, `foo`, Arabic digits, empty quoted text) | command-specific `row limit must be a non-negative integer` |
| `head -1` / `tail -1` | `unsupported token in command: -` |
| `head +1` / `tail +1` | `unsupported token in command: +` |
| `head 1 if x` / `tail 1 if x` | command-specific `does not accept if clauses, options, or assignment syntax` |
| `head 1, detail` / `tail 1, detail` | command-specific `does not accept if clauses, options, or assignment syntax` |

Limits are non-empty ASCII decimal digits only. Leading zeroes are normalized
to one `0` or the remaining significant digits. Do not parse through a bounded
Rust integer: the Python parser accepts values above `u64` and `u128`, so the
syntax type preserves canonical decimal text. A later execution contract must
make backend conversion/overflow behavior explicit rather than clamping.

## Rust contract

Extend `crates/tabdat-language/src/lib.rs` with:

```rust
pub enum Command {
    // existing variants
    Count,
    Head { limit: RowLimit },
    Tail { limit: RowLimit },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowLimit(Box<str>);
```

`RowLimit::as_decimal()` exposes the canonical ASCII decimal for later layers;
`Default` is five. Keep construction validated by the parser (a private helper
is sufficient) and preserve the crate's `#![forbid(unsafe_code)]` guarantee.

## Test contract

Add focused unit/integration tests for all positive and negative cases above,
including command case/whitespace, zero and leading-zero normalization, quoted
numeric limits, `u64::MAX + 1`, and a substantially larger decimal. Assert exact
diagnostic strings. Re-run the pinned Python parser suite as oracle evidence,
then run all required Rust workspace checks locally and in hosted CI.

## Implementation mapping

- Native Rust: parser command variants, validated `RowLimit`, and diagnostics.
- No DuckDB, relation, session, statistics, serde, terminal, JSON, or MCP code.
- Deferred: active-dataset preconditions; `CountResult`/`PreviewResult`; count
  state mutation; lazy relation execution; row-order/missingness guarantees;
  backend range conversion and reporting.

## Acceptance and stop conditions

Acceptance requires source/tests for this exact syntax slice, no unsupported
product-capability claims, and passing `fmt`, locked workspace `check`/`test`,
`clippy -D warnings`, `git diff --check`, pinned policy scans, and isolated
spike checks. Stop and report a contract conflict if preserving the oracle
requires execution/runtime code or silently bounding a decimal limit.

Implement exactly this contract. Do not broaden the parser into the remaining
Phase 2/3 command inventory or introduce a general tokenizer in this slice.
