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

## Slice 3: estat report and spatial option syntax

**Python contract:** pinned `_parse_estat` accepts report saving/noopen; spatial
coordinates with positive integer KNN (default 5), or weights file plus required
ID and queen/rook contiguity (default queen). It rejects mutually exclusive
sources and inapplicable options before constructing the command.

**Rust contract:** variant-specific owned fields on `EstatSubcommand` avoid
independently nullable report/coordinate/file options. Existing no-option JSON
representations remain unchanged; new option variants are additive internal
syntax representations. Runtime still supports only VIF.

**Test contract:** failing-before-implementation acceptance tests, retained-field
and default tests, plus full pinned corpus replay. Thirteen more differences
resolved; **27 remain**. No corpus cases are dropped or oracle outputs rewritten.

**Implementation mapping:** safe domain enum and language option validation;
no backend, artifact I/O, statistical or spatial execution added. All root Cargo
checks and diff check passed. Three local owner review passes checked finite
state/JSON, options/diagnostics, and unchanged runtime dispatch; no actionable
issues remained. These are not independent model/hosted reviews.

## Slice 4: conditional summary and row-filter syntax

**Python contract:** pinned `_parse_command_parts`, `_parse_keep` and `_parse_drop`
retain conditions; generic `ParsedCommand` retains summary arguments, conditions,
and options. Row filters cannot combine a variable list with a predicate.

**Rust contract:** `SummarizeQuery` owns variables, optional expression and typed
options; `RowFilter` has a finite keep/drop action and mandatory expression.
Existing no-condition column operations and eager summaries retain their old
representations. Both new variants remain explicitly unsupported by execution,
so parsing cannot silently execute a projection or unconditional summary.

**Test contract:** new tests fail before implementation; typed condition/option
fields, quoted `if`, nested function commas, and Python's accepted trailing comma
on row filters are covered. Obsolete bounded syntax rejections are replaced by
acceptance tests, not silently retained as runtime constraints. Corpus equivalence
explicitly maps Python generic summary/condition-bearing keep/drop to the new
specific Rust variants. Thirteen differences resolved; **14 remain**.

**Implementation mapping:** safe language/domain with conservative metadata;
no new runtime data operation. Runtime tests assert unsupported diagnostics and
no backend initialization. All root Cargo/diff checks pass. Owner reviews found
an expression-argument comma/option-boundary bug during development, fixed with
a depth-aware boundary helper and nested-call regression tests; follow-up checks
pass. Three local owner passes cover AST, errors/boundaries and runtime isolation.
Whole AST equivalence and arbitrary generic option semantics remain later evidence.

## Slice 5: conditional/multidimensional tabulate syntax

**Python contract:** pinned `_parse_tabulate` accepts positional or explicit
rows/columns, predicates and paired values/stat options. Finite statistics are
count/mean/sum/min/max. Validation rejects dimensions duplicated across axes,
value/dimension overlap, incomplete aggregation pairs and percentages on values.
Repeated flags are idempotent; repeated value-bearing options are errors.

**Rust contract:** existing `Tabulate` retains bounded eager frequency execution;
`TabulateQuery` owns richer dimensions/predicate and an optional atomic
value/statistic pair. All new queries remain unsupported by execution.

**Test contract:** failing-before-implementation acceptance, retained dimension/
aggregation fields, repeated flags and complete corpus replay. Eleven differences
resolved; **3 remain**. Historical bounded diagnostics were reconciled with pinned
Python, not accepted as deliberate deviations.

**Implementation mapping:** safe domain metadata and a dedicated syntax validator;
no aggregation or conditional runtime added. Root formatting, locked workspace
check/test/Clippy and diff check passed. Three local owner reviews covered
validation ordering, type/effect isolation and compatibility. Case normalization
was caught by existing uppercase tests and fixed before verification.

## Slice 6: final captured diagnostics

**Python contract:** tokenizer/options reject empty parenthesized values before
command-specific validation; an empty `by:` prefix reports `unknown command: by`.
**Rust contract:** remove obsolete exceptions for empty by/i/j values and preserve
that boundary diagnostic. No commands or runtime effects are added.
**Test contract:** failing-before-implementation diagnostics and reconciled older
bounded tests; all **672 captured inputs now agree on acceptance, equivalent typed
variant and exact error text**. The gap ledger is empty, not a blanket AST or
script parity assertion. **Implementation mapping:** safe syntax validation only.
All root Cargo checks and diff check pass; three owner review passes covered
error precedence, empty option regressions and scope. No independent review claimed.
