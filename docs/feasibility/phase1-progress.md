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

## Slice 3: Python-free visualization (blocked; draft PR #164)

Evaluate pinned stable `vl-convert-rs` 1.9.0 in an isolated safe Rust spike.
Generate an inline-data scatter Vega-Lite specification following the pinned
Python `visualization.py` contract; render SVG and PNG without Python at runtime.
Compare with externally generated Python artifacts using structure/dimensions
and the same explicit Vega-Lite version, not arbitrary byte golden replacement.
Measure converter construction, first render, repeated rendering, process/RSS
and binary size in release when the candidate builds. Record dependency licenses,
network restrictions and build failures before any acceptance decision.

Rust spec/SVG/PNG tests and exact Python artifact comparison pass. Release
orientation: first SVG ~256 ms, repeated SVG ~16.8 ms/render, ~102 MB RSS,
~114 MB binary. Required audit fails: 19 vulnerabilities / 32 denied warnings
on reproducible 1.9.0 graph. Separately, published 2.0.0-rc7 graph fails audit
with 4 vulnerabilities / 7 denied warnings (not built/resolved as a spike).
No advisory ignores, no adoption, no merge. Detailed evidence and next actions:
[visualization report](visualization.md). Leave this branch safely committed
on [draft PR #164](https://github.com/SaehwanPark/tabdat-explore-rs/pull/164).

## Remaining loops

1. DuckDB HTTP Parquet evidence and explicit S3 initial-parity scope.
2. libgretl covariance and estimator fixtures with oracle/reference evidence,
   semantic mismatch inventory, and first/repeated-use measurements.
3. Python-free Vega-Lite SVG/PNG rendering and artifact comparison/measurement.
4. Backend synthesis, linear-algebra decision, licensing/build-cost decisions,
   and exit-gate review (including ReadStat's existing evidence).

Never check the whole phase merely because a PR merges or Cargo passes.
