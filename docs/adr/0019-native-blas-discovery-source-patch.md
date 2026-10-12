# ADR 0019: Truthful source-build BLAS discovery

- Status: Accepted for bounded source-build feasibility
- Date: 2026-10-12

## Cause and decision

Pinned gretl 2026b dynamic OpenBLAS/MKL/BLIS detection precedes a fallback. macOS
fallback invokes otool on nonexistent gretlcli; Linux fallback invokes ldd on
GRETL_PREFIX/lib/libgretl-1.0.so unless explicit GRETL_LIB_NAME. Earlier reports
used missing-gretlcli shorthand for Linux too; source inspection corrects it.
Source outputs live in private Cargo directories, not installed-prefix locations.

For the source-controlled macOS route verified core/executable linkage uses
Accelerate. Fallback reports veclib directly; retain prior dynamic provider
recognition and normal num_threads_init. Do not use tool mode, suppress stderr,
change mathematics or assume every arbitrary external library uses Accelerate.
Prebuilt/external/custom BLAS routes are excluded from this new acceptance.

On Linux resolve embedding executable in parent via GLib readlink /proc/self/exe,
then pass allocated resolved path to ldd. A literal /proc/self/exe in the child
would name ldd/interpreter, not the caller. Preserve upstream explicit diagnostic
GRETL_LIB_NAME override by copying its string, never changing process-wide env.
Other Unix targets retain owned installed-prefix fallback, without validation.
Failed discovery remains unknown with diagnostics; never fabricate known provider.
Free owned path and output/error strings on every discovery exit.

## Source/ownership discipline

Retain source-only replacement snippet under gretl-sys/patches with GPL context,
apply only after external-prebuilt return and fail closed on expected function
boundaries. Verify cached and downloaded archive for every source route, not only
shared plugins. Same pinned source/checksum/headers/ABI and existing core/module
ownership remain. Corresponding-source/NOTICE/license redistribution is still
unaccepted; this patch must be included in any later distribution decision.

Owned bounded BLAS diagnostic copy gives measurement/tests a string, never native
string ownership. Fresh serial/concurrent process initialization plus OLS and
all numerical fixtures verify unchanged fits and clean normal streams. Linux
child-scoped PATH wrapper proves actual fallback argument, and explicit bad override
checks unknown/error behavior. No mutation of global environment in a running
Rust process; no public native loader/unload setter.

Linux may still spawn ldd, and initialization does other native/library I/O.
Emulated strace produced nonsensical decoded syscalls; QEMU_STRACE produced no
trace. Neither establishes file-write or no-subprocess/purity claims. Keep broader
physical-host/system-call/sanitizer/leak evidence open.

[Report](../feasibility/native-initialization.md),
[source snippet](../../spikes/gretl-prototype/gretl-sys/patches/blas-discovery.c).
