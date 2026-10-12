# Classical Gaussian Tobit feasibility

[PR #174](https://github.com/SaehwanPark/tabdat-explore-rs/pull/174), base
`720902b`; [ADR 0014](../adr/0014-tobit-observed-information-and-plugin-scope.md).
**Pass for bounded observed-information feasibility**, not product/fallback-SE
parity or primary-backend acceptance.

## Contract and reference provenance

`run_tobit` takes owned `EstimationProblem`, finite lower limit and optional upper
limit. Data are rectangular finite complete outcome/predictors; optional intercept
is first. Exact input sample is retained. Outcomes at or beyond bounds count as
censored. At least one uncensored observation and full-rank design are required.
An absent upper bound maps internally to native 1e300; the facade rejects outcomes
or lower limits colliding with that sentinel rather than silently censor them.

`TobitResult` owns beta coefficients/SEs, **joint beta/log(sigma) observed covariance**
(scale last), log(sigma), likelihood and exact sample rows. Robust/cluster,
missing-row deletion, general interval regression, prediction/model/session
state, p-values/DF and terminal/JSON/MCP are not validated by this spike.

Shared fixture: NumPy seed 9017, 240 rows, intercept/two Gaussian predictors,
Gaussian latent outcome. Both left-censored at zero and doubly censored at zero/two
are saved. SHA-256:
`08831bffcedc0fd704fa95da3206751bce64922f02a3cdc821348848f0c1f5cd`.
Pinned Python identity remains the [migration record](../migration/README.md);
checkout/lock were not altered. NumPy/pandas/statsmodels versions remain the
previous record; SciPy 1.17.1. Independent R 4.3.2 / survival 3.5-7 uses Gaussian
`survreg(Surv(..., type='interval2'))`, rel.tolerance 1e-12 and maxiter 100.
Native is source-pinned gretl 2026b's `interval.c` / `tobit_via_intreg`, default
optimizer with analytic observed inverse Hessian. No native tolerance is changed.

Predeclared MLE comparison: abs **1e-7 + rel 1e-6*abs(reference)**, exact names,
shape and sample. Compare beta/SEs, all 16 joint covariance entries, log(sigma)
and likelihood. Maximum absolute differences across exposed fields on macOS:

| Reference | Left-only | Two-sided |
| --- | ---: | ---: |
| independent R observed information | 1.541e-10 | 1.606e-11 |
| separately labeled Python observed Hessian | 1.959e-8 | 9.144e-9 |

### Recorded oracle discrepancy, not hidden parity

The actual executor prefers R but falls back to Python here: rpy2-rinterface 3.6.6
API binary references missing R 4.5 libraries, while ABI mode requires `R_getVar`,
absent in installed R 4.3.2. Both actual command invocations used Python fallback;
the fixture script spies on fallback entry and records it. No environment repair
or reference substitution is described as unmodified executor behavior.

Original beta values pass. Original fallback SEs **fail**: it returns BFGS/L-BFGS-B
inverse-Hessian approximations instead of observed curvature. Maximum SE differences
are ~0.00050 left-only and ~0.00078 two-sided. Original output is preserved in
`tobit-original-python.csv`. ADR 0014 accepts correctly labeled observed covariance
for feasibility, deliberately not old-fallback SE parity. `tobit_observed.py` uses
the same fallback likelihood, zero start and optimizer settings, replacing only
Hessian extraction with symmetric finite differences. This external normalized
reference is separately labeled; no tolerance was widened or data reselected.

## Build, ownership and verification

Opt-in `tobit-plugin` builds only `interval.so` against the same verified shared
core. `count-plugin` remains separately selectable; default remains static.
Internal shared-linkage feature centralizes the existing ADR 0012 lifecycle, SHA-
verified source and prebuilt prohibition. Both module features share one core,
private bounded build-owned path and process-lifetime serialized initialization;
there is no public loader-path setter or unload. Both builds exclude libR.
`otool -L` verifies interval and shim executable reference the exact same core.
Absolute paths are spike-only, not relocatable distribution acceptance.

Native `tobit_model` returns MODEL by value; shim retains ownership and existing
RAII frees it. `interval.c` donates full_vcv to the MODEL and cleans temporary
dataset series before return. Shim checks successful TOBIT family, matrix presence,
shape and indices before borrowing a scalar; **no foreign matrix/pointer escapes**.
Rust rejects failed/rank-dropped/changed-sample/nonfinite results before exposing
owned output. Invalid low-level limits/list/index arguments panic before FFI;
no panic crosses C. Calls/destructors retain the existing mutex boundary.

Tests cover reference values, exact labels/sample/full covariance shape, invalid
limits/data, sentinel collision, all-censored rejection, constant/collinear failures,
recovery, no-intercept smoke, beyond-bound censoring, failed/out-of-range covariance
buffers, wrong-family matrix access and four-thread fits. Fresh-process missing
actual linked interval.so returns owned native error; the module is restored and
reference fits pass in a fresh process. The probe derives the path from the test
executable's actual linked core, not the first cached build directory. Tobit-only
build is explicitly checked not to contain poisson.so.

macOS default 21, count-only 26, Tobit-only 26 and combined 31 tests pass; ordinary
missing-plugin probes are ignored and separately exercised manually. Fresh Linux
x86_64 emulated source builds use the existing [container](linux-revalidation.md),
not physical-host timing evidence. Locked check/format/Clippy and root full
baseline pass. Strict audit has zero vulnerabilities/warnings and unchanged deny
policy passes; no Cargo dependency/waiver added. Full geiger inventory records
62 used unsafe expressions in the adapter; nested facade inventory repeats these
62 (known attribution issue) while compiler forbid-unsafe remains enforced.
Reports are retained unfiltered, not native/sanitizer/leak certification.

Three owner review passes, not independent reviews: contract/reference authority;
FFI ABI/ownership/family/shape; failure/feature/platform scope. Initial compile probe
caught misuse of the public owned matrix extractor; shim now correctly borrows
MODEL-donated data. Follow-up adds wrong-family and no-upper sentinel guards.
All changed paths are reverified. No unresolved defect in the bounded slice.

## Release measurements and distribution limitations

macOS 27.0.1 arm64 M1 Max, Rust 1.97.1, five fresh release processes; each performs
explicit initialization, first owned 240-row left-Tobit fit and 1,000 repeated
owned fits. Fixture parsing precedes timers. Fit includes dataset/list creation,
plugin load on first use, estimation, owned copying and RAII; it is not pure model
arithmetic. Compilation is excluded; OS/disk caches are not evicted.

Medians: native init **18.066 ms**; first owned Tobit **1.077 ms**; repeated owned
Tobit **197.342 µs**; peak RSS **9,715,712 bytes (~9.27 MiB)**. First fit raw values
48.385, 1.077, 1.000, 1.130, 1.014 ms: do not hide the initial load/cache outlier
or call the median fully cold. Init still includes the recorded missing-gretlcli
`otool exited with status 256` diagnostic/subprocess. Executable 516,128 bytes,
linked core 3,379,608 bytes, interval module 54,920 bytes; these exclude the many
system/transitive artifacts required for redistribution.

```sh
CARGO_TARGET_DIR=target/gretl-review cargo build --release --locked --manifest-path spikes/gretl-prototype/Cargo.toml --features tobit-plugin --bin tobit-measure
/usr/bin/time -l target/gretl-review/release/tobit-measure
```

References run separately from runtime:

```sh
# From pinned Python checkout; --generate-data is optional, not required to test.
uv run python ../tabdat-explore-rs/spikes/gretl-prototype/fixtures/tobit_python.py
uv run python ../tabdat-explore-rs/spikes/gretl-prototype/fixtures/tobit_observed.py
# From Rust repository:
Rscript spikes/gretl-prototype/fixtures/tobit_reference.R spikes/gretl-prototype/fixtures/tobit.csv
CARGO_TARGET_DIR=target/gretl-review cargo test --locked --manifest-path spikes/gretl-prototype/Cargo.toml --all-features --all-targets
```

Interval code shares gretl's GPL source/licensing context; compatibility with AGPL
intent does not satisfy corresponding-source, dependency/font notices or packaging
obligations. Comprehensive native/transitive safety, sanitizers/leaks, relocation,
other architectures and final statistical-backend acceptance remain open.
