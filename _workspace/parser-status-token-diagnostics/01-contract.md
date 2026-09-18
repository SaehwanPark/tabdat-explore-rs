# Contract: bounded `status` sign-token diagnostics

Status: bounded implementation slice, pending validation

Producer: task owner, with an independent oracle-contract pass

Consumer: implementer and reviewer

Selected skills: `tabdat-migration`, `simple-code-writer`

Rust base: `main` at `af76bb8`

Python oracle: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` (tree
`601b236788872323af9277d2276a236154a0f129`)

## Scope

Correct the unresolved punctuation/empty-condition diagnostics for the
syntax-only `status` command:

- `status -1` must report `unsupported token in command: -`;
- `status +1` must report `unsupported token in command: +`.

The same lexical rule applies to attached command forms and punctuation before
a trailing comma: `status-1`, `status+1`, `status -`, `status +`,
`status --1`, `status ++1`, `status -1,`, and `status +1,`. A bare `status if`
continues to report `missing expression after if`, including the existing
syntax-only condition boundary. The parser remains pure and
backend-independent. No status result, session inspection, environment probe,
or runtime behavior is added.

This is a deliberately bounded parity correction, not the full tokenizer
migration. Generic status arguments, options, assignments, `if` expressions,
malformed numbers, quote handling, and other punctuation remain governed by
their existing Rust behavior and are explicitly deferred to the future
tokenizer/parser slice.

## Python contract

Authoritative paths at the pinned revision:

- `src/tabdat/parser.py:531-541`: status command validation after tokenization;
- `src/tabdat/parser.py:3043-3123`: command tokenization and symbol handling;
- `tests/test_parser.py:124-131`: public status behavior;
- `docs/commands/status.md`: zero-argument status syntax.

Pinned probes (Python 3.13.3) produce:

```text
status       -> StatusCommand()
status -1    -> unsupported token in command: -
status +1    -> unsupported token in command: +
status-1     -> unsupported token in command: -
status+1     -> unsupported token in command: +
status -     -> unsupported token in command: -
status +     -> unsupported token in command: +
status --1   -> unsupported token in command: -
status ++1   -> unsupported token in command: +
status -1,   -> unsupported token in command: -
status +1,   -> unsupported token in command: +
status if    -> missing expression after if
```

The existing generic cases remain in scope as regression guards:

```text
status now       -> status does not accept arguments, if clauses, options, or assignment syntax
status, verbose  -> status does not accept arguments, if clauses, options, or assignment syntax
status = now     -> status assignment requires a target before =
status == now    -> unsupported token in command: ==
status if x      -> status does not accept arguments, if clauses, options, or assignment syntax
```

## Rust contract

Keep the existing `Command::Status` shape and pure parser boundary. Route the
`status` body through a small command-specific validation helper that preserves
the current precedence for empty input, trailing commas, assignment, and `==`,
then recognizes a leading `-` or `+` token and returns the exact punctuation
diagnostic. Do not introduce a general tokenizer or backend/runtime dependency.

## Test contract

Add public and unit regression coverage for the six signed forms above and keep
the existing generic status matrix green. Re-run the pinned focused parser
probe, the full parser/script oracle suites, all locked Rust checks, policy
checks, and the hosted workflows.

## Deferred behavior and stop conditions

This slice does not claim whole-parser parity. In particular, the known
MIG-0002 reproduction remains unresolved for other status token classes until a
future tokenizer slice supplies evidence and a separate decision. Do not modify
`docs/TABDAT_RUST_PORT_ROADMAP.md`'s broad tokenizer checkbox or add execution
claims here. Stop if exact sign diagnostics require expression, session, or
backend code.
