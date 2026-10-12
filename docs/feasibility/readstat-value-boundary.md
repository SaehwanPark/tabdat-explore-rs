# ReadStat pointer-only value boundary

Status: **contract declared; implementation and verification pending**. Base
`8981ac8`; this is bounded safety hardening, not Phase 1 acceptance or DTA
migration parity.

## Contract

The pinned ReadStat 1.0.0 `src/readstat.h:136–149` defines `readstat_value_t`
as a union, enum, tag and bitfields. Size/alignment measurements alone do not
establish aggregate calling conventions or the initialization of padding when
modeled as Rust `[u8; 16]`. No runtime ABI failure has been demonstrated; remove
that assumption rather than claiming the existing fixture proves it safe.

- C compiled against the pinned header receives native by-value callbacks and
  calls native by-value accessors. Rust receives only a borrowed opaque pointer
  to the C callback's stack value and copies owned scalar/string/missing results
  before returning. No value layout or native padding is interpreted by Rust.
- Raw foreign types, constants and declarations become private to the adapter.
  The owned safe facade and its public behavior remain unchanged.
- No global callback state, foreign pointer retention, dependency addition,
  numerical tolerance change or warning suppression is permitted.
- Existing patient/labeled fixtures, all numeric accessor types, string ownership,
  system/tagged missing and value-label callbacks must pass. Native callback
  abort and repeated successful/error parse recovery must be tested.
- Run adapter and facade checks on macOS arm64 and emulated Linux x86_64, plus
  root gates and the existing dependency/unsafe policy checks. Emulation is not
  a physical-host, sanitizer or leak certification.

## Deferred

Native parser source-quality warnings, arbitrary malformed-file robustness,
allocation failure, general callback dimensions and panic behavior, path encoding,
thread guarantees, relocation/licensing and broader DTA semantics remain separate
acceptance work. This change neither proves nor relaxes those gates.
