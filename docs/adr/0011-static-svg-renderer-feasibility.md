# ADR 0011: Static SVG rendering without a Vega JavaScript runtime

- Status: Accepted for isolated feasibility evaluation; **not production adoption**
- Date: 2026-10-11
- Scope: Phase 1 static SVG/PNG plotting gate

## Context

The proposal names Vega-Lite and `vl-convert-rs` as candidates. Draft
[PR #164](https://github.com/SaehwanPark/tabdat-explore-rs/pull/164) demonstrates
exact Python scatter output, but its working 1.9.0 graph fails mandatory advisory
policy. A freshly resolved 2.0.0-rc7 application still has four vulnerabilities
and six denied warnings. Deno's exact Rustls/Rand and Hickory release constraints,
RSA without a listed patched release, and maintenance warnings prevent a simple
lockfile update from resolving the gate. Disabling default features does not
remove unconditional runtime capabilities. No exceptions were authorized.

The user requested implementing the suggested native SVG / resvg alternative.
Static artifacts do not require executing arbitrary Vega-Lite specifications.
Preserving data/mark semantics is distinct from reproducing a renderer's exact
bytes, font metrics, automatic bins, and layout.

## Decision

Evaluate Rust-generated SVG and **resvg 0.48.1** for PNG in a separate spike.
Keep SVG construction pure, with typed owned data and errors. Escape and validate
XML labels; do not accept arbitrary SVG, URLs, scripts, or external assets.
Load one explicit caller-supplied font into an owned database only on PNG first
use. Enable resvg's text support, not system-font scanning, memory-mapped fonts,
or raster-image decoding. Avoid all-font discovery: it measured ~836 MB RSS on
the evaluation machine; explicit Arial loading measured ~10 MB process RSS.

Revise the Phase 1 visualization tasks to test static rendering rather than to
require `vl-convert-rs`. Preserve draft #164, its oracle fixture and measurements
as rejected-candidate evidence. Do not close/merge it or assert Vega-Lite parity.
The bounded native SVG report supplies the new acceptance evidence.

## Consequences and limits

- No Python/R, V8, Deno, network or native FFI dependency is added to rendering.
  The production workspace graph remains unchanged.
- Scatter points, pre-disambiguated ordered category frequencies, and explicit
  histogram intervals can be validated separately from rendering machinery.
- Tick selection/formatting, fonts, categorical width and label rotation differ
  from Python. Automatic Vega bin selection, category disambiguation, long-label
  layout, arbitrary specification support and full chart UX are **not ported**.
- Fonts are caller-provided, not bundled. The tested Apple Arial file is not
  redistributed; Linux font/layout and other targets remain unvalidated.
  A production font choice requires its own license/notice and glyph coverage
  evidence. Font embedding/redistribution is not authorized here.
- resvg/usvg and font/raster components expose safe Rust APIs, but their graph
  contains upstream unsafe code. Inventory is not comprehensive safety approval.
  Production integration needs transitive review, fuzz/resource limits and the
  remaining platform/distribution checks.
- Passing this bounded plotting gate does not complete Phase 1 or select the
  primary statistics, ingestion, or linear-algebra backend.

## Evidence

[Native SVG report](../feasibility/native-svg.md),
[isolated spike](../../spikes/native-svg-prototype/src/lib.rs),
[pinned artifact probe](../../spikes/native-svg-prototype/fixtures/python_charts.py),
and [semantic/error/raster tests](../../spikes/native-svg-prototype/tests/rendering.rs).
