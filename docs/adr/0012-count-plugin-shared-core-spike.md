# ADR 0012: Opt-in shared libgretl core for the Poisson plugin spike

- Status: Accepted for bounded feasibility evaluation; not distribution approval
- Date: 2026-10-12
- Scope: `spikes/gretl-prototype`, classical integer-count Poisson

## Context

Pinned gretl 2026b `discrete.c::count_model` performs native OLS setup and calls
`get_plugin_function("count_data_estimate")`. `plugins.c` maps this entry point to
`poisson.so`, loads it lazily and caches its handle until native cleanup. The
previous static-core-only build supplies no plugin. Upstream modules link
libgretl; putting another static core inside a module could duplicate mutable
process globals and violate ADR 0010's single serialized lifecycle.

## Decision

Add an explicit **`count-plugin` feature**, propagated to gretl-sys. Default
static OLS/binary builds remain static and do not build/load count modules.
The opt-in path builds one shared libgretl core and only `poisson.la` from the
same SHA-256-verified source, including cached archives. Reject the unverified
prebuilt shortcut for this capability. Use the upstream `count_model` and plugin
dispatch; do not duplicate optimizer/setup code in the shim.

Configure a build-owned absolute shared-library directory, pass it to executable
and test linkers through Cargo `links` metadata, and embed the private plugin
directory in the C shim. Compile-time bounds account for the native loader's
path+basename+suffix buffer. Initialization sets the path once while holding the
existing native mutex. There is no public arbitrary path setter, raw module
handle, cleanup or concurrent plugin mutation API. Model/dataset ownership,
checked indices and serialized destructors remain unchanged. Loading/cached
handles have process lifetime; no plugin unload can invalidate live objects.

For this pinned generated configure script, shared builds require canonical host
detection. It is incorrectly conditional on the gettext header/NLS path: disabling
NLS (or missing keg-only libintl.h) leaves host_os blank and libtool declines
shared libraries. The opt-in build therefore enables NLS and exposes Homebrew
gettext headers/tools/library path. It also creates plugin `.deps` when building
a single module, bypassing the aggregate target. These are documented native
build prerequisites, not statistical algorithm patches. Default static linkage is
unchanged; both source routes explicitly disable auto-detected libR integration
so external R validation cannot become a runtime dependency. Relocatable packaging and Linux revalidation remain deferred.

## Consequences and limits

- Counts are exact nonnegative integers; the pinned Python oracle accepts
  nonnegative fractions. The facade rejects fractional outcomes explicitly.
- Classical finite complete-data Poisson is the only newly validated family.
  Robust/cluster, offsets/exposure, missing samples, prediction/model state and
  negative-binomial semantics are not accepted by this decision.
- Paths are intentionally build-specific. This is not a relocatable application
  bundle, codesigning/notarization strategy or native redistributable package.
- Core and Poisson source are GPL-3.0-or-later, compatible with the proposal's
  AGPLv3 intent through GPLv3 §13. Shared linkage does not remove source/notice
  obligations. NLS/gettext and all native dependencies still need full release
  license/NOTICE/source and redistribution review; Cargo policy cannot scan them.
- The feature does not add Python/R or new Cargo packages. Existing safe facade
  forbids unsafe; the single approved low-level adapter contains the new FFI call
  with adjacent reasoning. Inventory is not sanitizer/transitive safety proof.

## Evidence

[Poisson report](../feasibility/poisson-plugin.md),
[build boundary](../../spikes/gretl-prototype/gretl-sys/build.rs),
[owned facade](../../spikes/gretl-prototype/src/poisson.rs),
[reference/error/thread tests](../../spikes/gretl-prototype/tests/poisson_fixture.rs).
