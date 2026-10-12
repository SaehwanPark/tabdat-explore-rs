# Exponential / Weibull PH survival feasibility

[PR #176](https://github.com/SaehwanPark/tabdat-explore-rs/pull/176), base
`c5e6efb`; [ADR 0016](../adr/0016-survival-ph-and-observed-information.md).
**Pass for bounded classical observed-information feasibility**, not original
approximate-SE parity, product streg or primary-backend acceptance.

## Contract, normalization and references

`SurvivalProblem` owns time/failure/predictor names and rows containing positive
finite time, binary failure (1 event / 0 right-censored), predictors and explicit
intercept. Native requires an intercept; no-intercept is rejected, not silently
changed. Complete finite full-rank sample only, at least one failure. Result owns
PH log-rate coefficients, Weibull log(shape), full joint observed covariance/SEs,
time-density likelihood and exact sample/failure/censored row identities.

Hazard is `exp(X gamma) * shape * t^(shape-1)`; exponential fixes shape=1. Native
duration uses AFT beta/raw sigma and 1=censored. Normalize exponential gamma=-beta;
Weibull gamma=-beta/sigma, log(shape)=-log(sigma). Sparse Jacobian has at most two
nonzero derivatives per parameter, transforming `J V J'` in O(k²), including all
cross terms. Shape is explicit, not mislabeled as a regression slope.

Native duration_loglik evaluates event density on **log(time)**. Initial comparison
failed by 163.860533595 for exponential; sum of event log times is -163.860533595
(and -52.787478957 for Weibull). Owned likelihood subtracts this event-only time-
density Jacobian; censored survivor terms are unchanged. It does not alter fits.
Time-unit metamorphic tests verify PH intercept/shape invariance and density
likelihood shift by failure count, not total N.

Fixture: seed 18019, 320 rows, Gaussian predictors, exponential / shape-1.6 Weibull
latent times and independent exponential censoring. Fixed CSV SHA-256
`e4c7be203406d314b2a17a89c12e1ed1dfe16bfb36ab00064b231ce1cb3155ef`.
Pinned Python identity is the [migration record](../migration/README.md), statsmodels
0.14.6 / SciPy 1.17.1. Independent R 4.3.2 / survival 3.5-7 uses survreg exponential/
Weibull, tolerance 1e-12, maxiter 100. R covariance is beta/log(sigma), so its
independently implemented Jacobian differs correctly from native raw-sigma J.
Native gretl 2026b duration module defaults are not changed.

Predeclared MLE comparison abs **1e-7 + rel 1e-6*abs(reference)**; exact sample,
parameter names, shape and event/censor partition. Compare all PH beta/logshape,
SEs, all covariance elements (3×3 / 4×4) and likelihood.

| Family | Native vs independent R maximum abs | Native vs external Python observed Hessian max | Original approximate SE discrepancy |
| --- | ---: | ---: | ---: |
| exponential | 1.618e-11 | 1.802e-8 | 7.243e-4 |
| Weibull | 1.257e-11 | 2.699e-8 | 9.778e-4 |

Original public Python beta coefficients pass. Its scipy BFGS/L-BFGS-B inverse-
Hessian approximation SEs do not; original values are preserved separately.
ADR 0016 accepts observed information, not old SE parity. External observed probe
uses the same likelihood/start/optimizer and independently evaluates symmetric
Hessian at that solution. No tolerance widening, data reselection or pinned
checkout modification. Optimizer/time/rate clipping extremes remain unvalidated.

## Boundary and native limitations

Independent `survival-plugin` selects only duration.so under existing verified
shared-core/source/lifetime controls. Default stays static; other modules are not
implicitly built. No prebuilt shortcut, public path setter or unload. Shared
core/loader/fit/destructors remain serialized and process lifetime; source excludes
libR. Absolute paths are spike-only, not relocatable packaging acceptance.

Native returns copied MODEL with owned coefficients/packed VCV/names. Its temporary
inverse-Hessian matrix is copied into model VCV then freed. Shim owns model and
any normalized list; RAII frees model once. Bounds/separator/indices/constant are
checked before FFI; C shim additionally validates positive finite times, binary
censoring and at least one event. Failed models are never read for coefficients/
covariance. No native pointer/model/matrix ownership escapes safe owned results.

Native precheck blindly decrements initial OLS rank for censor dummy. All-event
samples with supplied zero dummy would silently lose a real predictor after OLS
already dropped that dummy. Facade omits it; low-level shim independently strips
it from an owned copied list, never mutating the caller's Rust slice. All-censored
input is rejected before entering plugin. Mixed censor-dummy collinearity remains
an unsupported native design: exact parameter/sample checks reject rank loss.
Do not call that scientifically valid design generally supported.

Tests cover references/labels/sample and exact event IDs, time-unit normalization,
nonpositive/nonfinite times, nonbinary failure/censor, ragged/no-intercept/all-censored,
collinear predictors/dummy, recovery, all-event fits, malformed low-level lists and
failed buffers, parallel families and mixed Poisson/Tobit/quantile/OLS/survival.
Actual linked missing duration.so returns owned error in fresh process; restoring
it allows reference fits in a new process. Survival-only directory is checked not
to contain other modules.

macOS default 21, count-only 26, Tobit-only 26, quantile-only 25, survival-only 26,
combined 41 tests pass; ordinary missing-loader probes ignored and separately
exercised. Same [Linux x86_64 emulated container](linux-revalidation.md) passes
combined 41 / survival-only 26 / default 21 source tests. These are not physical-host
performance or Linux missing-module acceptance. Locked fmt/check/Clippy, root full
baseline, strict audit (zero vulnerabilities/warnings) and unchanged deny policy
pass; no Cargo dependency/waiver. A composite local batch timed out during Tobit
rebuild; both interrupted/unstarted feature suites were explicitly rerun to completion.
Full geiger inventory records 70 used unsafe expressions in adapter; nested facade
repeats 70 (known attribution) while compiler forbid remains enforced. Reports
retained unfiltered, not native sanitizer/leak certification.

Three owner reviews, not independent: PH/censor/covariance/density conventions;
owned list/model/partial-error and low-level protections; sample/feature/concurrency/
platform evidence. Initial likelihood disagreement and redundant-dummy defect were
fixed and follow-up checks run, including direct low-level tests. No unresolved
bounded-slice defect; broader native/estimator limitations remain explicit.

## Release cost and distribution

macOS 27.0.1 arm64 M1 Max, Rust 1.97.1, five fresh release processes, explicit native
init then first owned 320-row Weibull fit and 1,000 repeats. Parsing precedes timers;
compilation excluded and OS caches not evicted. Owned cost includes dataset/list,
plugin first load, estimation, full PH covariance/likelihood/sample copying and RAII.

Medians: init **17.408 ms**, first fit **0.939 ms**, repeat **257.814 µs**, RSS
**9,633,792 bytes (~9.19 MiB)**. First-fit raw values 48.718, .956, .888, .868,
.939 ms preserve initial load/cache outlier; median is not fully cold. Init still
includes missing-gretlcli otool diagnostic/subprocess. Binary 555,216 bytes,
core 3,379,608 bytes, duration module 53,096 bytes; system/transitive artifacts
excluded. Module shares gretl GPL licensing context, but full source/notice/native
redistribution approval is not established. No runtime Python/R.

```sh
# From pinned Python checkout:
uv run python ../tabdat-explore-rs/spikes/gretl-prototype/fixtures/survival_python.py
uv run python ../tabdat-explore-rs/spikes/gretl-prototype/fixtures/survival_observed.py
# From Rust repository:
Rscript spikes/gretl-prototype/fixtures/survival_reference.R spikes/gretl-prototype/fixtures/survival.csv
CARGO_TARGET_DIR=target/gretl-review cargo test --locked --manifest-path spikes/gretl-prototype/Cargo.toml --all-features --all-targets
CARGO_TARGET_DIR=target/gretl-review cargo build --release --locked --manifest-path spikes/gretl-prototype/Cargo.toml --features survival-plugin --bin survival-measure
/usr/bin/time -l target/gretl-review/release/survival-measure
```

No Cox, robust/cluster, left truncation, competing risks, missing sample deletion,
no-intercept/mixed dummy-collinear support, predictions/model state or product
command acceptance. Comprehensive native/transitive safety, sanitizers/leaks,
relocation, other architectures and primary-backend decisions remain separate.
