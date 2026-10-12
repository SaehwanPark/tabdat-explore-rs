# Phase 1 final backend feasibility report

Date: 2026-10-12. Base `ca10ac0`; [exit PR #183](https://github.com/SaehwanPark/tabdat-explore-rs/pull/183).
**Disposition: Phase 1 complete; all six exit gates accepted.** This
report supersedes the preliminary assessment's undecided backend dispositions.
[ADR 0021](../adr/0021-phase1-backend-and-linear-algebra-decisions.md) records
initial architecture acceptance, not binary shipping approval or broad parity.
Subsequent [triage #184](reshape-collision-replay.md) closes reproduced fixture
ownership issue #167 and rejected/superseded renderer #164, preserving historical
uncertainties and the rejected branch. Earlier open/draft statuses below record
this report's #183 closeout snapshot.

## Gate interpretation and scope

Phase 1 resolves whether the initial Rust-native capabilities are feasible and
which backends the port should use. A successful spike is a reproducible bounded
contract, not every native option, production integration, hostile-file safety,
release packaging or a proof that transitive C is defect-free. The original exit
items are retained, not deleted. “Unsafe boundaries reviewed” means first-party
FFI/ownership/callback code and immediate upstream ownership contracts were
reviewed, findings tested/fixed or rejected, and the remaining dependency trust
and integration/release prerequisites recorded. It does not mean “all unsafe
code is proven safe.” This distinction is justified by Phase 1's technical-spike
goal and the existing later integration/validation, Phase 14 packaging and
Phase 15 cutover gates.

A P0 feasibility blocker invalidates accepted numerical/safety evidence or
prevents an evaluated initial capability. Merely renaming a demonstrated defect
as future work does not close it: the ReadStat I/O defect and DuckDB advisory
found during this review were fixed and replayed. Unsupported variants are
explicitly not accepted; they need new contracts before execution integration.

## Selected backends and cost decisions

| Capability | Decision | Evidence / constraints |
| --- | --- | --- |
| Data engine | Accept bundled DuckDB as canonical | [DuckDB report](duckdb.md); CSV/Parquet, owned Arrow, loopback HTTP/unsigned S3. No Polars; TLS/credential/offline-extension policies later |
| DTA ingestion | Accept ReadStat 1.0.0 DTA-only, with pinned read-error/constructor patches | [ReadStat report](readstat.md), [pointer bridge](readstat-value-boundary.md); labels/types/tagged missing, RAII/dimension tests, native I/O fault injection. Other formats/CLI/writers rejected from initial scope |
| Broad compiled statistics | Accept libgretl 2026b as primary, not exclusive | Family/reference matrix below; safe facades, owned results, one source-matched serialized core, optional independent modules; no R/Python runtime |
| Initial compiled linear algebra | Reuse libgretl's BLAS/LAPACK | Accelerate on macOS; system BLAS/LAPACK on Linux. Existing safe Rust QR utilities retained. No new generic matrix API or faer/standalone BLAS dependency without a caller/comparative evidence |
| Rendering | Accept native SVG/resvg 0.48.1 | [ADR 0011](../adr/0011-static-svg-renderer-feasibility.md), [renderer report](native-svg.md); prepared categories/explicit bins, caller font; no Vega-Lite parity |

No accepted FFI dependency is too costly for this initial capability envelope.
DuckDB's large C++ build buys the canonical engine; libgretl's native burden buys
validated estimator breadth; ReadStat's narrow bridge buys DTA labels/missingness.
Build time is material, particularly under emulation. Reject duplicate data
engines/cores, unnecessary general BLAS FFI, broad ReadStat formats, prebuilt/
custom-BLAS acceptance and audit-blocked mandatory Vega. Draft #164 stays unmerged.
This is an evidence-backed reuse/cost decision, not unmeasured faer speed superiority.

## Numerical and semantic-mismatch index

Existing fixtures keep their original tolerances and exact sample/parameter
identities. Rust tests replay these frozen trusted outputs; original external
Python/R runs and pinned oracle provenance remain in each linked report, not
invented as fresh reference runs here.

| Accepted baseline | Validation / intentional difference |
| --- | --- |
| OLS, HC1, one-way cluster | [libgretl](libgretl.md): NIST and Python/R references, full covariance, explicit finite-sample correction and inference df |
| Classical logit/probit | [libgretl](libgretl.md): Python/R MLE, observed covariance/likelihood, separation/error recovery |
| Integer classical Poisson | [count](poisson-plugin.md): Python/R covariance/likelihood; fractional/offset/robust variants not inferred |
| NB2 | [NB2](negative-binomial.md), ADR 0013: raw alpha/full observed beta-alpha covariance, refined references; original lnalpha naming/optimizer discrepancy preserved |
| Classical IV/2SLS | [IV](iv2sls.md): Python/R unadjusted covariance and structural R²; not universal IV inference |
| Balanced FE / explicit-intercept RE | [panel](panel.md): Python/R plm full covariance and canonical keys; default no-constant RE and broader panel options unaccepted |
| Left/two-sided Gaussian Tobit | [Tobit](tobit.md), ADR 0014: joint beta/logscale observed covariance; not approximate fallback-BFGS SE parity |
| Quantile .25/.5/.75 LP | [quantile](quantile.md), ADR 0015: HiGHS/R point estimates and canonical iid Epanechnikov/Hall–Sheather covariance; original IRLS/native sparsity differences explicit |
| Exponential/Weibull PH | [survival](survival.md), ADR 0016: full PH/logshape Jacobian and time-density likelihood; not original approximate-SE parity |
| Proper two-step Heckman | [selection](heckman.md), ADR 0017: corrected outcome and probit blocks, distinct samples; native zero cross-block placeholders are not joint independence/covariance evidence |
| Collapsed one-step difference GMM | [dynamic panel](dynamic-panel.md), ADR 0018: entity sandwich, native sample/group/instrument counts; original independent-row SE mismatch and just-identification limit retained |
| DTA labels / missingness | Numeric labels round-trip; native writer loses string label keys; empty strings remain strings, system/tagged missing explicit. No undocumented empty-string conversion |
| Charts | Exact data/mark semantics, not pixel/layout/font identity; Linux DejaVu and macOS Arial are evaluated caller fonts, not bundled licenses |

## Reviewed ownership / unsafe boundaries

The owner read all first-party low-level Rust/C adapter files and build scripts,
traced facade call sites, and checked immediate dependency source contracts.
This is owner review, not an independent security audit.

| Boundary | Review and enforced invariant | Evidence / retained trust |
| --- | --- | --- |
| DuckDB `ActiveRelation` / immediate duckdb-rs | Owned connection, borrowed statement/iterator; query output is owned Arrow; no custom unsafe or foreign pointers in domain APIs. Reviewed `inner_connection.rs`, `statement.rs`, `raw_statement.rs` connection/result teardown | New batch test reads numeric/string/NULL after statement/connection drop and native-error recovery. Upstream connection/Arrow release callbacks and thread guarantees remain dependency trust; no custom Send/Sync |
| Gretl Rust / C shim | Every foreign handle private; datasets/models RAII, no public cleanup, serialized init/fit/getters/drop; checked counts/indices/separators, successful-model guards; no Rust callbacks into C | Full source read of sys/lib.rs, shim.c/build.rs; pinned dataset.c allocation and gretl_model.c init/free contracts. Model survives dataset drop; combined plugin/thread/error/restoration fixtures pass. Allocation exhaustion remains prototype fail-stop, not fault-injection certification |
| Gretl arithmetic/resource envelope | Existing signed packed-count criterion retained; dataset ≤4096 stored variables, ≤1,000,000 observations and ≤10,000,000 cells; model nominal work ≤1024 parameters and ≤10,000,000 parameter-observation cells | Pure extreme/overflow/boundary tests reject before FFI; safe facade uses conservative ≤1024 stored columns and owned errors. These are prototype limits, not native engine limits or proof of every internal product |
| Gretl model/plugin ownership | MODEL initialized pointers before by-value assignment; model destructor owns buffers; quantile tau freed after copied/scalar metadata; Tobit full_vcv borrowed only under owner; panel metadata restored; Heckman temporary Mills removed on all returns | Pinned `gretl_model_new/init/free`, family reports/tests; source-matched modules share one core and process lifetime. Absolute paths/loader effects are later packaging work |
| ReadStat callbacks/value bridge | Native aggregate stays in C; callback-stack pointer copied synchronously; raw declarations private, no globals, catch_unwind bodies, checked callback stream | All scalar types/string copies/missing flags, labels and real native handler-abort recovery. Native source invokes handlers on calling stack and cleans dta context before return |
| ReadStat parser/results | Three native constructor allocations are null-checked with exact partial cleanup; non-null parser has one RAII owner; metadata/index/order/cell caps checked before row growth; final dimensions checked; boundary failures owned; non-UTF8/NUL paths rejected without substitution | 13 Rust tests each platform. ≤32767 variables / ≤10,000,000 cells, nonnegative rows, positive columns; unsupported zero-column files return a boundary error |
| ReadStat native label-table reads | Signed read errors cannot bypass unsigned-length checks; partial table payload and seek/read errors return failure instead of consuming unread memory/silently succeeding | Unpatched probe fails at injected read 56 (returns OK); pinned patch passes all 66 injected read errors and same-parser recovery. Full native DTA reader/bridge probe runs UBSan on both platforms; three injected constructor allocation failures return NULL and free exactly the preceding allocations |
| Renderer | Safe Rust, XML-escaped generated SVG only, fixed raster dimensions, explicitly loaded one caller font; no arbitrary SVG/file/URL surface | 8 tests each platform, invalid-data/font recovery and real raster marks; dependency policy clean. Font parser/renderer internals remain audited dependency trust, not whole-font fuzz certification |

Root first-party unsafe policy passes for all four packages. Full nested inventories
retain **gretl 93 / ReadStat 84 used unsafe expressions**, including dependency
attribution in each facade; facade compiler `forbid(unsafe_code)` stays true.
Inventories are not safety proofs. No foreign handle gets speculative Send/Sync.

## Platform / verification matrix

macOS 27.0.1 arm64 M1 Max, Rust 1.97.1; existing emulated Linux x86_64
`tabdat-phase1-linux` Rust 1.97.1 Bookworm container. Linux is not physical-host
performance or sanitizer/leak certification.

| Check | macOS | Emulated Linux |
| --- | --- | --- |
| Gretl all-feature / default tests | 57 / 28 pass | 59 / 30 pass |
| Gretl manual missing-module tests | 6 intentionally ignored in automatic batch; historical actual module-removal evidence retained in family reports | Same distinction; not counted as new passes |
| ReadStat Rust | 13 pass | 13 pass |
| Native DTA reader UBSan | 3 constructor allocation faults with exact partial cleanup + 200 success/value-abort/native-error parses + 66 injected read failures/recovery pass | Same probe passes; C only, not Rust instrumentation |
| DuckDB default / explicit httpfs test | 3 + 1 pass | 3 + 1 pass; final Clippy replay passes |
| SVG/PNG renderer | 8 pass | 8 pass |
| Root locked fmt/check/test/Clippy, strict audit/deny and first-party unsafe policy | pass | Root full runtime not reasserted here; named spike tests only |
| All four standalone spike audit/deny | pass, no waiver | Same lockfiles/source; policy tools run on host |

Exact commands use `cargo {check,test,clippy} --locked --manifest-path
spikes/<name>/Cargo.toml --all-targets`, ReadStat adds `--workspace`, gretl adds
`--all-features` for combined modules, Clippy adds `-- -D warnings`. DuckDB's
explicit opt-in network test uses `cargo test ... -- --ignored`. Native probe:

```sh
sh spikes/readstat-prototype/fixtures/run_native_probe.sh \
  <readstat-sys-OUT_DIR>/readstat <owned-output-directory>
```

`build.rs` verifies the vendored archive before extraction/patching. Native probe
compiles the DTA reader/common C sources, bridge and test driver with upstream-style
`-Werror -pedantic-errors`, `-O1 -g -fsanitize=undefined`; it does not include
unaccepted writer/SAS/SPSS sources or suppress a discovered sanitizer diagnostic.

Failures retained, not counted as passes:

- DuckDB standalone audit found RUSTSEC-2026-0285 in rustls 0.23.44; only rustls
  updated to 0.23.45, no waiver. Affected tests/policy replay pass.
- Initial all-format native sanitizer builds failed on deprecated sprintf in
  DTA writer/SPSS sources; stricter extra-warning reader build also reported
  signed comparisons/initializer diagnostics. No source-quality approval or new
  named-warning suppression. Evaluated reader probe uses the upstream warning
  baseline; its actionable label-read defect is patched and regression-tested.
- ASan runtime on this macOS fails initialization (`sanitizer_malloc_mac.inc:189`)
  before main. No ASan or leak pass is claimed. The first UBSan probe omitted the
  required variable handler and exposed a probe-only null access; registering the
  handler, as the Rust adapter already does, fixes the probe. Corrected UBSan
  runs halt on diagnostics and pass.
- Initial gretl Clippy caught imports made unused by centralized allocation;
  removed, no lint suppression. Linux native build batches timed out and were
  replayed/observed separately; a timeout itself is never a passing gate.

Raw logs are local `/tmp/phase1-*` observations; reproducible commands, source,
test assertions and dispositions above are durable evidence. No assumption that
local temporary logs survive another session.

## Costs and license review

No new performance benchmark or superiority claim is necessary to select the
already validated numerical substrate. Historical measured release costs are
retained: [normal source-build initialization](native-initialization.md) median
1.778 ms on macOS (uncontrolled caches), [owned model calls](libgretl.md) on tiny
fixtures, and [native rendering](native-svg.md). Emulated builds do not establish
physical-host speed. Build/relocation cost is explicitly accepted for development,
not concealed as a finished installer.

[ADR 0021](../adr/0021-phase1-backend-and-linear-algebra-decisions.md) records the
MIT/GPLv3/AGPL-intent compatibility review and native license families, including
GPLv3 §13, retained ReadStat MIT/NCSA notices, and artifact-specific obligations.
No legal/shipping approval of every transitive native artifact is inferred.

## P0 disposition and later hard gates

| Finding | Disposition |
| --- | --- |
| Native callback aggregate ABI assumption | Fixed by pointer bridge, PR #182 |
| ReadStat unchecked callback dimensions / parser ownership / native constructor NULL dereferences | Checked typed stream + RAII and native allocation checks in #183; 3 allocation-fault/cleanup cases and Rust tests pass |
| ReadStat native read failure silently accepted | Reproduced on unpatched source; pinned patch + all-read fault injection/UBSan pass |
| Native packed/work/dataset arithmetic envelope | Packed guard #181 retained, practical envelope + extreme/owned rejection in #183; no huge native matrices executed |
| DuckDB standalone TLS advisory | Fixed lock update to rustls 0.23.45; strict checks pass |
| Mandatory Vega dependency policy | Route rejected; policy-clean native SVG/PNG accepted under ADR 0011; #164 unmerged |
| Native family numerical mismatches | Explicit corrected contracts/reference ADRs 0013–0018; no widened tolerance, unsupported variants not accepted |
| Native init path / early Heckman dataset mutation | Fixed/reviewed in #179/#177 with diagnostics/restoration evidence |
| Runtime reshape issue #167 | Concrete fixture ownership corrected in #180; causal issue remains open. Not a blocker to named Phase 1 spikes; runtime integration/stability gate, never silently closed |

There is no unresolved P0 **within the accepted Phase 1 envelope** after the
above fixes/rejections. This does not downgrade shipping prerequisites: before
native product integration/release, enforce broad samples/model inference, error/
cancellation/path/remote security policies, hostile-input/fuzz/allocator and
supported ASan/leak testing, physical-host/architecture matrix, relocatable
modules, complete native/transitive license notices and corresponding-source
artifacts (including source patches), and a project-license release decision.
Those obligations remain unchecked in later phases and can block integration or
shipping. New evidence invalidating an accepted baseline reopens its gate.

## Review and exit decision

Three explicit owner passes covered (1) ABI/ownership/callback and immediate
native contracts, (2) numerical semantics, error/fault/resource recovery and the
P0 ledger, and (3) policy, licensing/cost decisions and phase/scope consistency.
Follow-up retained the unpatched I/O failure, verified all constructor/read faults,
replayed affected checks and validated roadmap/document links. No actionable
blocker remains in this accepted feasibility scope. Review is **owner**, not
independent review or whole-program security certification.
