# Classical Poisson / shared-core plugin feasibility

**Disposition: bounded macOS spike passes; not product or distribution support.**
2026-10-12, Rust source on [PR #170](https://github.com/SaehwanPark/tabdat-explore-rs/pull/170)
based on `f4a23c2`,
[ADR 0012](../adr/0012-count-plugin-shared-core-spike.md). Phase 1 remains partial.

## Python contract

Clean pinned `../tabdat-explore`, commit
`16b45d9b66b0d80f32d4d220e84d81bc5180bdbe`, tree
`601b236788872323af9277d2276a236154a0f129`, uv.lock SHA-256
`0f0e1dedbff49b4c77b470a75510b8809436c7c3dc77d1eac372ab9c7264d239`;
checkout clean before generating references. `executor.py::_execute_poisson`
uses statsmodels Poisson, intercept-first design, classical covariance by default,
complete-row selection, normal asymptotic inference and owned result fields.
It also supports robust/cluster and stores prediction state; those are not ported.
Python accepts **nonnegative fractional outcomes**, whereas Gretl's count check
requires integers. This slice rejects fractions explicitly; no whole-command
parity or silent behavior choice is claimed.

## Rust contract / mapping

Opt-in `count-plugin` feature only. `run_poisson` returns owned coefficient names,
values/SEs, full covariance, likelihood and exact zero-based source rows. Finite,
complete, rectangular input; exact nonnegative integer outcomes up to 2^53;
intercept + predictors, no offset, weights, robust/cluster, automatic missing
sample construction, prediction or product session/model mutation. Bad shapes,
nonfinite/negative/fractional/oversized counts and NUL names are facade errors;
native errors preserve `EstimationFailed { code }`. Rank/sample changes and
nonfinite outputs are rejected. No native handle or plugin path escapes.
No-intercept fitting is smoke-tested but the trusted fixture uses an intercept.

`count_model` in pinned `discrete.c` performs native OLS setup and dispatches
`count_data_estimate` from `poisson.so`. Native optimizers/covariance remain
upstream; the shim does not duplicate numerical code. The plugin supplies
Poisson information/Hessian covariance; canonical log-link observed/expected
information agree. No finite-sample robust correction is inferred from this.

Default OLS/binary builds stay static and do not include count APIs/modules.
The feature builds one **shared** core plus only the count module from the same
verified 2026b archive (SHA-256
`fb57f4922da546067c8be542aafc5a26be77faf2668b40e651ee8c90b702563d`).
Cached archives are reverified for this path; prebuilt shortcuts are rejected.
Absolute build-owned library/plugin paths are deliberately nonrelocatable.
Cargo metadata carries the core path to parent executable/test rpath flags.
`otool -L` confirms the fixture executable and actual loaded Poisson module
reference **the identical shared core**, not another static/independent copy.

## Test contract / references

Tolerances declared in the early draft checkpoint **before comparisons**:
`|actual-reference| <= 1e-7 + 1e-6*|reference|` for every coefficient, SE,
covariance entry and log likelihood. Finite values required; exact parameter
order/names, matrix dimensions and source rows 0..239. No tolerances widened.

[Data](../../spikes/gretl-prototype/fixtures/poisson.csv): 240 synthetic rows,
NumPy RNG seed 7193; two standard-normal predictors and
`Poisson(exp(0.3 + 0.4*x1 - 0.25*x2))` outcomes. Data SHA-256:
`2ecc445b1bf5d37546317f50dacdb04f0b0f31541f65ada17f1fd970b0d631e5`.
External [Python probe](../../spikes/gretl-prototype/fixtures/poisson_python.py)
uses TabDat executor, checks convergence, exact input/design arrays and names;
Python 3.13.3 / NumPy 2.4.4 / pandas 3.0.3 / statsmodels 0.14.6. Independent
[R probe](../../spikes/gretl-prototype/fixtures/poisson_reference.R): R 4.3.2 base
`glm(family=poisson(link="log"))`, epsilon 1e-12, maxit 100; verifies convergence,
ordered model-frame rows and information covariance against `vcov` at 1e-10.
R is validation tooling, never a Rust runtime dependency.

The [reference CSV](../../spikes/gretl-prototype/fixtures/poisson-reference.csv)
contains independently generated Python/R numbers, not native-produced goldens.
Native coefficients: `[0.3794753907401761, 0.3385418558659754, -0.23503284119795598]`;
SEs `[0.055445202670848445, 0.0477358419144867, 0.04903143690006355]`;
log likelihood `-366.231211569813`.

| Reference | Maximum coefficient abs difference | Maximum SE abs difference | Maximum covariance-entry abs difference | Likelihood abs difference |
| --- | ---: | ---: | ---: | ---: |
| Python | 2.182e-10 | 7.066e-12 | 6.747e-13 | 0 |
| R | 2.182e-10 | 7.066e-12 | 6.747e-13 | 0 |

All quantities pass, as do exact sample/names. Feature suite: **17 passing tests**
(existing OLS/binary/boundary/covariance/measurement cases plus three Poisson
cases), with one deliberately ignored manual loader probe. Poisson failure tests
cover invalid counts/data/names/shapes, undersized sample, collinearity/all-zero
native errors and successful recovery. Four threads perform mixed independent
Poisson/OLS/logit calls repeatedly under the same serialized native core.

The ignored loader probe was run separately in a fresh test process after
**temporarily renaming the executable's actual build-owned** `poisson.so`.
It returned an owned native error, not a crash. After restoration, another fresh
process passed the numerical fixture. Choose the module from the executable's
linked core path: a first probe selected a different cached build and correctly
failed its expected-error assertion; it is not acceptance evidence. No shared
plugin file is modified by ordinary concurrent tests. `DYLD_PRINT_LIBRARIES`
metadata-mode inspection loads the linked core but **not** the lazy count module;
metadata mode does not call native initialization.

## Ownership / safety / licenses

| Boundary | Invariant / ownership | Evidence / remaining gap |
| --- | --- | --- |
| Source/build | One pinned verified archive, shared core + module, matching headers; no arbitrary prebuilt core | From-source build, feature rejection of prebuilt shortcut; Linux unvalidated |
| Plugin path/load | Compile-time length bound includes separator/basename/suffix; private initialization under native mutex | C `_Static_assert`, matching linkage and manual missing-module test; no relocatable packaging |
| Plugin lifetime | Upstream cached handle retained for process life; no public cleanup/unload | ADR 0010 remains in force; no callback/foreign ownership leaks |
| Dataset/model | Checked C dimensions/list indices, private pointers, RAII; all calls/destructors serialized | Existing boundary tests and mixed-thread tests; no sanitizer/leak proof |
| Owned results | Copy only successful correctly shaped finite native output; facade forbids unsafe | Python/R comparisons and failed-fit recovery |

All added Rust unsafe is the adjacent-reasoned `GretlModel::poisson` call inside
approved gretl-sys; `unsafe_op_in_unsafe_fn` stays denied. The facade, executable,
new parent build script and tests forbid unsafe. Full geiger inventory (all
features/targets/dependencies/tests) reports **46 used unsafe expressions** in
the adapter. In the repository's nested layout, geiger 0.13.0 also attributes the
same 46 to the parent facade despite compiler-forbid enforcement. A source-byte-
identical temporary sibling layout (only dependency path changed) yields adapter
46, facade **0**, facade forbid true. Both full reports are retained; this is an
attribution trap, not an advisory waiver or reduced `--forbid-only` scan. C and
upstream native dependencies are outside this Rust inventory; their comprehensive
safety review remains open.

Core and Poisson source: GPL-3.0-or-later. GPLv3 §13 permits combination under the
proposal's AGPLv3 intent, but corresponding-source/notice obligations remain,
including the compatibility patch. Dynamic linking does not avoid them. NLS uses
GNU gettext/libintl; its native licensing/redistribution and all existing native
BLAS/OpenMP/GLib/XML/FFTW/GMP/MPFR/curl dependency obligations still require release
review. Cargo audit/deny scan the two Rust packages **only**, not these C libraries
or system tools. No native/font artifact is committed or distribution approved.

## Build findings / platform / verification

macOS 27.0.1 arm64 / M1 Max / Rust 1.97.1 only; no Linux, Windows, sanitizer,
Valgrind or codesigning/relocation run. Shared build initially produced only a
static archive: generated configure's host canonicalization is nested under the
gettext/NLS/header branch. Enabling NLS alone was insufficient with keg-only
Homebrew gettext. Supplying its headers/library/tool path resolved canonical host
and shared linking. Single-module make additionally requires `plugin/.deps`.
Cargo metadata must use `cargo::metadata=KEY=VALUE`; legacy `cargo:metadata=...`
produced the wrong environment key and was corrected before tests passed.

Native initialization prints `otool exited with status 256`: source
`gretl_utils.c::detect_blas_via_ldd` inspects an unbuilt `gretlcli` to detect BLAS;
on Apple it retains its VECLIB fallback after failure. Fits/reference checks pass.
This subprocess/diagnostic behavior is **not** product CLI acceptance and needs
normalization before selecting/shipping this backend. Metadata avoids that init.

Default static **from-source** locked check/test/Clippy pass (14 tests).
An initial prebuilt-check run passed check/tests but Clippy could not find its
cached prebuilt path; the full source rerun completed without that shortcut.
Feature locked fmt/check/test/Clippy pass, plus the manual loader/restoration test,
linkage/metadata inspection, full/sibling-layout unsafe inventories, audit (0
vulnerabilities/warnings) and existing deny policy. A pre-existing wildcard path
requirement in this isolated spike was replaced by exact `=0.0.0`, without
changing dependency resolution or policy. Root source/dependencies remain
unchanged; baseline checks and four-package unsafe loop passed on base #169.
Changed-document links and whitespace are checked before merge. Actions disabled.

Three explicit owner review passes: numerical/sample contracts; FFI/plugin
lifecycle and build provenance; policy/packaging/documentation. Fixed cached-source
verification, exact local dependency requirement and loader-probe target selection;
follow-up found no unresolved blocker in this bounded scope. No independent
reviewer runtime was available; these are not independent reviews.

Reproduce core checks and external fixtures:

```sh
PYTHONPATH=../tabdat-explore/src ../tabdat-explore/.venv/bin/python \
  spikes/gretl-prototype/fixtures/poisson_python.py
Rscript spikes/gretl-prototype/fixtures/poisson_reference.R \
  spikes/gretl-prototype/fixtures/poisson.csv
cargo fmt --manifest-path spikes/gretl-prototype/Cargo.toml -- --check
CARGO_TARGET_DIR=target/gretl-review cargo check --locked \
  --manifest-path spikes/gretl-prototype/Cargo.toml --features count-plugin --all-targets
CARGO_TARGET_DIR=target/gretl-review cargo test --locked \
  --manifest-path spikes/gretl-prototype/Cargo.toml --features count-plugin --all-targets
CARGO_TARGET_DIR=target/gretl-review cargo clippy --locked \
  --manifest-path spikes/gretl-prototype/Cargo.toml --features count-plugin --all-targets -- -D warnings
cargo audit --file spikes/gretl-prototype/Cargo.lock -D warnings
cargo deny --manifest-path spikes/gretl-prototype/Cargo.toml --all-features \
  --locked --config deny.toml check
```

Repeat Cargo checks without `--features count-plugin` for the default static
boundary. Do not use `GRETL_PREBUILT_DIR` for feature acceptance. Continue NB
parameterization/reference work and remaining estimator/platform/native safety
and distribution/backend decisions. Passing one module does not accept all plugins.
