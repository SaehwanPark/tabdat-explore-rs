# ReadStat feasibility reconnaissance

- Status: **partial**; the native candidate builds and runs its upstream tests on
  the two target platforms, but no Rust binding or DTA ingestion contract is
  accepted yet.
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

## Ownership, safety, and licensing ledger

| Boundary | Observed evidence | Decision/risk |
| --- | --- | --- |
| Native API | Callback functions receive borrowed metadata, variable, value, and label pointers plus a caller context; `readstat_parser_free` owns parser teardown. | A future low-level crate must model lifetimes operationally by copying callback data and guaranteeing teardown on success, error, abort, and panic boundaries. |
| Missingness and labels | Header functions distinguish system, defined, and tagged missing values; variable labels, label sets, value labels, and missing ranges are exposed. | Semantics remain unvalidated against TabDat/Python fixtures; do not mark roadmap label or missingness items complete. |
| Build dependencies | Release build uses Autotools, C99, zlib when available, and iconv for text conversion; macOS needs the explicit compatibility flags above. | Packaging, ABI, allocator, thread-safety, and cross-compilation obligations need a separate adapter decision. |
| License | The pinned source `LICENSE` is MIT; retain its copyright and permission notice with any redistribution. | Native redistribution and notices still require the project license review before runtime adoption. |

## Decision and remaining risks

Continue ReadStat as an isolated feasibility candidate. Do not create a
`readstat-sys` crate or safe facade until the Linux build is green and a
representative DTA fixture establishes callback ownership, variable/value-label
preservation, numeric/string conversion, and system/defined/tagged missingness.
The candidate is not product support, DTA support, or migration parity.

Remaining risks include the age of the `v1.0.0` release, the Apple Clang warning
workaround, native iconv/zlib linkage, callback reentrancy and abort behavior,
format-version coverage, and the absence of trusted fixture/reference outputs.
