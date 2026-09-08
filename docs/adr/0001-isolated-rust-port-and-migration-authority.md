# ADR 0001: Isolated Rust port and pinned migration authority

- Status: Accepted
- Scope: Repository and migration governance; no runtime backend adoption

## Context

The Rust repository contains a binary scaffold while the separate Python repository
contains the working application. Copying module layouts or treating proposed Rust
commands as implemented would obscure behavioral migration. A mutable Python branch
cannot identify which behavior produced a fixture.

## Decision

Keep the Rust port in this separate repository, not inside the Python runtime or
workspace. Pin Python as an external behavioral oracle using the
[migration manifest and authority policy](../migration/README.md). Preserve public
contracts through bounded tests; use Rust-owned domain types and directional
language → execution → backend boundaries, independent of Python module layouts.
Record disagreements/deviations before claiming parity, with separate statistical
reference evidence where applicable. Core Rust distribution must need neither
Python nor R.

## Alternatives

- A shared Python/Rust workspace simplifies fixture access but risks runtime and
  dependency coupling; explicit external validation paths are sufficient initially.
- Following Python `main` without a pin is convenient but makes comparisons mutable.
- Copying all Python code/tests now would add licensing/provenance work and imply
  unsupported breadth; recover only the contract/fixtures needed by each slice.

## Consequences and verification

The repositories evolve independently. Baseline updates are explicit reviewed work;
missing oracle/reference evidence blocks dependent parity claims, not unrelated
build setup. Architecture docs distinguish present source from planned boundaries.
The initial manifest's commit/tree and lock digest were verified against a clean
local checkout and upstream commit metadata; bounded parser/script checks ran.
No implementation or fixture code was copied. Future copying and native dependencies
require license/redistribution review; the Python package declares AGPL-3.0-or-later,
which does not by itself license or package this Rust scaffold.
