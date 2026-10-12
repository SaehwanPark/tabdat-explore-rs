# Source-build native initialization discovery

[PR #179](https://github.com/SaehwanPark/tabdat-explore-rs/pull/179), base
`29190a2`; [ADR 0019](../adr/0019-native-blas-discovery-source-patch.md).
**Pass for bounded truthful source-build discovery**, not pure/no-I/O initialization,
external prebuilt/custom BLAS acceptance or final-backend/distribution approval.

## Source diagnosis and correction

Pinned gretl 2026b dynamically discovers OpenBLAS/MKL/BLIS then falls back. macOS
probed unbuilt gretlcli with otool; Linux probed GRETL_PREFIX/lib/libgretl-1.0.so
unless GRETL_LIB_NAME. Earlier Linux reports used missing-gretlcli shorthand too:
**Linux cause is missing installed-prefix library, not CLI**. Actual source output
lives under Cargo private build dirs. The math passed even when probes failed.

Mac source-controlled core and executable otool linkage confirms Accelerate.
Fallback reports veclib directly; prior dynamic provider detection and ordinary
num_threads_init remain. Do not use tool mode, suppress stderr or change kernels.
Linux resolves embedding executable via parent GLib readlink /proc/self/exe then
passes owned resolved path to ldd: literal /proc/self/exe in child would identify
ldd/interpreter. Existing explicit GRETL_LIB_NAME override is copied/preserved;
no process-wide environment mutation after threads exist. Other Unix targets
retain owned installed-prefix fallback without new validation. Failed discovery
stays unknown and emits genuine errors, not fake known-provider metadata.

Source-only replacement snippet retained in gretl-sys/patches with original GPL
context. Apply after prebuilt fast return and fail closed on expected function
boundaries; every cached/downloaded archive now verified against original hash,
including static default. Same source/header/ABI and module/core lifecycle.
External prebuilt/custom BLAS routes are unchanged and **excluded** from this new
clean-initialization acceptance. The source-route diagnostic tests are not an
approval of arbitrary GRETL_PREBUILT_DIR artifacts.

## Owned diagnostics and verification

`blas_variant_for_measurement()` initializes through same once-only serialized
boundary, copies static ASCII diagnostic into bounded owned buffer, returns owned
String, never exposes native string/ownership. `init-probe` checks first/repeated
or four concurrent initialization calls followed by known OLS intercept1/slope2.

Fresh serial/concurrent source processes have exactly one expected stdout line,
empty stderr, normal fits; macOS veclib and Linux netlib are honest provider values.
Linux child-scoped PATH wrapper proves ldd received resolved embedding executable,
not installed-prefix library or child /proc/self/exe. Bad explicit override yields
unknown plus genuine ldd error while OLS still fits; dynamically detected vendors
can legitimately skip fallback. No stderr redirection in implementation.

macOS **51 all-feature / 22 default** tests, Linux emulated x86_64 **53 all-feature /
24 default** tests pass (Linux adds target and bad-override probes). Numerical
fixtures unchanged, including seven-way plugins and sample/model buffers. Root
locked fmt/check/test/Clippy, spike/adapter fmt/check/Clippy, strict audit zero
vulnerabilities/warnings and unchanged deny pass. Full inventory retains **93 used
adapter unsafe expressions**, known nested-facade attribution93/compiler forbid,
not C safety certification. No Cargo dependency/waiver/runtime Python/R.

Root verification exposed another unchanged reshape tempfile failure at line301:
DuckDB could not open generated reshape_wide_collision.parquet. Focused12 tests
and final complete root rerun pass; intermediate full rerun batch timed out and was
not counted as pass. Recorded in [issue167](https://github.com/SaehwanPark/tabdat-explore-rs/issues/167).
Timestamp/create_dir_all ownership hypothesis remains unproven; separate fixture
hardening/investigation follows, not sleeps/tolerance retries in product code.

Three owner reviews, not independent: provider/linkage/no tool-mode/kernel change;
source/path/string ownership and fresh-process/concurrency/error fallback; numerical/
platform/policy/side-effect evidence. All follow-up checks pass; broader native
initialization/library effects are explicit, not hidden by clean stderr.

## Measurements and side-effect limits

macOS 27.0.1 arm64 M1 Max / Rust1.97.1, five fresh release init-probe processes,
all-feature shared-source build, no OS-cache eviction. Normal init then once-only
repeated call; parsing/compilation excluded. Init raw ms **3.775,1.609,1.728,1.798,
1.778**, median **1.778 ms**, repeated median **84 ns**. Historical previous five-
process all-feature initialization median was **19.485 ms** in dynamic-panel report;
this is an uncontrolled comparison, not a cold-start benchmark or numerical cost.
Original outlier/cost remains historical, not overwritten. `/tmp/native-init-measure.log`.

Linux initialization **still may spawn ldd**. PATH wrapper proves its argument,
not complete process/file I/O. Installed strace only as external container validation
tool; emulated trace decoded nonsensical syscall arguments/results and is unusable.
QEMU_STRACE produced no trace; neither proves no file writes or pure initialization.
No macOS syscall trace/physical Linux host trace/sanitizer/leak acceptance. libset,
random/XML/library/thread initialization effects still require comprehensive review.

```sh
CARGO_TARGET_DIR=target/gretl-review cargo test --locked --manifest-path spikes/gretl-prototype/Cargo.toml --all-features --all-targets
CARGO_TARGET_DIR=target/gretl-review cargo build --release --locked --manifest-path spikes/gretl-prototype/Cargo.toml --all-features --bin init-probe
/usr/bin/time -l target/gretl-review/release/init-probe
# --parallel validates concurrent once-only initialization
```

Corresponding source/patches/NOTICE/licensing/native redistribution, relocation,
other architectures, comprehensive integer/allocation/ownership/sanitizer/leak,
final backend choice and Phase1 exit review remain open. No phase completion.
