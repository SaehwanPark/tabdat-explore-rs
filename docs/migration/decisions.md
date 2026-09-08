# Migration decision log

This log tracks authority changes and intentional deviations, not feature completion.

| ID | Status | Scope | Decision / evidence |
| --- | --- | --- | --- |
| MIG-0001 | Accepted | Repository and baseline | [ADR 0001](../adr/0001-isolated-rust-port-and-migration-authority.md); Python commit `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` is the initial external oracle. |

No behavioral deviations have been accepted or parity established. This does not
mean a cross-implementation conflict audit has occurred; no Rust commands exist yet.

For each conflict/deviation, append an entry with a stable ID and details:

- Status: unresolved, accepted, rejected, or superseded.
- Affected contract and Python/Rust revisions; source/doc/test paths.
- Minimal reproduction, expected/observed results, and environment.
- Alternatives, user impact, and rationale (link an ADR for accepted deviations).
- Exact tests/reference evidence and resolution criteria; blocked dependent slices.
- Reviewing PR and superseding entry, if any.

Never delete unresolved evidence or silently rewrite tolerances/fixtures. Baseline
updates also get an entry identifying old/new pins and affected contract checks.
