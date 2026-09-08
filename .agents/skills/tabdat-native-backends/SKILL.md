---
name: tabdat-native-backends
description: Use when evaluating or integrating TabDat native backends such as DuckDB, ReadStat, libgretl, or plotting libraries, including FFI ownership, capability isolation, packaging, licensing, and startup costs.
---

# TabDat native-backend feasibility and safety

## When to use

Use for a bounded backend spike, dependency decision, safe wrapper, or native
capability integration. Not every backend needs project-written unsafe code;
prefer a maintained safe interface when it satisfies the contract. libgretl,
ReadStat, and other named libraries are candidates until evidence and a decision
accept them. Spatial/Bayesian backends must not delay core work by default.

## Required inputs

- Capability contract, representative fixture, candidate library/API version,
  and proposed supported targets/linkage.
- Existing dependencies/adapter code and any relevant ADRs.
- Proposal §§6–8, 11–14, 17 and roadmap Phases 1 and 14, through
  [AGENTS.md](../../../AGENTS.md).

## Workflow

1. Define the smallest spike that resolves an uncertainty: operation, typed owned
   output, safety requirements, semantic checks, platform matrix, and measurable
   startup/first-use/repeated-call costs. Keep a spike distinct from production
   support; do not scaffold the entire proposed workspace.
2. Inspect upstream API/ABI, ownership and allocator rules, error model, global
   initialization, callback behavior, cancellation, thread guarantees, native build
   prerequisites, license, and redistribution obligations. Cite source/version;
   missing evidence is a risk, not permission to assume safety.
3. Design the boundary: safe TabDat capability → safe facade → minimal low-level
   adapter → C ABI. For C++ use a narrow C ABI shim where needed. Raw bindings may
   exist inside low-level layers but foreign pointers/model/data types must not
   appear in domain/application APIs. Return owned Rust results promptly.
4. Review every owned/borrowed resource: constructor, owner, lifetime, allocator,
   destructor, cleanup after partial failure, buffer shape/length/nullability,
   callback userdata, and thread access. Prefer bounded result copying over fragile
   foreign borrows; pursue zero-copy only with a proven ownership protocol and
   material benefit for large buffers.
5. Where unsafe is necessary, confine it to approved low-level crates, use
   `#![deny(unsafe_op_in_unsafe_fn)]`, and explain each block's actual preconditions
   in adjacent `// SAFETY:` comments. Use RAII; prevent panics from crossing the
   ABI. No speculative `Send`/`Sync`: require upstream guarantees or a safe
   serialized/thread-confined design. Safe facade and domain crates forbid unsafe.
6. Validate normal calls, invalid shapes/arguments, native errors, callback failure,
   repeated allocation/free, partial construction, and teardown. Use sanitizers,
   fuzzing, or leak checks where supported; state exactly what ran. Statistical
   spikes also require `tabdat-statistical-validation`, and DTA conversion requires
   `tabdat-data-semantics` for labels/missingness. Safety is not semantic parity.
7. Measure a release build, excluding compilation, on named hardware/OS with
   versions, commands, repeat counts, and raw results. Distinguish cold/warm startup,
   first capability initialization, repeated backend execution, conversion, rendering,
   and peak RSS. State cache conditions rather than asserting an unmeasured cold run.
   Compare a baseline; proposal latency numbers are aspirational, not release gates
   already achieved. Metadata-only paths must avoid unnecessary native initialization.
8. Evaluate build/distribution burden on macOS Apple Silicon and Linux x86_64,
   with other targets only when in scope. Review AGPL compatibility, linkage,
   required notices and corresponding-source obligations. Do not infer distribution
   readiness from one local link success or add a large overlapping dependency
   without evidence it eliminates more complexity than it introduces.
9. Produce an accept/reject/defer recommendation and, for an adopted major
   dependency/boundary, an ADR. Run root checks and any configured audit/deny/unsafe
   tooling; clearly label missing tools/configuration and untested platforms.

## Expected outputs and validation

Return a feasibility report with **contract and scope**, **API/version evidence**,
**ownership/safety ledger**, **semantic results**, **platform/build matrix**,
**performance measurements**, **license/distribution review**, and **decision and
remaining risks**. For each unsafe operation the ledger links the code location,
precondition evidence, safe wrapper, and test/review coverage.

Use the [shared artifact contract](../../../docs/harness/tabdat/team-spec.md) when
handoff is durable. Mark `partial` for untested targets or incomplete tooling.
Stop integration if ownership/ABI/thread guarantees cannot be established,
required semantics cannot be reproduced, licensing is unresolved, or the requested
core path would acquire a Python/R dependency. Preserve spike evidence and name
the decision or prerequisite needed; do not paper over uncertainty with unsafe.
