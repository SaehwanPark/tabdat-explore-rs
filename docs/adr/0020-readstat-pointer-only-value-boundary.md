# ADR 0020: Pointer-only ReadStat value boundary

- Status: accepted for bounded spike evaluation, not product ingestion
- Date: 2026-10-12
- Scope: ReadStat 1.0.0 aggregate callback/accessor ABI

## Context

The prototype modeled `readstat_value_t` as a 16-byte aligned Rust byte array.
The native header defines a union, enum, tag and bitfields. Observing matching
size/alignment and fixture outputs does not establish aggregate calling
conventions or initialized padding. No runtime ABI failure has been demonstrated.

## Decision

Compile a narrow C bridge against the pinned native header. Native by-value
callbacks and accessor calls remain wholly within C; Rust receives only a
borrowed opaque pointer, valid until the synchronous callback returns, and copies
owned values. Keep raw types/declarations private. Preserve the safe facade,
missingness and label semantics, panic-catching bodies and per-parse state;
introduce no callback globals or dependencies.

Uniquely named C-exported Rust callback symbols are private bridge implementation
details. Native registration errors retain their first nonzero code rather than
being bitwise-combined. No new suppression or policy waiver is authorized.

## Consequences and evidence

[The report](../feasibility/readstat-value-boundary.md) records 11 tests each on
macOS arm64 and emulated Linux x86_64, scalar/string/missing copies, label fixtures,
native metadata-handler abort/recovery and strict policy checks. Existing fixture
checkout writes, a lazy-fallback Clippy diagnostic and a wildcard path dependency
were fixed and complete checks replayed.

This accepts only the bridge for the spike. Native source-quality findings,
allocation/panic/dimension guarantees, malformed files, path/thread semantics,
source-archive verification, compiler wrappers/cross-compilation, licensing and
redistribution remain unresolved. No physical-host, sanitizer, leak or performance
acceptance is inferred.
