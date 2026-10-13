# Phase 3 language migration evidence

## Slice 1: pinned parser corpus (complete; Phase 3 remains partial)

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

### Results and review

The capture ran the original **489 parser tests**, all passing, on Python 3.13.3
with pytest 9.0.3. It recorded **672 unique inputs**: 231 successful typed parses
and 441 errors. The replay agrees on acceptance/variant/exact diagnostic for 623
inputs and explicitly tracks **49 differences** in
`crates/tabdat-language/tests/fixtures/python-parser-gaps.json`. That ledger is
migration debt, not approved deviations or a parity assertion. All successful
Python dataclass fields are retained for subsequent deeper comparisons.

Validation: formatting, locked workspace check/test/all-target Clippy and
`git diff --check` passed. The first 120-second workspace test invocation timed
out during compilation; the complete rerun passed with a 600-second allowance.
No runtime dependency was added; serde_json is test-only and already locked.

Owner review used three separate passes (capture provenance/clean checkout;
replay and exact mismatch accounting; scope/dependency/documentation integrity).
No actionable issues remained. These were local owner reviews, not independent
model or hosted reviews. No additional model-backed workers were launched.

Next slices: reconcile conditional `summarize`/`keep`/`drop`, richer `tabulate`,
and missing `estat` syntax; compare expression/command fields; validate canonical
script behavior; add property/fuzz coverage; then perform the Phase 3 exit review.

## Slice 2: remaining no-option estat diagnostics

**Python contract:** pinned `src/tabdat/parser.py::_parse_estat` accepts residuals,
ovtest, margins, gof, did, drdid, dml and bayes without options; option-bearing
forms fail with `estat <name> does not support options`.

**Rust contract:** extend the finite `EstatSubcommand` enum and existing parser;
no untyped escape hatch or backend dependency. Execution remains unchanged:
only the already implemented VIF path runs, other diagnostics remain deferred.

**Test contract:** new typed-JSON/option-rejection characterization for all eight
forms, plus replay of the entire pinned corpus. The new test failed before the
implementation. Nine captured differences are resolved, leaving 40 explicit
gaps. This does not establish report/spatial syntax or diagnostic execution.

**Implementation mapping:** safe domain enums and language parsing only.
Local formatting, locked workspace check/test/Clippy and diff verification pass.
Three owner review passes covered enum/serialization, parser/error behavior, and
execution/dependency scope; no actionable findings. No delegated review claimed.
