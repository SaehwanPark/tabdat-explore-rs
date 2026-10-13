---
name: tabdat-native-backends
description: Review native dependency, FFI ownership, adapter, licensing, packaging and platform risks for concrete TabDat capabilities.
---

# Native backends and safety

Use for native adapter changes or release integration; not for routine data or
parser work. Existing ReadStat/libgretl/plotting experiments are **prototypes**,
not installed product backends.

1. Justify the dependency with an actual supported feature, measured gain, or
   concrete limitation. Prefer an existing safe binding when sound.
2. Review named upstream version/API, ownership and allocator contracts,
   errors/callbacks, concurrency and cancellation assumptions, platform and
   redistribution obligations.
3. Keep foreign handles/pointers out of application/domain interfaces. Use safe
   facades, RAII and narrow documented unsafe blocks only where approved;
   ensure panics cannot unwind into foreign code.
4. Exercise creation, normal calls, invalid inputs, partial initialization,
   cleanup and repeated use. Run sanitizer/leak checks when risk warrants them.
5. For statistical adapters, separately validate estimator meaning against a
   trusted independent reference. Native safety does not establish statistical
   correctness.
6. Record the concise support boundary and any remaining installation/license
   gap. An isolated feasibility spike is not permission to claim production
   integration or portable packaging.

Do not build extra native crates for future-only roadmap features. See
[architecture](../../../ARCHITECTURE.md) and [AGENTS.md](../../../AGENTS.md).
