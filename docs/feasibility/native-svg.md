# Native SVG / resvg feasibility slice

**Disposition: accept the bounded static-rendering spike, not product support.**
2026-10-11, [PR #169](https://github.com/SaehwanPark/tabdat-explore-rs/pull/169).
[ADR 0011](../adr/0011-static-svg-renderer-feasibility.md) records the scope change.
Blocked draft #164 and its exact-output evidence are preserved, not overwritten.
No dependency enters the production workspace.

## Contract and scope

The isolated [Rust spike](../../spikes/native-svg-prototype/src/lib.rs) produces
owned SVG and PNG for finite scatter points, ordered **pre-disambiguated**
category frequencies, and histograms with **explicit bin edges**. Numeric missing
rows are omitted. Nonfinite inputs (even in incomplete scatter rows), invalid
XML characters, missing-only/empty input, invalid bins/out-of-range observations,
duplicate category labels and counts above the exact f64 integer range return
owned errors. Histogram intervals are left-closed and final-edge-inclusive.

Fixed 351×347 canvas / 300×300 plotting area; zero-inclusive numeric scatter
scales and five evenly spaced ticks. Overflowing finite domains are rejected.
SVG generation is pure and deterministic, escaping every data-derived label.
Only generated SVG can reach the PNG renderer: no arbitrary SVG, external
assets, URLs, JavaScript, filesystem output or native handles in the library API.
The measurement executable writes explicitly requested artifacts.

No Vega automatic `maxbins` algorithm, category disambiguation, full axis/layout
UX, long-label sizing, arbitrary Vega-Lite spec interpretation, session mutations
or product plot commands are implemented. Explicit-edge histograms and prepared
category counts are a feasibility contract, **not full Python command parity**.

## Semantic and artifact evidence

External oracle: clean `../tabdat-explore`, commit
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, lock SHA-256
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`.
Python 3.13.3 / Altair 6.1.0 / vl-convert-python 1.9.0 / Vega-Lite 6.4.
Python is validation tooling only.

[Oracle probe](../../spikes/native-svg-prototype/fixtures/python_charts.py)
regenerates three SVG/PNG artifacts. The committed
[mark fixture](../../spikes/native-svg-prototype/fixtures/python-marks.json)
contains descriptions extracted from those artifacts, not expectations invented
by the native renderer. [Comparison script](../../spikes/native-svg-prototype/fixtures/compare_artifacts.py)
checks exact mark data, labels and order, plus scatter relative coordinates to
1e-10 absolute tolerance declared in the probe. Tests independently check the
scatter's three coordinates and the zero baseline. Its circle radius 2.739,
stroke/color and opacity follow the oracle's ordinary point mark.

| Fixture | Confirmed semantics | Python PNG | Native PNG |
| --- | --- | --- | --- |
| Scatter | (1,2), (2,1), (3,4); exact labels/order; relative centers (100,150), (200,225), (300,0) | 351×347 | 351×347 |
| Histogram | 0–1:2, 1–2:1, 2–3:2; missing omitted; fixture's explicit bins match Python maxbins=3 | 347×347 | 351×347 |
| Bar | b:3, `<missing>`:1, a:2; incoming order preserved | 106×383 | 351×347 |

[Comparison output](../../spikes/native-svg-prototype/fixtures/artifact-comparison.json)
records all three PNG byte comparisons as **unequal**. Visual inspection of all
three native PNGs confirms readable axes, blue marks, correct levels/order and
white background. Native ticks differ from Vega's nice ticks; bar labels are
horizontal and bands expand to 300 pixels; histograms use a fixed left margin.
SVG structure, font metrics and raster antialiasing can differ. No byte/pixel
parity or subjective visual-equivalence threshold is claimed.

Seven integration tests cover semantic fixture equality, scatter geometry,
negative/constant/zero and incomplete rows, bin boundary/empty-bin handling,
Unicode/XML escaping, invalid inputs and successful recovery. PNGs are decoded
and inspected for blue pixels, exact dimensions and repeat-byte determinism
**within the same process/font**, not across platforms. Invalid/empty font bytes
are errors; a supplied font database is reused across requests.

Reproduce (run from repository root):

```sh
PYTHONPATH=../tabdat-explore/src ../tabdat-explore/.venv/bin/python \
  spikes/native-svg-prototype/fixtures/python_charts.py /tmp/tabdat-native-svg-oracle
cargo fmt --manifest-path spikes/native-svg-prototype/Cargo.toml -- --check
cargo check --locked --manifest-path spikes/native-svg-prototype/Cargo.toml --all-targets
cargo test --locked --manifest-path spikes/native-svg-prototype/Cargo.toml --all-targets
cargo clippy --locked --manifest-path spikes/native-svg-prototype/Cargo.toml --all-targets -- -D warnings
cargo build --release --locked --manifest-path spikes/native-svg-prototype/Cargo.toml --bin measure
spikes/native-svg-prototype/target/release/measure /tmp/tabdat-native-svg-artifacts
python3 spikes/native-svg-prototype/fixtures/compare_artifacts.py \
  /tmp/tabdat-native-svg-oracle /tmp/tabdat-native-svg-artifacts
cargo audit --file spikes/native-svg-prototype/Cargo.lock -D warnings
cargo deny --manifest-path spikes/native-svg-prototype/Cargo.toml --all-features \
  --locked --config deny.toml check
CARGO_TARGET_DIR=target/geiger-native-svg cargo geiger \
  --manifest-path "$PWD/spikes/native-svg-prototype/Cargo.toml" --all-targets \
  --all-features --all-dependencies --include-tests --locked --output-format Json
```

## Dependency, ownership and safety ledger

Pinned `resvg =0.48.1`, defaults disabled, **text only**. No system-font scanning,
font mmap, raster-image decoders, V8/Deno, TLS/network or Python/R dependency.
Committed application lock: 62 packages including the unpublished spike and dev
fixtures. Strict audit: **0 vulnerabilities / 0 warnings**. Existing deny policy
passes advisories/licenses/sources/bans; only the already allowed duplicate-version
warning for two miniz_oxide versions remains. No policy change or exception.

The library, measurement executable and tests forbid first-party unsafe code.
Full geiger inventory (all targets/features/dependencies/tests) reports zero
first-party unsafe use, no packages without metrics, and two non-Rust README
paths not scanned (flate2/siphasher). resvg/usvg report no used unsafe code.
Upstream unsafe appears in font/raster buffer casts, collections, CRC/SIMD,
compression, string scanning and build macro support (including fontdb, tiny-skia,
bytemuck, arrayvec/smallvec/slotmap, memchr, simd-adler32). This source inventory
is **not a reachability proof or complete transitive safety audit**.

| Resource/boundary | Ownership and constraints | Evidence/gaps |
| --- | --- | --- |
| Chart → SVG | Owned Rust data/string/error; validated finite domains and XML; no IO | Semantic, injection and malformed-input tests |
| Font bytes → fontdb | Caller-supplied owned Vec, database owns font data; choose its named family as sans-serif fallback | Invalid-font tests; one local Arial file; no fuzz/glyph-completeness proof |
| Generated SVG → usvg tree | Safe parser API; tree scoped to one call; shares owned font DB; no arbitrary input/assets | Repeated requests/errors; no untrusted-SVG claim |
| Tree → tiny-skia pixels → PNG | Fixed bounded canvas, owned pixmap and byte Vec, RAII teardown | Actual colored pixels/dimensions, repeat determinism; no sanitizer/Miri proof |

Versioned upstream source: resvg 0.48.1 `src/lib.rs` (`render` takes tree and
borrowed pixmap; crate forbids unsafe); usvg 0.48.1 `src/parser/options.rs`
(owned Arc font DB), fontdb 0.24.0 `src/lib.rs` (`load_font_data` owns bytes and
`set_sans_serif_family`), tiny-skia 0.12.0 `src/pixmap.rs` (owned allocation and
PNG encoding). Ordinary application/domain APIs receive no foreign ownership.

## Platform / licenses / distribution

Tested locally: macOS 27.0.1 arm64, Apple M1 Max, Rust 1.97.1. No Linux, Windows,
WASM, sanitizer, font-coverage or cross-platform raster-parity run was performed.
Tests/measurement read the single local Arial file on macOS; other hosts default
to `/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf`. Override with
`TABDAT_SPIKE_FONT=/path/to/font.ttf`; absent files fail explicitly. The library
itself accepts bytes and discovers no system files.

Key Cargo license metadata: resvg/usvg/read-fonts/skrifa MIT OR Apache-2.0;
fontdb/harfrust MIT; tiny-skia BSD-3-Clause. The full graph passes the existing
AGPL-intent allowlist. These permissive licenses are compatible with the intended
AGPL distribution subject to retaining applicable license/copyright notices and
Apache conditions; no native shared library or reciprocal runtime is introduced.
No font is bundled or redistributed. Apple Arial redistribution is **not**
authorized or required for this local test. A released renderer needs an approved
font asset, glyph strategy and complete third-party NOTICE/license bundle.

## Release measurements

Five fresh processes, compilation excluded; filesystem/page caches uncontrolled.
Three-point scatter only. Each measures first pure SVG, 1,000 full SVG calls,
explicit font read/database init, first complete PNG, and 100 complete PNG calls
(SVG generation + parsing + raster + encoding). `/usr/bin/time -l` captures
process peak RSS; results are orientation, not release acceptance thresholds.

| Run | First SVG µs | Repeat SVG µs | Font read/init ms | First PNG ms | Repeat PNG ms | Peak RSS bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 97.208 | 6.349 | 0.552 | 4.263 | 2.671 | 9,879,552 |
| 2 | 18.625 | 7.055 | 0.201 | 3.521 | 2.867 | 9,207,808 |
| 3 | 21.875 | 5.895 | 0.222 | 2.844 | 2.477 | 10,403,840 |
| 4 | 20.625 | 7.265 | 0.257 | 2.459 | 2.723 | 10,338,304 |
| 5 | 20.584 | 5.871 | 0.210 | 2.869 | 2.450 | 9,633,792 |

Medians: first SVG 20.625 µs, repeated SVG 6.349 µs; font read/init 0.222 ms;
first PNG 2.869 ms, repeated PNG 2.671 ms; peak RSS 9,879,552 bytes (~9.42 MiB).
Release executable: **4,033,600 bytes**. Metadata mode performs no font access:
~0.00 s wall, 2,031,616 bytes peak RSS. SVG itself requires no font/backend init.
The old rejected renderer measured ~256 ms first SVG / ~16.8 ms repeated SVG,
~102 MB RSS / ~114 MB binary: not equivalent algorithms, fonts or chart breadth,
so do not treat these orientation values as a controlled speedup benchmark.

Initial all-system-font experiment consumed **836,468,736 bytes RSS**, with
305.397 ms font discovery. It passed rendering but was rejected on resource cost.
Loading one explicit 755 KiB font resolved the observed cost without enabling
mmap/unsafe FFI. Avoid indiscriminate font discovery in a lazy CLI capability.

## Verification / review / next actions

Spike locked format/check/test/Clippy, release build, oracle/artifact comparison,
strict audit, deny and full unsafe inventory pass. Root verification and owner
review disposition are finalized on the PR before merge. GitHub Actions remain
disabled under ADR 0009; no hosted-check or independent-review claim.

This removes the demonstrated renderer advisory blocker by **changing the
candidate**, not hiding dependencies. Keep #164 draft. Continue plugin-backed
libgretl fixtures and the remaining numerical, platform/native safety,
redistribution and backend-decision gates. Phase 1 remains partial.
