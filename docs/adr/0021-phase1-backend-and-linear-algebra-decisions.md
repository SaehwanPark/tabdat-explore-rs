# ADR 0021: Initial backend and linear-algebra decisions

- Status: accepted for initial port architecture; binary distribution not approved
- Date: 2026-10-12
- Supersedes the undecided backend dispositions in the preliminary Phase 1 assessment
- Evidence: [final feasibility report](../feasibility/phase1-final-report.md)

## Decisions

1. **DuckDB is the canonical data engine.** Use the existing bundled Rust adapter
   route and owned results. Keep HTTP/S3 extension installation explicit, not an
   automatic metadata/startup effect. Do not add Polars. Evaluated remote scope is
   unsigned loopback HTTP/path-style S3; credential/TLS/offline policy is a later
   integration requirement, not inferred support.
2. **Accept libgretl as the primary compiled classical/econometric backend.**
   Retain existing native Rust inference/OLS utilities, typed capability dispatch,
   explicit samples and covariance conventions. Primary does not mean exclusive:
   quantile covariance and statistical normalizations remain TabDat-owned. The
   tested baseline variants in the report are the initial accepted capability
   envelope, not every native option or existing Python approximation. Keep one
   source-matched core, optional independently selected plugins, serialized calls,
   lazy process-lifetime initialization and owned models. Never expose cleanup
   that invalidates live handles. No R/Python runtime.
3. **Accept ReadStat for DTA-only ingestion.** Use the pointer-only value bridge,
   parser RAII, checked callback dimensions and a checksum-verified 1.0.0 source
   with the documented label-table read-error and constructor-allocation patches. Numeric labels and tagged/
   system missing values are supported by the prototype; empty strings remain
   strings. Do not promise string value-label keys the native writer loses.
   Reject ReadStat CLI, writers, SAS/SPSS and arbitrary hostile-file acceptance
   from the initial capability. Its source-quality findings remain release and
   broader-format blockers. The adapter reports UTF-8/NUL path and resource-limit
   failures rather than silently substituting files or growing unchecked rows.
4. **Select BLAS/LAPACK inside libgretl as the initial compiled linear-algebra
   substrate:** system Accelerate on macOS arm64, system BLAS/LAPACK on Linux
   x86_64. Do not add a second standalone BLAS FFI or faer merely to fill a crate
   diagram. Keep the already implemented safe Householder QR baseline unchanged;
   do not market it as a general-purpose numerical library. A future domain-owned
   general matrix API can adapt a mature substrate when a real caller requires
   it. This is a reuse/ownership/dependency decision, **not an unmeasured claim
   that BLAS is faster than faer**. Comparative benchmarks are prerequisites to
   adding/replacing a substrate, not necessary to reuse the validated one.
5. **Native SVG/resvg remains the rendering route** under ADR 0011. Reject the
   mandatory Vega/JavaScript route: draft #164 remains unmerged with its advisory
   evidence. Prepared categories/explicit bins and caller-provided fonts are the
   accepted feasibility envelope; layout/font redistribution is not approved.

## FFI cost decision

DuckDB's C++ build is substantial but buys the canonical SQL/Arrow/file engine,
not an overlapping execution path. libgretl's native dependencies, source patches,
serialization and module paths are justified by the validated estimator breadth;
source builds are slow but do not impose measured Python/R startup costs.
ReadStat's small DTA bridge is justified by labels/tagged missingness and direct
owned conversion. None of these three dependencies is rejected as too costly
within this scope. Reject prebuilt/custom-BLAS acceptance, duplicate cores,
additional data engines, broad ReadStat formats and mandatory Vega. These would
add unjustified or presently unaccepted cost/risk.

## Licensing and packaging disposition

DuckDB and pinned ReadStat are MIT; the pinned ReadStat copyright/permission
notice is retained at [readstat-MIT.txt](../third-party-notices/readstat-MIT.txt).
The ReadStat release tarball omits that top-level notice; it was recovered from
its pinned upstream commit, not a moving branch. resvg/Rust dependencies pass
cargo-deny; native gretl COPYING is GPLv3, and its quantile NCSA notice remains
retained. [GPLv3 §13](https://www.gnu.org/licenses/gpl-3.0.html#section13) expressly
permits combination with AGPLv3, consistent with the proposal's AGPL intent.

The gretl recipe also uses FFTW (GPL-compatible copyleft), GLib/gettext/GMP/MPFR
(LGPL-family), libxml2/curl/zlib (permissive), and platform OpenMP/BLAS/system
libraries. Those license families do not reveal an architectural AGPL conflict;
this is **not an exhaustive artifact-specific legal/redistribution approval**.
Actual platform versions, nested dependencies, notices, corresponding source,
patches/build/install instructions and system-library exceptions must be reviewed
for every release. No root project LICENSE or binary release is invented here.

Source-only prototype acceptance avoids pretending nonrelocatable Homebrew/
build-owned paths are distributable. Before shipping, Phase 14 packaging (§16 of the roadmap) must demonstrate
relocation, audited native artifacts, complete notices/corresponding source,
physical-host validation and supported sanitizers/leak checks. Unreviewed other
architectures/Windows remain unsupported, not silently accepted.

## Consequences

Phase 1 resolves candidate selection and feasibility, not integration or release
certification. The final report records the reviewed boundary, tested envelopes,
closed P0 defects and concrete later gates. A regression invalidating those
results reopens the corresponding decision/gate. Domain/application APIs retain
safe Rust and no foreign types; broad variants need new contracts/references.
