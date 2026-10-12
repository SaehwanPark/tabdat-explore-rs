# Phase 1 execution ledger

Phase 1 remains **partial / blocked**, not complete. The roadmap checkboxes are the acceptance
tracker; this ledger records the current bounded loop and durable handoff.
No spike is product support.

## Slice 1: libgretl boundary hardening (complete)

Base: `8f95f89` on `main`. Owner: current agent (serial execution).

Before extending estimator coverage, enforce the safe low-level API contract:

- Check dimensions, C integer conversions, dataset indices, and variable lists
  before FFI; reject invalid input without entering C.
- Initialize lazily inside the low-level adapter and serialize native calls,
  including destructors. Do not expose process cleanup while handles may live.
- Guard coefficient access on failed models and out-of-range indices.
- Keep variable zero reserved for Gretl's constant even without an intercept.
- Keep raw pointers private, RAII teardown, adjacent safety reasoning, and
  `deny(unsafe_op_in_unsafe_fn)` in the approved adapter.
- Run the existing Longley reference fixture, invalid-input tests, parallel
  independent-fit tests, and root/spike formatting/check/test/Clippy checks.

Prototype contract: invalid low-level arguments may panic **before** FFI;
facade validation returns `GretlError`. No Rust callback enters C, so pre-call
panics do not unwind through C. Allocation failure remains a prototype panic.
No sanitizer, Linux revalidation, or broader estimator parity is implied.

