# Phase 3 language migration evidence

## Slice 1: pinned parser corpus (in progress)

### Python contract

Oracle: `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`; clean sibling checkout verified
before capture, including the manifest's `uv.lock` SHA-256. Recover the actual
inputs exercised by `tests/test_parser.py`, including parametrized errors,
rather than assuming the existing bounded Rust tests cover the whole grammar.

### Rust contract

`tabdat_language::parse_command` must remain backend-independent and return an
owned typed command or deterministic parse diagnostic. This slice compares
acceptance, command variant and exact error messages; full field/expression
comparison is separate evidence and is not implied by variant agreement.

### Test contract

Capture actual oracle calls while running the original pytest suite, retain
successful dataclass fields and errors, and replay each unique command in a
Rust integration test. The capture tool is an external validation utility,
not a runtime dependency. Report mismatches and fix them in bounded follow-up
slices before closing Phase 3 gates.

### Implementation mapping

Safe Rust parser and checked-in synthetic JSON fixtures; Python/pytest only
for explicit oracle regeneration. No backend initialization or runtime command
execution is added. Property/fuzz and canonical script evidence follow this
corpus slice.
