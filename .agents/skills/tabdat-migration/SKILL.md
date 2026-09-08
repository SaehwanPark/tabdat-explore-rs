---
name: tabdat-migration
description: Use when recovering Python TabDat behavior or porting a bounded command, parser, script, session, CLI, reporting, or MCP contract into Rust with explicit parity evidence.
---

# TabDat behavioral migration

## When to use

Use for contract recovery and end-to-end migration, not mechanical Python source
translation. Pair with the data, statistics, or native-backend skill only when
that boundary is touched. This skill owns public behavior; specialists own their
domain checks.

## Required inputs

- Requested command/capability and bounded acceptance criteria.
- Current Rust source/tests and relevant proposal/roadmap sections.
- Python oracle checkout and exact revision, authoritative behavior docs, reusable
  fixtures, and any recorded intentional deviations.

Read [AGENTS.md](../../../AGENTS.md), proposal §§9–10, the relevant roadmap
phase via its linked documents, and [the pinned migration authority](../../../docs/migration/README.md).
Recover the exact revision and verify the checkout, tree, lock digest, and relevant
paths before recording oracle output. Without a verified checkout or required
fixtures, local design/prototyping can proceed if in scope, but parity is blocked.

## Workflow

1. Inspect what exists before choosing an implementation surface. Select one
   observable workflow rather than scaffolding every proposed crate. Identify
   whether the task is contract recovery, implementation, or validation.
2. Recover the Python contract: accepted syntax/options/defaults; typed inputs
   and outputs; state preconditions/effects; errors; missingness/coercion/order;
   script and serialization behavior. Cite oracle files/tests and revision.
   Mark unresolved contradictions; Python output alone is not proof of correctness.
3. Define the Rust contract: command/result/error types, ownership, capability
   requirements, atomic state transitions, and dependency direction. Avoid
   generic dictionaries or independently nullable model-family fields where
   enums express valid states. Record intentional changes before blessing fixtures.
4. Define tests before implementation. Reuse canonical `.td` workflows and data;
   compare semantic JSON, errors, and state separately from terminal goldens.
   Specify exact fields and any justified normalization, not blanket string cleanup.
5. Implement the smallest complete slice using domain-owned semantics. Keep
   syntax-only parsing independent of backend initialization. Route human, JSON,
   and MCP interfaces through the same typed result rather than separate logic.
6. Run relevant unit/integration and differential checks plus root Cargo checks.
   Investigate divergence by input, parse, state, execution, and rendering layer.
   Fix the underlying contract or document a justified deviation; do not rewrite
   golden files merely to fit Rust output.
7. Update only implemented/validated roadmap items and decisions. Report deferred
   forms explicitly; passing a supported subset is not whole-command parity.

## Boundary-specific checks

- Parser/scripts: precedence, quoting, identifiers, options, missing literals,
  macro expansion, multiline SQL, nested `run`, recursion rejection, and file/line
  diagnostics. Preserve documented behavior rather than inventing Stata compatibility.
- Session/runtime: capability errors, state before/after success and failure,
  deterministic typed errors, model-family compatibility, and cancellation.
- CLI/reporting/MCP: repeated commands, script execution, JSON schema/errors,
  discovery/help, deterministic formatting, shared result semantics, and clean
  machine-output channels according to the recovered contract.
- Interactive paths: completion must not materialize data or initialize stats;
  metadata-only commands must avoid unnecessary heavy initialization. Aspirational
  latency targets are not achieved guarantees without measurements.

## Expected outputs and validation

Return four named sections: **Python contract**, **Rust contract**, **Test
contract**, and **Implementation mapping** (Rust/DuckDB/native/optional/deferred).
Include source revisions, changed paths, exact check commands/results, supported
scope, deviations, and completion state: `complete`, `partial`, or `blocked`.

For a resumable handoff use the [shared artifact contract](../../../docs/harness/tabdat/team-spec.md).
Local tests may pass while oracle comparison remains blocked. Do not mark
migration complete with missing fixtures, ambiguous public behavior, or unexplained
divergence; name the evidence or decision needed to resume.
