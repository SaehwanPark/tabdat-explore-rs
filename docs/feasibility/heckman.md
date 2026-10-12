# Heckman two-step selection feasibility

[PR #177](https://github.com/SaehwanPark/tabdat-explore-rs/pull/177), base
`8606026`; [ADR 0017](../adr/0017-heckman-two-step-covariance-and-cleanup.md).
**Pass for bounded proper two-step feasibility**, not original ordinary-OLS
outcome-SE parity, product heckman or primary-backend acceptance.

## Contract and reference authority

Complete finite two-equation sample, binary selection with both classes, explicit
intercepts, outcome predictors and excluded selection predictor. `HeckmanProblem`
owns names/data; rows are outcome, outcome predictors, selection, selection
predictors. Identically named shared predictors must contain identical values.
Result owns outcome beta plus `mills_lambda`, selection probit beta, separate
outcome/selection covariance blocks, sigma/rho and distinct exact sample IDs.
Unselected outcome values do not affect fit, but NULL/missing unselected outcomes
are deliberately not accepted by this complete-data slice. Native OPT_T, not MLE.

Predeclared abs **1e-7 + rel 1e-6*abs(reference)** for all coefficients/SEs/equation
covariance/sigma/rho; exact shape, names and both sample identities. Seed 44107,
480 total selection observations / **210 selected**; two outcome predictors, x1
and excluded z for selection. CSV SHA-256
`7bbca00f739b1f153dcc72497d8bc411b60055ff38e987f31b1c65b8147a9f06`.
Pinned checkout identity is in [migration record](../migration/README.md);
statsmodels 0.14.6 / SciPy 1.17.1, independent R 4.3.2 base probit/QR and explicitly
evaluated observed Hessian/two-step algebra. R is independent reference algebra,
not a claim of sampleSelection package coverage.

| Comparison | Maximum absolute difference |
| --- | ---: |
| native vs independently corrected external Python | 1.371e-11 |
| native vs independent R | 1.087e-8 |
| native proper two-step vs original ordinary-OLS outcome SE | 8.203e-4 |

Original public executor coefficients pass; ordinary-OLS second-stage SEs ignore
estimated first-stage Mills uncertainty. Preserve original values separately and
accept corrected inference explicitly under ADR 0017. Full outcome 4×4 and
selection 3×3 blocks plus SEs/sigma/rho pass without widening tolerance or modifying
pinned checkout. Selection uses observed probit information, not glm Fisher
covariance. Native composite matrix has zero cross-equation placeholders: expose
separate blocks only, never a purported full joint covariance or independence.
Sigma/rho are point diagnostics, not coefficient rows with unsupported SEs.

## Owned boundary, review and evidence

Independent `heckman-plugin` selects only same-source heckit.so under existing
verified shared-core/serialized/owned lifetime controls. Static default remains;
other modules are not implicit. No prebuilt shortcut/path setter/unload. Native
source excludes libR. MODEL and arrays/names copied before RAII destruction.

Checked encoded two-equation list, exactly one separator, dependent/constant/index
roles, finite complete names/data, same-name predictor agreement, sufficient sample,
rank/shape and owned failed-model handling. Bound combined parameter count before
FFI because upstream computes `(tot*tot+tot)/2` in signed int for packed VCV;
46,341 nominal parameters are rejected even if rank dropping could reduce them.
Facade returns owned invalid error and low-level malformed arguments panic before C.

Upstream early second-stage failure may leave temporary Mills series: heckit_init
returns error, and caller destroys its container before normal dataset cleanup.
Shim records original native variable count and removes added columns on success
and failure inside serialization. Scalar `variable_count()` exposes no ownership.
Direct probe first proves rare-event first-stage probit succeeds, then fails the
selected second stage, checks actual dataset remains seven variables, guards failed
VCV/metadata, restores selection and proves fit recovery on the same dataset.

Tests cover all references, exact masks/labels, invalid/ragged/nonfinite/names/
selection/constant/rank cases, unselected outcome invariance, recovery, low-level
malformed/oversized lists, failed metadata, parallel fits and six-way mixed
OLS/count/Tobit/quantile/survival/selection threading. Actual linked missing
heckit.so fresh-process probe returns owned error; module restored in finally,
new process fits references. Heckman-only directory excludes other modules.

macOS **46 combined / 26 Heckman-only / 21 default** tests and same emulated
[Linux x86_64](linux-revalidation.md) equivalents pass. Ordinary missing-loader
probes are ignored then separately exercised on macOS; no Linux missing-module or
physical-host performance claim. Root/spike locked fmt/check/test/Clippy, strict
audit (zero vulnerability/warning), unchanged deny and full inventory pass.
Adapter has **77 used unsafe expressions**, nested facade reports same known
attribution while compiler forbid remains enforced; retain full report, do not
call geiger native safety certification. No new Cargo dependency or waiver.

Three owner reviews, not independent: two-equation covariance/sample authority;
list/packed-arithmetic/model/error/dataset ownership; feature/platform/concurrency/
loader and evidence. Early-return mutation and signed packed-size hazard fixed
and follow-up boundary/combined/platform tests pass. Broader native integer/
allocation/sanitizer/leak and model variants remain separate acceptance work.

## Release cost and distribution

macOS 27.0.1 arm64 M1 Max / Rust 1.97.1, five fresh release processes, explicit
init followed by first owned 480-row fit and 1,000 repeats. Parsing precedes timers,
compilation excluded, OS caches not evicted. Includes datasets, first plugin load,
probit/selected regression, covariance/sample copy and RAII.

Medians: init **17.810 ms**, first fit **1.693 ms**, repeated **510.219 µs**, RSS
**10,223,616 bytes (~9.75 MiB)**. First-fit raw ms: 47.865, 1.552, 1.693, 1.565,
1.725; first-load outlier preserved, median not fully cold. Executable **554,672**,
core **3,379,608**, module **73,320 bytes**, excluding system/transitive artifacts.
Known missing-gretlcli otool/ldd diagnostic persists. GPL source/notice/module/native
redistribution and relocation approval remain open; no runtime R/Python.

```sh
# From pinned Python checkout:
uv run python ../tabdat-explore-rs/spikes/gretl-prototype/fixtures/heckman_python.py
uv run python ../tabdat-explore-rs/spikes/gretl-prototype/fixtures/heckman_corrected.py
# From Rust repository:
Rscript spikes/gretl-prototype/fixtures/heckman_reference.R spikes/gretl-prototype/fixtures/heckman.csv
CARGO_TARGET_DIR=target/gretl-review cargo test --locked --manifest-path spikes/gretl-prototype/Cargo.toml --all-features --all-targets
CARGO_TARGET_DIR=target/gretl-review cargo build --release --locked --manifest-path spikes/gretl-prototype/Cargo.toml --features heckman-plugin --bin heckman-measure
/usr/bin/time -l target/gretl-review/release/heckman-measure
```

No robust/cluster, MLE, missing-row deletion, no-intercept, correlation-boundary,
separation/extreme tails, prediction/session or product support. Comprehensive
native/transitive safety, licenses/distribution, other architectures and final
backend selection remain open; Phase 1 is partial.
