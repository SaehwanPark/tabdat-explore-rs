# ReadStat pointer-only value boundary

Status: **bounded hardening complete; wider acceptance deferred**. Base
`8981ac8`; [PR #182](https://github.com/SaehwanPark/tabdat-explore-rs/pull/182).
This is not Phase 1 acceptance or DTA migration parity.

## Contract and evidence

The pinned ReadStat 1.0.0 `src/readstat.h:136–149` defines `readstat_value_t`
as a union, enum, tag and bitfields. Size/alignment measurements alone do not
establish aggregate calling conventions or initialization of padding when
modeled as Rust `[u8; 16]`. No runtime ABI failure was demonstrated; this change
removes that assumption rather than claiming the existing fixture proved it safe.

- `readstat-sys/value_bridge.c`, compiled against the extracted pinned header,
  receives native by-value callbacks and calls native by-value accessors. Rust
  receives only a borrowed opaque pointer to the C callback's stack value and
  copies owned scalar/string/missing results before returning. Rust neither
  instantiates nor interprets the native aggregate, union or padding.
- Raw foreign types, constants and declarations are private to the adapter.
  Exported, uniquely named C callback symbols are bridge implementation details,
  not public Rust APIs. The owned safe facade remains unchanged.
- No global callback state, foreign pointer retention, dependency addition,
  numerical tolerance change or new warning suppression. The bridge itself
  compiles with C99 `-Wall -Wextra -Werror` on both evaluated platforms.
- Handler registration now preserves the first nonzero error instead of combining
  error codes with bitwise AND. Native setters currently return success; failure
  injection for registration/allocation is not claimed.

## Ownership and safety ledger

| Location | Preconditions and lifetime | Coverage / limitations |
| --- | --- | --- |
| C `value_callback` / `label_callback` | Native by-value argument lives on the C callback stack; Rust receives its address synchronously; C forwards the handler result | Actual patients/labeled DTA values and numeric/string label sets; no address retention |
| Rust `extract_value` / pointer accessor declarations | Pointer comes from live C callback/test-driver value; optional variable comes from native callback; accessors use the pinned C header | All five numeric scalar types, string, system/tagged missing; owned text survives driver mutation/return; defined-missing SPSS branch not validated |
| Rust exported callbacks | C ABI signatures match bridge declarations; parse-owned context remains live; `catch_unwind` body copies data | Existing ingestion fixtures; arbitrary panic/OOM containment is not certified |
| `parse_dta_with_metadata_handler` | Private injection seam; callbacks synchronous; parser freed after success, native error or abort | 20 real native metadata-handler aborts (exact code 4), each followed by successful labeled parse; no leak/sanitizer claim |
| Test corruption file | Exclusive `create_new` grants cleanup authority; `AlreadyExists` retries without adopting existing files | Read-only Linux checkout passes; guard removes only its owned file |

## Platform and policy checks

On 2026-10-12, macOS arm64 and **emulated** Linux x86_64 each pass **11 tests**
(3 adapter / 8 facade integration), locked workspace/all-targets check/test/Clippy,
and spike formatting. Linux is the existing `tabdat-phase1-linux` container,
not new physical-host or sanitizer evidence. No capability cost or performance
change is asserted.

Commands (from the repository root):

```sh
cargo fmt --manifest-path spikes/readstat-prototype/Cargo.toml --all -- --check
cargo check --locked --manifest-path spikes/readstat-prototype/Cargo.toml --workspace --all-targets
cargo test --locked --manifest-path spikes/readstat-prototype/Cargo.toml --workspace --all-targets
cargo clippy --locked --manifest-path spikes/readstat-prototype/Cargo.toml --workspace --all-targets -- -D warnings
cargo audit --file spikes/readstat-prototype/Cargo.lock -D warnings
cargo deny --manifest-path spikes/readstat-prototype/Cargo.toml check
```

Root locked fmt/check/test/Clippy, strict audit/deny, and all four first-party
unsafe-policy checks also pass. Full locked/all-target nested spike geiger reports
**79 used unsafe expressions** in the adapter and the same dependency-attributed
count in the facade, whose compiler `forbid(unsafe_code)` remains true. Inventory
is not native safety certification. No dependency or lockfile changes; the local
path dependency now explicitly pins `=0.0.0` rather than violating wildcard policy.

Recorded failures and replay:

- Initial Linux run: 3 adapter / 6 integration passed, corruption test failed on
  read-only checkout. The test now uses an exclusively owned temporary file;
  complete Linux replay passes.
- Initial strict Clippy found the existing unnecessary lazy metadata fallback;
  replaced with `unwrap_or`, without diagnostic suppression; replay passes.
- Spike deny found the existing wildcard path dependency; exact pin fixes it
  without policy changes; replay passes.
- First geiger invocation required an absolute manifest path. The first root
  inventory batch timed out; all four first-party reports were replayed and
  validated, followed by the full nested spike report.

Logs are local observations under `/tmp/readstat-bridge-*`; the commands and
bounded results above are the durable evidence, not a promise of preserved logs.

## Owner review and decision

Three explicit **owner**, not independent, passes covered (1) header/bridge scalar
and callback ABI, (2) stack-value/context ownership, native abort/recovery and
error handling, and (3) scope, fixture ownership and policy. Follow-up replay
passes with no unresolved blocker in this bounded contract.
[ADR 0020](../adr/0020-readstat-pointer-only-value-boundary.md) accepts only the
pointer-only bridge for continued spike evaluation.

## Deferred

Native parser source-quality warnings, arbitrary malformed-file robustness,
allocation failure, general callback dimensions and panic behavior, path encoding,
thread guarantees, archive verification in the Cargo build script, compiler-wrapper
`CC` strings/cross-compilation, relocation/licensing and broader DTA semantics
remain separate acceptance work. No sanitizer/leak testing ran. This change
neither proves nor relaxes those gates.
