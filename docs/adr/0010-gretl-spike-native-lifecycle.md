# ADR 0010: Serialize and retain libgretl spike process state

- Status: accepted for the isolated spike only
- Date: 2026-10-11

## Context

The safe low-level spike API previously let callers index native buffers without
checks, narrow arbitrary Rust dimensions to C integers, initialize from the
facade only, and clean up global state while handles were live. Non-Send/Sync
handles alone do not prevent independent threads from entering libgretl.

## Decision

All native calls, including getters and destructors, take one process-global
mutex in `gretl-sys`. Initialization happens once under that mutex on first
native use. Remove public initialization/cleanup operations; retain library
process state until process exit. Owned datasets/models still free their own
allocations. This deliberately does not claim full native-state leak freedom.

Check dimensions, dataset indices, list counts and entries, successful-model
status, and coefficient indices before C access. Keep handles non-Send/Sync.
Invalid low-level prototype arguments panic before entering C; facade input
errors return `GretlError`. Reserve dataset variable zero for the constant even
when the estimation list omits it. Deny unsafe operations in unsafe functions.

## Consequences

Independent Rust callers can fit on different threads without concurrent entry
through this adapter. This is not upstream thread-safety certification, does not
coordinate unrelated libgretl consumers, and sacrifices parallel backend calls.
There are no callbacks, so Rust argument-check panics cannot unwind through C.
Any future callbacks, plugins, asynchronous cancellation, or cleanup feature
need a new lifetime/locking review. This does not accept libgretl for production.

## Evidence

Five boundary tests plus the existing three OLS tests pass locally, including
80 independent-thread fits, invalid-input rejection, failed-model buffer guards,
no-intercept scalar-formula comparison, and the existing Longley references.
See [the feasibility update](../feasibility/libgretl.md#boundary-hardening-2026-10-11).
