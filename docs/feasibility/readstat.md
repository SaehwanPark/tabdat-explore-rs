# ReadStat feasibility reconnaissance

- Status: **binding and DTA ingestion spike complete**; the pinned native
  candidate builds and runs its upstream tests on the two target platforms, and a
  Rust `readstat-sys`-style low-level binding plus a `forbid(unsafe_code)` safe
  facade now load a representative `.dta` without pandas. The spike is not
  product support, DTA support, or migration parity.
- Candidate: ReadStat `v1.0.0`, release commit
  `988d78ca51e3330e1d51bbfb0796784781014b70`.
- Reproducible source artifact:
  `readstat-1.0.0.tar.gz`, SHA-256
  `7635c73dd04532dbfa785c0485dbd6d52d86bc23030a1227e4a5a77a83824b84`.
- Upstream: [WizardMac/ReadStat](https://github.com/WizardMac/ReadStat).
- Decision record: [ADR 0004](../adr/0004-readstat-feasibility.md).

## Scope and observed contract

This slice only answers whether the pinned native source can be built and tested;
it does not add a Cargo dependency, bind C symbols, read a representative `.dta`,
or claim DTA support. The upstream header exposes a callback-based parser with
explicit handlers for metadata, variables, values, value labels, errors, and
progress. The DTA entry point is `readstat_parse_dta`; parser and callback APIs
also expose variable labels, value-label sets, and system/defined/tagged missing
value predicates and missing ranges.

Those symbols are useful feasibility evidence, not a Rust-facing contract. A future
adapter must copy callback data into Rust-owned values before returning from each
callback, translate all native error codes, and free the parser on every path. Raw
parser pointers, C string pointers, label-set pointers, and the `void *` context
must not escape a low-level adapter or enter domain/application APIs.

## Build and test evidence

| Platform | Result | Exact scope |
| --- | --- | --- |
| macOS Apple Silicon | Pass, local | Pinned release tarball; `./configure --disable-shared --enable-static LIBS=-liconv`; `make -j10 libreadstat.la CFLAGS=-Wno-strict-prototypes`; `make check CFLAGS=-Wno-strict-prototypes`; four upstream tests passed |
| Linux x86_64 | Pass ([PR #7 workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/34197952522)) | Pinned release tarball; static library and complete upstream build with `-Wno-error=stringop-truncation -Wno-error=use-after-free -Wno-error=format-truncation`; `make check` passed all four tests; hosted CI evidence |

The macOS run found two portability prerequisites rather than silently hiding them:
ReadStat's default `-Werror -pedantic-errors` flags turn Apple Clang's
`-Wstrict-prototypes` diagnostics into build errors, and the configure test did
not add macOS's `-liconv` automatically. Suppressing only that warning and passing
`LIBS=-liconv` allowed the unmodified release library to build and pass all four
upstream tests. The Linux probe builds the static library and complete upstream
tree, then runs `make check`; it does not execute the optional command-line tools.
GCC's `-Werror` reports `-Wstringop-truncation` in the release writer
implementation, `-Wuse-after-free` in both the CLI error path
(`src/bin/readstat.c:377-383`) and a SAS7BDAT error path
(`src/sas/readstat_sas7bdat_read.c:580-588`), and `-Wformat-truncation` in the
SPSS writer. The probe suppresses only these warning-to-error promotions to measure
the build/test surface; the findings remain production-adoption blockers and are
not silently treated as clean source. These are compatibility and source-quality
findings, not evidence that a Rust wrapper is safe or that native defaults are
portable.

The workflow verifies the release archive before extraction, builds the static
library and complete upstream tree with the documented warning demotions, and runs
the upstream `make check` suite. It does not execute the known-flawed CLI, run a
generated binding, convert a fixture, or run an integration test because none exists
in this slice.

## Rust binding and DTA ingestion evidence

A two-crate spike now lives under `spikes/readstat-prototype/`:

- `readstat-sys/` — the low-level FFI adapter. It is the only place that touches
  foreign pointers or `unsafe`. It builds the pinned release from the vendored
  tarball (`build.rs`), declares the raw C ABI, and exposes a safe `parse_dta`
  entry point that returns owned Rust values. `unsafe_op_in_unsafe_fn` is denied
  crate-wide; every `unsafe` block carries an adjacent `// SAFETY:` rationale.
- `tabdat-readstat-spike/` (the crate root) — a `#![forbid(unsafe_code)]` safe
  facade that re-exports the owned result types and forwards to `parse_dta`. The
  compiler enforces that no foreign pointer, parser handle, or C string appears
  here.

The `build.rs` reproduces the documented platform workarounds and adds one new
macOS finding: newer Apple Clang releases require
`-Wno-implicit-const-int-float-conversion` for `src/sas/readstat_sas.c` (in
addition to the previously recorded `-Wno-strict-prototypes` and `LIBS=-liconv`).

### Callback ownership model (verified against the pinned source)

- DTA callback order is **metadata → variables → data rows → value labels**; the
  value-label handler fires after all rows, so a single `ParserState` accumulator
  is complete when `readstat_parse_dta` returns.
- The variable handler's third parameter (`const char *val_labels`) carries the
  value-label set name; there is no `readstat_variable_get_label_set` accessor.
- `readstat_value_t` is a 16-byte, align-8 pass-by-value struct (union + type +
  tag + bitfields). It is modeled opaquely and read only through the C accessor
  functions, so the internal union/bitfield layout is never interpreted from Rust.
- The parser handle is created and freed exactly once on every path (success,
  native error, and callback abort); trampolines copy borrowed C data into owned
  values before returning and wrap the body in `catch_unwind` so a Rust panic
  aborts the parse instead of unwinding across the FFI boundary.

### Fixture and test evidence

`fixtures/labeled.dta` is generated by `fixtures/generate_labeled_dta.c` using the
ReadStat writer API (pandas cannot write tagged missing values). It carries
variable labels, a numeric value-label set, a string value-label set, system
missing, and tagged missing (`.a`/`.b`). `fixtures/patients.dta` is generated with
pandas to match the Python oracle's `sample_dta` fixture. Seven integration tests
in `tests/dta_ingestion.rs` assert the ingestion contract; all pass locally on
macOS Apple Silicon.

### Findings that constrain the future adapter

| Finding | Evidence | Consequence for TabDat |
| --- | --- | --- |
| String value-label keys are not preserved. | The writer stores `0` for every string key (`val=[0,0]` in the DTA bytes); the reader delivers that raw int32. | Numeric value labels round-trip fully; string value-label keys are lost. TabDat must not rely on ReadStat for string value-label keys, or must resolve them from the string table itself. |
| Missing strings arrive as empty strings, not system missing. | A missing string cell is delivered as `READSTAT_TYPE_STRING` with an empty payload and no missing flag. | `DtaValue::is_missing` does not flag an empty string; treating an empty string as missing is a TabDat data-semantics decision, not a ReadStat one. |
| Numeric value labels, variable labels, storage types, and system/tagged missing are preserved. | `income_band` set (1→low, 2→mid, 3→high), per-variable labels and `DtaType`, and `.a`/`.b`/`.` on the `score` column all assert correctly. | The core ingestion surface is sound for the numeric/label/missingness cases that dominate Stata data. |

These are compatibility and source-quality findings, not evidence that a Rust
wrapper is safe or that native defaults are portable. The spike does not claim
DTA support or migration parity.

## Ownership, safety, and licensing ledger

| Boundary | Observed evidence | Decision/risk |
| --- | --- | --- |
| Native API | Callback functions receive borrowed metadata, variable, value, and label pointers plus a caller context; `readstat_parser_free` owns parser teardown. | A future low-level crate must model lifetimes operationally by copying callback data and guaranteeing teardown on success, error, abort, and panic boundaries. |
| Missingness and labels | Header functions distinguish system, defined, and tagged missing values; variable labels, label sets, value labels, and missing ranges are exposed. The spike confirms numeric value labels, variable labels, storage types, and system/tagged missing round-trip; string value-label keys are lost and missing strings arrive as empty strings. | Semantics remain unvalidated against TabDat/Python fixtures; do not mark roadmap label or missingness items complete. |
| Build dependencies | Release build uses Autotools, C99, zlib when available, and iconv for text conversion; macOS needs the explicit compatibility flags above. | Packaging, ABI, allocator, thread-safety, and cross-compilation obligations need a separate adapter decision. |
| License | The pinned source `LICENSE` is MIT; retain its copyright and permission notice with any redistribution. | Native redistribution and notices still require the project license review before runtime adoption. |

## Decision and remaining risks

The `readstat-sys`-style low-level binding and the `forbid(unsafe_code)` safe
facade are now in place and load a representative DTA fixture without pandas,
establishing callback ownership, variable-label and numeric value-label
preservation, storage types, and system/tagged missingness. The two documented
limitations (string value-label keys are not preserved; missing strings arrive as
empty strings) must be carried into the future adapter decision. The candidate is
still not product support, DTA support, or migration parity.

Remaining risks include the age of the `v1.0.0` release, the Apple Clang warning
workarounds, native iconv/zlib linkage, callback reentrancy and abort behavior,
format-version coverage, the string value-label key loss, and the absence of
trusted fixture/reference outputs.
