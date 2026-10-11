# Python-free visualization feasibility

Status: **partial / blocked on dependency policy**. Candidate `vl-convert-rs`
1.9.0 renders a Rust-generated scatter specification to SVG and PNG without a
Python runtime, but its reproducible dependency graph fails the required audit.
Do not adopt it or mark Phase 1 complete from rendering success.

## Contract and provenance

- Isolated `spikes/visualization-prototype/`, `forbid(unsafe_code)`, inline
  synthetic rows `(x,y) = (1,2), (2,1), (3,4)` only.
- Python oracle: clean commit `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`,
  tree `601b236788872323af9277d2276a236154a0f129`, lock SHA-256
  `0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.
- Recovered `src/tabdat/visualization.py::save_scatter`, `_base_chart`, and
  `_save_chart`: inline records, point mark, quantitative x/y, field titles,
  default view size 300×300, SVG/PNG output. No filename, session or CLI port.
- External environment: Python 3.13.3, Altair 6.1.0, vl-convert-python 1.9.0.
  This is the existing oracle environment, not a fresh lock sync.
- Rust and Python explicitly match Vega-Lite 6.4 (schema 6.4.1). The initial
  Rust 5.21 orientation was replaced before comparison to match the actual
  oracle version; no output/tolerance was altered to hide a disagreement.
- `fixtures/python_scatter.py` regenerates Python artifacts externally;
  `fixtures/python-scatter.json` is the recovered expected spec. Rust tests
  compare parsed JSON exactly (object key order is irrelevant).

## Build findings and security blocker

Fresh 1.9.0 resolution fails: Deno pins Tokio 1.36.0, newer `tokio-util`
requires APIs it lacks, and old SWC imports the removed `serde::__private`.
Starting from the **published 1.9.0 Cargo.lock** allows compilation; the spike
lock preserves that historical graph, with its own package and direct pins.
Do not update away those failures without recording a tested resolution.

`cargo audit --file spikes/visualization-prototype/Cargo.lock -D warnings`
reports **19 vulnerabilities, 32 denied warnings** (16 unmaintained, 8 unsound,
8 yanked). Examples include crossbeam-channel double-free
[RUSTSEC-2025-0024](https://rustsec.org/advisories/RUSTSEC-2025-0024), Tokio
broadcast unsoundness [RUSTSEC-2025-0023](https://rustsec.org/advisories/RUSTSEC-2025-0023),
RSA timing attack [RUSTSEC-2023-0071](https://rustsec.org/advisories/RUSTSEC-2023-0071),
and numerous TLS, buffer and parser advisories. The lockfile contains 682 packages.
These are graph findings, not claims that this fixed-inline-data fixture
exercises every vulnerability. No advisory ignores were added.

The published `vl-convert-rs` **2.0.0-rc7** lock was separately audited:
**4 vulnerabilities, 7 denied warnings**. That graph was not built or adopted;
its audit is not an audit of a newly resolved/pruned application lock. An
updated candidate/graph could improve the result, but must prove policy,
compatibility and artifact behavior before acceptance. No production dependency
or root lockfile change is made. This PR remains draft and must not merge while
these required policy failures remain unresolved.

## Ownership / license review

`VlConverter` owns a worker-thread/channel wrapper over Deno/V8. Async methods
return owned `String`/`Vec<u8>`; no raw foreign handle escapes our safe spike.
`new()` creates workers, while first rendering performs substantial lazy work.
The spike constructs the converter only in the render path. No project-written
unsafe code, callbacks, custom Send/Sync, or model handle is introduced.
Options disallow external base URLs and fixtures use inline data; this is not
an arbitrary-spec sandbox guarantee, especially for image/font resolution.

The wrapper declares BSD-3-Clause, but Deno/V8, font/rendering/native and all
transitive licenses/notices still need a full AGPL compatibility and binary
redistribution review. No production license approval is inferred from the
wrapper's SPDX label. No sanitizer or complete transitive unsafe review ran.

## Semantic evidence (local macOS 27.0.1 arm64)

Three tests pass: inline spec/axes, exact Python spec, and Rust SVG/PNG rendering.
External Python and release Rust artifacts match **byte for byte**:

| Artifact | Bytes | Dimensions | SHA-256 |
| --- | ---: | --- | --- |
| SVG | 8,953 | 351×347 | `a374d5b5d41e51eba0aa7b58ac974bfaeac9424fc7a79f134a82691d23b927fe` |
| PNG | 32,467 | 351×347 | `39b57417fbae45494fff9dcb649bbd444ef432ee0682bdc459f92b31b3c1a5ed` |

SVG point labels are exactly `x: 1; y: 2`, `x: 2; y: 1`, `x: 3; y: 4`.
This is one nominal scatter fixture, not histogram/bar/Bayesian or font/platform
parity. Fonts can affect other-machine results; do not promise universal hashes.

## Release measurements

One orientation run, compilation excluded, filesystem/page caches uncontrolled,
three-point fixture, pinned repository toolchain:

```text
construction_us=414
first_svg_us=255732
png_us=24780
repeated_10_svg_us=168484
process_wall_seconds=2.01
maximum_resident_set_size_bytes=101728256
binary_size_bytes=114164992
```

First-render time includes embedded JavaScript initialization and compilation.
PNG runs after the first SVG. The process wall measurement includes Tokio,
artifact I/O, all renders and teardown; it is not isolated converter latency or
cold-start certification. No baseline/threshold or Linux build was measured.
The ~114 MB binary and ~102 MB RSS motivate optional/lazy distribution review.

## Reproduction and review

```sh
CARGO_TARGET_DIR=target/visualization-spike cargo test --manifest-path spikes/visualization-prototype/Cargo.toml --locked
CARGO_TARGET_DIR=target/visualization-spike cargo build --manifest-path spikes/visualization-prototype/Cargo.toml --locked --release
/usr/bin/time -l target/visualization-spike/release/measure /tmp/tabdat-viz-rust
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=/path/to/oracle/src /path/to/oracle/.venv/bin/python spikes/visualization-prototype/fixtures/python_scatter.py /tmp/tabdat-viz-python
cargo audit --file spikes/visualization-prototype/Cargo.lock -D warnings
```

Spike format/test/release/Clippy pass with dependency future-incompatibility
warnings; audit fails as above. Three explicit owner review passes covered
spec/oracle integrity, worker/owned outputs and network scope, and dependency/
performance/acceptance claims. Blocking finding: candidate dependency policy
fails; no merge-readiness. No independent reviewer runtime was available.
Root workspace checks remain independent of this excluded candidate.

## Decision / handoff

Defer production adoption and the plotting exit gate. Preserve the spike only
on draft PR #164 for investigation. Next: test a freshly resolved current
candidate, seek upstream dependency updates where no patched graph is feasible,
rerun audit/deny/native license review and compare the same artifacts. Do not
silently replace the required `vl-convert-rs` experiment or weaken policy.
Other estimator feasibility work can proceed independently. Phase 1 remains
incomplete until this blocker and its other exit gates are resolved.
