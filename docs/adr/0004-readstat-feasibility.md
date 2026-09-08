# ADR 0004: Pin ReadStat for bounded DTA feasibility work

- Status: Accepted for continued feasibility evaluation; production adoption deferred
- Scope: ReadStat source/build reconnaissance only; no Rust FFI or DTA runtime API

## Context

The roadmap identifies ReadStat as the preferred native candidate for Stata `.dta`
input and label handling. The Rust repository must establish reproducible source,
build, licensing, and ownership evidence before adding a binding. The upstream
`v1.0.0` release is old and its generated release archive is preferable to running
unreproducible Autotools generation from a moving source checkout.

## Decision

Use the ReadStat `v1.0.0` release archive, verify its SHA-256 before extraction,
and run the upstream configure/static-library/test path in an isolated feasibility
workflow. Record macOS Apple Silicon evidence separately from hosted Linux evidence.
Keep the candidate outside the root Cargo workspace until a fixture-backed adapter
contract is accepted. Do not treat the upstream source as a clean warning-free
build: GCC reports `-Wstringop-truncation` in the writer,
`-Wuse-after-free` in both the optional command-line tool and a SAS7BDAT error
path, and `-Wformat-truncation` in the SPSS writer under `-Werror`.

A future low-level adapter must own parser teardown, copy all callback-borrowed data
into Rust-owned representations, translate native errors, and keep raw pointers,
foreign strings, label sets, and callback contexts out of domain/application APIs.
The current slice does not create that adapter and does not mark DTA, label, or
missingness support implemented.

## Alternatives

- Build from the Git repository's moving default branch: rejected for this slice
  because source and generated Autotools inputs would not be pinned reproducibly.
- Add a Rust crate or C FFI now: deferred until a representative DTA fixture and
  callback ownership/error contract are specified and tested.
- Select a pure-Rust reader immediately: deferred because the roadmap's candidate
  is ReadStat and no comparative format/label/missingness evidence has been gathered.

## Consequences and verification

The release archive and checksum make the build probe repeatable and preserve a
clear MIT notice/redistribution obligation. The macOS probe required
`LIBS=-liconv` and `CFLAGS=-Wno-strict-prototypes`; this portability finding is
recorded rather than hidden. The Linux probe suppresses only the observed
`-Wstringop-truncation`, `-Wuse-after-free`, and `-Wformat-truncation` warning
promotions, builds the complete tree, and does not execute the optional CLI because
its error path also has a use-after-free warning. All observed warnings remain
adoption blockers. Hosted Linux verification passed in [PR #7 workflow](https://github.com/SaehwanPark/tabdat-explore-rs/actions/runs/34197952522)
before marking the two roadmap library-build items complete.

The probe does not establish ABI stability, allocator/thread safety, callback panic
behavior, DTA semantic parity, or production packaging. A follow-up decision must
be based on representative `.dta` fixtures, trusted/reference outputs, and an
independent FFI safety review. Supersede this ADR if ReadStat is rejected or a
newer pinned candidate is selected.
