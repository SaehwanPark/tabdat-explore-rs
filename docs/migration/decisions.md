# Migration decision log

This log tracks authority changes and intentional deviations, not feature completion.

| ID | Status | Scope | Decision / evidence |
| --- | --- | --- | --- |
| MIG-0001 | Accepted | Repository and baseline | [ADR 0001](../adr/0001-isolated-rust-port-and-migration-authority.md); Python commit `16b45d9b66b0d80f32d4d220e84d81bc5180bdbe` is the initial external oracle. |
| MIG-0002 | Unresolved | `status -1` and `status +1` parser diagnostics | At the pinned Python revision, both inputs return `unsupported token in command: -` and `unsupported token in command: +`. Rust `tabdat-language` preserves the pre-existing generic `status does not accept arguments, if clauses, options, or assignment syntax` diagnostic for both inputs. The difference was reproduced on Python 3.13.3 and Rust after PR #13's regression fix (`622e100`, merged as `f6525b3`). It is outside the `describe` slice and remains unresolved pending a dedicated parser-parity decision; do not silently treat the Rust result as Python parity. |

No behavioral deviations have been accepted. The bounded syntax ports establish
Rust command behavior only for their recorded forms; this is not a claim of
whole-language parity.

For each conflict/deviation, append an entry with a stable ID and details:

- Status: unresolved, accepted, rejected, or superseded.
- Affected contract and Python/Rust revisions; source/doc/test paths.
- Minimal reproduction, expected/observed results, and environment.
- Alternatives, user impact, and rationale (link an ADR for accepted deviations).
- Exact tests/reference evidence and resolution criteria; blocked dependent slices.
- Reviewing PR and superseding entry, if any.

Never delete unresolved evidence or silently rewrite tolerances/fixtures. Baseline
updates also get an entry identifying old/new pins and affected contract checks.
