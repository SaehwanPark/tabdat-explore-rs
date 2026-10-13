# Phase 3 script differential evidence

Status: **in progress**. This document records the next language/script slice;
it does not establish script parity or close Phase 3.

## Python contract

Oracle: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, lock digest in
[the baseline manifest](python-baseline.toml). Authoritative inputs are
`src/tabdat/script.py`, `tests/test_script.py`, script cases in
`tests/test_cli.py` and `docs/user-guide/scripting-and-json.md`.
The checkout remains clean; its original 27 script tests passed during recovery
with Python 3.13.3 and pytest 9.0.3. This is existing-environment evidence,
not a fresh lockfile environment claim.

Recover exact command text/start lines, comments/blank lines, multiline SQL,
macros, seed/let directives, conditions, active branches, nested script context,
recursion rejection and path/line/message diagnostics. Preserve Python's
`splitlines`/whitespace rules and seed integer behavior unless a reviewed
migration decision explicitly changes them. Tests passing for LF/ASCII alone
cannot support an unrestricted script contract.

## Rust contract

Keep owned typed script commands/directives/errors and explicit macro/seed/block
state. Syntax APIs initialize no backend. Existing session script coordination
owns filesystem inclusion and command execution; compare observable stream,
state and errors without importing Python's module architecture.

## Test contract and acceptance

1. Capture the original script suite's actual public API calls from a verified
   oracle, retaining exact outputs/errors/context inputs and provenance.
2. Replay all captured outputs and errors in Rust; compare text and start lines,
   directive fields, macro output and condition values. Any normalization must
   name the representational difference (for example synthetic source paths).
3. Add canonical shared `.td` fixtures for comments, multiline SQL, macro/seed
   state and conditional branches; exercise nested inclusion, recursion and
   file/line failures through existing session coordination. Record runtime
   deferrals separately from language equivalence.
4. Probe source-established edge cases, including Unicode line boundaries and
   whitespace, macro literal/reference boundaries, seed integer syntax/range,
   invalid directives and misplaced/unclosed control flow. Fix differences or
   register a reviewed intentional deviation; retain failed evidence.
5. Run focused differential/regression checks, required root Cargo checks,
   documentation/link/diff verification and three local owner review passes.
   Update only roadmap gates covered by the resulting evidence.

## Implementation mapping and remaining work

Safe Rust language and existing execution/I/O coordination; checked-in synthetic
fixtures and external Python capture only. No new statistical implementation,
backend integration, CLI transport or production Python dependency is implied.
Full parser AST/tokenizer comparison and the final Phase 3 audit remain separate
required loops after this slice. Current evidence proves only oracle recovery;
capture, differential replay, corrections and final verification remain open.
