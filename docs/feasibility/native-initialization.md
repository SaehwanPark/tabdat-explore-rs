# Native initialization probe contract (in progress)

Base29190a2, after all initial-family bounded fixtures. Remove false BLAS-discovery
startup errors without suppressing stderr, using tool mode, changing numerical
kernel/covariance, or pretending unknown BLAS is known.

Source inspection corrects prior shorthand: macOS fallback probes unbuilt gretlcli
with otool, but Linux probes GRETL_PREFIX/lib/libgretl-1.0.so (unless GRETL_LIB_NAME),
not gretlcli. Actual source-built libraries/executables live in Cargo private
output directories. Ordinary fits pass despite failed probes; no whole-phase gate
is accepted from them.

Bounded plan: verified-source compatibility patch only. For source-controlled
macOS Accelerate linkage use known-provider initialization rather than a fictitious
CLI probe; confirm actual linkage before acceptance. For Linux resolve embedding
executable path in parent before ldd (literal /proc/self/exe passed to subprocess
would resolve the child!), preserving dynamic provider detection and ownership of
allocated path. Do not mutate process-wide environment after threads can exist.
Prebuilt/external BLAS routes excluded unless separately validated.

Fresh-process tests observe owned BLAS variant, clean stdout/stderr where expected,
first/repeated initialization and baseline OLS/whole feature fixtures; macOS and
emulated Linux source routes, check/Clippy/policy and unsafe inventory. Measure
init changes explicitly; prior costs remain historical. Trace residual file/process
side effects rather than claim pure/no-subprocess init: Linux fallback may still
spawn ldd on actual executable. Record source patch/corresponding-source/native
redistribution context and failure fallback, no audit waiver or runtime Python/R.
Native integer/allocation/leak/sanitizer/transitive ownership and final-backend
acceptance remain separate work. No implementation/verification claim yet.
