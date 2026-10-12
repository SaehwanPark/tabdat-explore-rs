# Native SVG / resvg feasibility slice

Status: **in progress; not product support**. Replaces the blocked renderer
candidate for bounded static-artifact evaluation, not Vega-Lite compatibility.
Draft #164 and its exact-output evidence are preserved, not rewritten or merged.

## Contract checkpoint

- Isolated Rust-only spike; no dependency enters the production workspace.
- Owned SVG and PNG for finite scatter points, ordered pre-aggregated category
  counts, and a histogram with explicit bin edges. Missing numeric rows are
  omitted; nonfinite inputs and malformed bins/counts return errors.
- Fixed 351×347 canvas and 300×300 plotting area. Deterministic geometry,
  escaped labels, accessible mark descriptions, no external assets or URLs.
- Compare scatter coordinates/data, bar order/counts/labels, histogram intervals
  and frequencies with the pinned Python artifacts. No byte/pixel parity claim:
  tick placement, font metrics, and SVG structure deliberately differ.
- Explicit histogram edges are a spike limitation: Python's Vega automatic
  `maxbins` selection and full categorical disambiguation remain future work.
- Strict audit and existing deny policy; no advisory exceptions. Check transitive
  safety inventory, license metadata, invalid inputs/recovery, and release costs.
- macOS arm64 first; Linux and cross-platform font parity remain acceptance gaps.

Implementation, evidence, review disposition, and an ADR follow before acceptance.
