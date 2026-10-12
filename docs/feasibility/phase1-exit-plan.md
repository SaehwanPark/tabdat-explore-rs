# Phase 1 exit decision plan

Base `ca10ac0`, 2026-10-12. Risk: **high** (native boundaries/backend decisions).
User authorizes implementation, defensible specification changes and autonomous
PR handoff/merge until all six Phase 1 exit items are evidenced and checked.

## Scope and execution

1. Review all first-party ReadStat/gretl/DuckDB adapter code, immediate native
   ownership contracts and known failures. Fix actionable exposed safety defects,
   with tests, before accepting the reviewed boundaries. Record native/transitive
   trust and unresolved production work rather than pretending review is proof.
2. Evaluate licensing/build/initialization costs and baseline numerical reports.
   Select/reject each backend and an initial linear-algebra route in an ADR.
   Keep rejected Vega draft #164 unmerged. Do not waive dependency policy.
3. Produce the final feasibility report, semantic mismatch index and P0 disposition
   ledger. Define P0 as an unresolved issue preventing the *evaluated initial
   Rust-native capability* or invalidating its safety/numerical evidence. An issue
   outside that scope must retain a concrete later integration/release gate.
4. Replay macOS and emulated Linux native tests, renderer/DuckDB tests and all
   root/policy checks. Use sanitizers where supported and label their limitations.
5. Update roadmap/specification consistently: Phase 1 is a backend feasibility
   decision, not product parity, release packaging or proof that all transitive C
   is defect-free. Check each exit gate only after its recorded review disposition.
   Preserve production obligations in later phase gates; do not erase failures.
6. Three explicit owner review passes plus follow-up, PR handoff/merge, clean main.

## Intended decisions to verify, not predetermined acceptance

DuckDB remains canonical. libgretl is the preferred broad classical/econometric
backend alongside the existing native Rust utilities; ReadStat is DTA-only.
Reuse libgretl's already required BLAS/LAPACK before introducing a competing
linear-algebra runtime. Native SVG/resvg replaces audit-blocked Vega under the
existing ADR. Reject other ReadStat formats/CLI and unsupported native model
variants until separately validated. No shipping acceptance is inferred.

## Verification and constraints

Existing fixtures/tolerances/sample identities remain unchanged. Add tests for
any adapter fixes. Existing numerical/reference reports are evidence only for
explicitly named variants, not broad family support. No Python/R runtime,
policy suppression, advisory waiver, speculative unsafe code or unmeasured
performance superiority. A demonstrated memory/ownership defect on the accepted
path remains a P0 until fixed or that path is rejected.

Implement exactly this plan. Do not broaden scope. If the plan conflicts with
the codebase, record the conflict and revised evidence-backed decision rather
than improvising silently.
