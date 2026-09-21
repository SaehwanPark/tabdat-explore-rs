# Bounded eager-runtime `save` review

Reviewer: independent read-only review agent
Scope: PR #70 implementation and focused contract tests through `8f40c2a`

## Findings and resolution

1. The Rust path does not expand `~` as the pinned Python helper does. This is
   an accepted bounded deviation, recorded in `01-contract.md` and the
   migration evidence; broader path normalization is deferred.
2. The initial test suite lacked a backend-copy failure after successful parent
   preparation. `e2ac065` added a valid-parent overlong target that reaches the
   DuckDB copy and asserts `SaveFailed` with unchanged published metadata.
   `e37a59f` added a recovery save/read-back assertion proving that the private
   active rows remain intact after the failure. The test also documents the
   backend limitation that a failed copy has no atomic temporary-file guarantee.
3. The initial read-back helper sorted rows and therefore did not prove row
   order. `e2ac065` changed the fixture to deliberately non-sorted input,
   removed `ORDER BY`, and asserts the exact sequence for both ordinary and
   transformed saves.

## Review conclusion

The independent review found no remaining implementation defects or P0/P1
risks. SQL path binding is parameterized, active state is preserved on success
and failure, and the focused suite passes `8 passed`. The accepted deviations
and deferred surfaces are explicit rather than silently presented as parity.

## Evidence reviewed

- Python authority: pinned sibling checkout and save execution probe;
- Rust contract and implementation in `01-contract.md` and
  `crates/tabdat-runtime/src/lib.rs`;
- focused tests in `crates/tabdat-runtime/tests/save_contract.rs`;
- local locked workspace checks, policy checks, and `git diff --check`; and
- PR #70 hosted checks: Rust baseline, tabdat-runtime on Linux, and
  dependency/unsafe-code policy, all green on `8f40c2a`.
