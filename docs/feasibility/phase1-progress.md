# Phase 1 execution ledger

Phase 1 remains **in progress**. The roadmap checkboxes are the acceptance
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

## Slice 4: libgretl HC1 / clustered OLS (WIP)

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

## Remaining loops

1. DuckDB HTTP Parquet evidence and explicit S3 initial-parity scope.
2. libgretl covariance and estimator fixtures with oracle/reference evidence,
   semantic mismatch inventory, and first/repeated-use measurements.
3. Python-free Vega-Lite SVG/PNG rendering and artifact comparison/measurement.
4. Backend synthesis, linear-algebra decision, licensing/build-cost decisions,
   and exit-gate review (including ReadStat's existing evidence).

Never check the whole phase merely because a PR merges or Cargo passes.