Implementation and eight spike tests pass. Root locked checks, Clippy, audit,
deny, and adapter unsafe inventory completed. Three explicit owner review
passes found no unresolved blocker in this bounded change; no independent
reviewer runtime was available. Accepted lifecycle decision: ADR 0010.
PR: [#162](https://github.com/SaehwanPark/tabdat-explore-rs/pull/162).
The broader Phase 1 unsafe-boundary gate remains open.

## Slice 2: DuckDB HTTP/unsigned S3 Parquet (complete)

Test the pinned bundled DuckDB against an agent-owned loopback HTTP endpoint
serving synthetic Parquet. Validate values, Arrow rows, repeated reads, HTTP
range requests, and missing-object recovery. No third-party dataset, bucket,
credentials, or public endpoint is needed. Keep extension install/load explicit
and opt-in because it can use the network. Record HTTPS/S3 gaps rather than
claiming them from HTTP success. S3 is present in the Python contract, so its
roadmap task cannot be dismissed as not applicable without an explicit deviation.

Also correct the extra EOF blank line noticed in the preceding report and rerun
changed-document links and whitespace checks with fail-fast shell execution.
The preceding merge command did not stop after that whitespace warning; its
report's whitespace/link pass statement is corrected by this follow-up.

HTTP and unsigned S3 checks pass on 4,096 rows. The two narrowly scoped roadmap
spike tasks are checked, with TLS/authenticated AWS parity explicitly excluded.
PR: [#163](https://github.com/SaehwanPark/tabdat-explore-rs/pull/163). No dependency
or lockfile change. Three owner review passes; root and spike checks pass.

## Slice 3: Python-free visualization (blocked; draft PR #164)

On branch `feat/phase1-visualization`, Rust scatter spec/SVG/PNG matches the
verified pinned Python fixture, including byte-identical 351×347 artifacts.
Release measurements are preserved there. Required audit fails on reproducible
1.9.0 graph (19 vulnerabilities / 32 denied warnings); separately, the published
2.0.0-rc7 lock audit fails (4 vulnerabilities / 7 warnings), not a freshly resolved
application audit. No advisory ignores, no merge, no product adoption.
Report/source remain on [draft PR #164](https://github.com/SaehwanPark/tabdat-explore-rs/pull/164),
not main. Recover that branch to investigate a policy-compliant candidate.

## Slice 4: libgretl HC1 / clustered OLS (complete)

Extend the isolated owned OLS boundary with explicit HC1 and one-way cluster
requests and owned covariance matrices. Use a finite, full-rank synthetic fixture,
complete ordered sample, intercept-first coefficients and explicit corrections:
HC1 n/(n-k); cluster G/(G-1)*(n-1)/(n-k). Compare coefficients, all covariance
entries and SEs with pinned Python TabDat and an external R/sandwich reference.
Declare |actual-expected| <= 1e-10 + 1e-9*|expected| before comparison; exact
sample count, order/names and covariance kind. Invalid groups and failed fits
must not poison subsequent classical/robust calls. Configure and restore native
global covariance settings within one serialized shim call. Document native
versus TabDat inference-DF semantics and do not claim estimator breadth.

Visualization draft PR #164 has rendering evidence but an unresolved audit
blocker; it is preserved separately and must not merge as-is.

HC1 and cluster coefficients/SEs/full covariance agree with both Python and
R/sandwich. Eleven spike tests pass, including rank/error recovery and mixed-mode
thread calls. Native classical VCV required explicit materialization; its initial
NaN failure was preserved in the report and corrected without tolerance changes.
PR: [#165](https://github.com/SaehwanPark/tabdat-explore-rs/pull/165).
See the libgretl report for exact provenance, finite-input limitations and DF
metadata difference. No whole-phase completion or primary-backend decision.

## Slice 5: libgretl logit/probit (complete)

Classical binary MLE only, intercept and two finite numeric predictors, explicit
0/1 outcomes and complete ordered sample. Owned coefficients, SEs, full
covariance and log likelihood; verify with pinned Python and R base `glm` using
identical data/links. Predeclare abs 1e-7 + rel 1e-6 for optimizer-derived values;
exact sample/labels. Validate outcome domain and missing/ragged inputs; preserve
native errors and check successful outputs are finite. No robust/cluster binary,
prediction, marginal effects or broader model-state port is implied.

Native reconnaissance found count models dynamically load a plugin via
`get_plugin_function("count_data_estimate")`; the current spike builds only the
library. Plugin packaging must be resolved before those fixtures can pass.

Both classical binary fixtures pass full coefficient/SE/covariance/likelihood
comparisons with pinned Python and independent R observed information. All
sample rows are exact; separation and invalid-data recovery are covered.
PR: [#166](https://github.com/SaehwanPark/tabdat-explore-rs/pull/166).
The full spike now has 13 tests. See libgretl report for Hessian-versus-Fisher
semantics and explicit unvalidated scope.

## Slice 6: libgretl initialization / repeated-model costs (complete)

Add a release measurement executable for explicit serialized initialization,
first owned OLS/binary fits and repeated complete adapter calls. A metadata-only
mode must not call initialization. Run five fresh processes, synthetic 40/200-row
fixtures, record raw timings, process/RSS and executable size; distinguish
backend initialization from dataset fill/result copying and do not call
uncontrolled filesystem/page caches cold startup. No release threshold claim.

Also publish a preliminary feasibility decision matrix and durable Phase 1
handoff: rendering security, plugin packaging, unvalidated estimator families,
platform/unsafe/license gates remain explicit, not checked by inference.

Five fresh release processes measured explicit initialization (median ~15.6 ms),
40-row owned OLS calls (~4.76 µs repeated) and 200-row owned logit calls
(~66.2 µs repeated). Metadata mode avoids init; median full-process RSS ~9 MB.
Fourteen spike tests pass. Report preserves every raw run, cache limitations and
native diagnostic. PR: [#168](https://github.com/SaehwanPark/tabdat-explore-rs/pull/168).

## Slice 7: native static SVG / resvg (complete)

Implement the requested alternative rather than weakening the rejected renderer's
advisory policy. Early draft [#169](https://github.com/SaehwanPark/tabdat-explore-rs/pull/169)
records the bounded contract. Isolated Rust-only SVG and PNG preserve exact
scatter/bar/histogram fixture marks, labels and order; scatter relative geometry
matches Python. Full Vega bin selection/category normalization is not ported.
Layout/ticks/fonts and histogram/bar dimensions differ deliberately.

Pinned resvg 0.48.1 text-only graph passes strict audit (0 vulnerabilities / 0
warnings), existing deny policy and first-party unsafe inventory. Seven tests
cover semantic/geometry/raster output, malformed/missing inputs, escaping and
recovery. Initial all-font discovery measured ~836 MB RSS; loading one explicit
font instead measured median ~9.42 MiB RSS, ~0.222 ms font read/init and ~2.671 ms
repeated PNG. No font is redistributed. ADR 0011 changes the Phase 1 static
rendering candidate, not the production architecture. Linux/full glyph coverage
and transitive safety/distribution acceptance remain open. See
[native report](native-svg.md). Draft #164 remains unmerged and is not superseded
by a claim of Vega-Lite compatibility.

## Remaining loops / handoff

The [preliminary assessment](phase1-assessment.md) records decisions, blockers
and next actions. Static-renderer dependency policy is resolved under ADR 0011;
recover native plugins;
validate the remaining estimator families; review the full native/unsafe/license
matrix; benchmark/select general linear algebra; then review Phase 1 exit gates.
No final backend acceptance or phase completion is inferred from current slices.
The intermittent unchanged reshape fixture failure is tracked in issue #167.

Never check the whole phase merely because a PR merges or Cargo passes.
