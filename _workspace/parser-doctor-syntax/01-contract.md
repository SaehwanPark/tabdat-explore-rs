# Doctor-command syntax contract

Status: accepted and merged in PR #14
Producer: task owner
Consumer: implementer/reviewer
Selected skills: `tabdat-migration`, `simple-code-writer`
Rust base: `main` at `f6525b3fe76f85611bf907d2404d52aed66311d5`
Python oracle: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`)

## Scope

Extend the backend-independent `tabdat-language` parser with the single
zero-argument diagnostic command `doctor`. It returns a typed syntax value only.
It must not probe the host environment, inspect a dataset, access session state,
initialize DuckDB or another backend, execute a command, or add a runtime
dependency. Existing command forms and diagnostics remain unchanged, including
the unresolved `status -/+` parity deviation recorded in
`docs/migration/decisions.md`.

## Python contract

Authoritative paths at the pinned revision:

- `src/tabdat/models.py:150-154`: empty frozen `DoctorCommand` model;
- `src/tabdat/parser.py:543-553`: `doctor` validation and dispatch;
- `tests/test_doctor.py:51-68`: valid and invalid parser cases;
- `docs/commands/doctor.md:8-12`: public syntax (`doctor` only).

Observed oracle behavior (Python 3.13.3, existing pinned environment):

| Input | Result |
| --- | --- |
| `doctor` (any command case, surrounding whitespace) | `DoctorCommand()` |
| `doctor foo` / `doctor if x > 0` / `doctor, option` | `doctor does not accept arguments, if clauses, options, or assignment syntax` |
| `doctor if` | `missing expression after if` |
| `doctor,` / `doctor foo,` | `comma must be followed by at least one option` |
| `doctor = 1` / `doctor=1` | `doctor assignment requires a target before =` |
| `doctor == 1` | `unsupported token in command: ==` |
| `doctor -1` / `doctor +1` | `unsupported token in command: -` / `unsupported token in command: +` |

The full tokenizer's prefixed-command, punctuation, expression, option, and
malformed-token diagnostics are outside this bounded dispatch. `by group: doctor`
will remain deferred until prefixed-command parsing exists; the Python-specific
`doctor is not supported inside by commands` guard is not part of this slice.

## Rust contract

Extend `crates/tabdat-language/src/lib.rs` with `Command::Doctor`. A bare
`doctor` (case-insensitive, surrounding command whitespace ignored) maps to that
unit variant. Reuse the existing owned `ParseError` and deterministic
diagnostics. Keep `#![forbid(unsafe_code)}`, standard-library-only dependencies,
and pure parsing.

## Test contract

Add focused unit/integration tests for the valid form, case/whitespace
normalization, every exact invalid example above, and the unchanged existing
command matrix. Re-run the pinned doctor parser subset and the full parser/script
oracle, then run all required Rust workspace checks locally and in hosted CI.

## Implementation mapping

- Native Rust: one typed command variant and its no-argument validation.
- No environment/capability result model, data/session state, relation,
  execution, serialization, terminal, JSON, MCP, DuckDB, statistics, or native
  backend code.
- Deferred: environment inspection and `DoctorResult`, which require a later
  capability/reporting contract; `by:` prefix handling remains a separate parser
  slice.

## Acceptance and stop conditions

Acceptance requires source/tests for exactly this syntax slice, current-state and
roadmap wording that does not claim a usable CLI or environment diagnostics, and
passing `fmt`, locked workspace `check`/`test`, `clippy -D warnings`,
`git diff --check`, pinned policy scans, and hosted CI. Stop and record a conflict
if preserving the oracle requires host probing, execution/runtime code, or a
general tokenizer.

Implement exactly this contract. Do not broaden the parser into prefixed
commands, varlists, options, expressions, or `doctor` execution.

PR #14 disposition: implemented and merged as `0b918c7`; the feature branch was
deleted locally and remotely after all current-head checks passed.
