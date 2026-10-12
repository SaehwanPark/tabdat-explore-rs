# Quantile LP and canonical iid covariance feasibility

[PR #175](https://github.com/SaehwanPark/tabdat-explore-rs/pull/175), base
`8e25c80`; [ADR 0015](../adr/0015-quantile-lp-and-iid-covariance-normalization.md).
**Pass for bounded LP/canonical iid feasibility**, not default-IRLS parity or
product qreg support.

## Contract and numerical authority

`run_quantile` accepts finite complete full-rank `EstimationProblem` and quantile
.01 through .99. Nominal validation covers .25/.5/.75, intercept/two predictors
and exact ordered sample. Owned output includes coefficients/SEs, canonical iid
full covariance, bandwidth, residual density, check-loss pseudo-R² and sample rows.
No robust/cluster, alternate kernels, general missing sample deletion, confidence
intervals/DF/post-estimation or product model state is accepted by this slice.

Native `quantreg` supplies mature LP coefficients. Canonical covariance is
normalized in safe Rust to pinned statsmodels defaults: Epanechnikov kernel,
Hall-Sheather alpha .05, population outcome SD, type-7 linearly interpolated
residual IQR/1.34 scale, q(1-q)/density² times inverse X'X. Reuse checked owned QR/OLS
sigma² and covariance for inverse X'X; no second inversion/runtime dependency.
Reject zero/nonfinite sigma², invalid probability bandwidth and degenerate density.
Normal inverse comes through a checked serialized scalar-only native call.

`native_iid_covariance` is a separately labeled backend diagnostic, **not canonical
Python inference**. Gretl's iid sparsity estimator differs. It matches R iid at
.25/.5 (~9e-12), but .75 differs ~5.92e-4 even with near-identical coefficients.
The precise sparsity/near-zero-residual sensitivity is not proven resolved; its
output is retained, not silently substituted for canonical covariance.

Fixture: 360 rows, seed 21307, Gaussian predictors, Student-t(4) noise, fixed CSV
SHA-256 `79d8662b623d909be34f4a49482851f6ba06199a008b031b0b002806726b1763`.
Pinned Python authority remains the [migration record](../migration/README.md),
statsmodels 0.14.6 / SciPy 1.17.1. Actual executor uses default IRLS (p_tol 1e-6,
max_iter 1000), stopping at 57/25/34 iterations. All original full results are
preserved in `quantile-original-python.csv`.

Predeclared comparator abs **1e-8 + rel 1e-7*abs(reference)** across beta/SEs,
all 9 canonical covariance entries, bandwidth, density and pseudo-R²; exact names,
shape and sample. Independent external Python uses HiGHS check-loss LP, checks
primal residual <1e-8 and evaluates the pinned covariance convention. Independent
R 4.3.2 / quantreg 5.97 uses Barrodale-Roberts coefficients and independently
written R quantile/density/cross-product algebra. Neither external probe is called
unmodified executor output.

| q | Native vs HiGHS/R maximum absolute difference | Original IRLS beta discrepancy | Original excess check loss |
| --- | ---: | ---: | ---: |
| .25 | 1.276e-12 | 5.254e-7 | 8.198e-7 |
| .50 | 5.218e-15 | 6.139e-3 | 1.509711e-3 |
| .75 | 3.754e-9 | 5.893e-3 | 3.247315e-5 |

All normalized comparisons pass without changing tolerances/data. Original IRLS
coefficients fail. R emits a possible-nonuniqueness warning, retained in its
reproduction stderr; agreement on this fixture does not prove general uniqueness.
Positive objective excess establishes the original difference is not merely
another equally optimal representation. ADR 0015 accepts the LP target explicitly
and records the stopping discrepancy, without modifying the pinned checkout.

## Native boundary, feature isolation and verification

Opt-in `quantile-plugin` builds only source-matched quantreg.so under existing
verified shared-core/lifetime controls (ADRs 0012/0014). Default remains static;
count/interval modules are not built by quantile-only selection. Combined features
share one core and serialized loader/calls/destructors. Private compile-time path
uses the longest selected basename in its native bound assertion; no setter or
unload is exposed. Source build continues `--without-libR` and rejects prebuilt
shortcuts for plugins. Absolute paths remain nonrelocatable spike artifacts.

Shim allocates one-element tau matrix, calls quantreg and frees it; single-quantile
rq_driver stores tau as scalar model metadata, not a borrow. Failed allocation
returns owned failed model or the existing allocation panic. Copied MODEL owns
coefficient/VCV data and existing RAII frees it. Rust validates full rank/sample,
finite output and coefficient indices before copying. No foreign model/matrix
pointer escapes. Source-backed safety comments accompany two new native calls.

Tests cover metadata/shape/references/objective discrepancy, invalid tau/probability,
list mismatch, ragged/nonfinite/collinear/constant outcome, recovery, no-intercept
smoke, failed covariance buffers, parallel quantiles and mixed Poisson/Tobit/OLS/
quantile calls. Fresh-process missing **actual linked** quantreg.so returns owned
native error; restoring the module makes reference fits pass in a new process.
Quantile-only directory is checked not to contain interval.so or poisson.so.

macOS default 21, count-only 26, Tobit-only 26, quantile-only 25 and combined 36
tests pass; ordinary missing-module tests are ignored and separately probed.
Locked format/check/Clippy, strict audit (zero vulnerabilities/warnings), unchanged
deny policy and root full baseline pass. Linux x86_64 emulated source checks use
the existing [container](linux-revalidation.md), not host-performance evidence.
Full geiger inventory records 66 used adapter unsafe expressions. Nested facade
inventory repeats 66 (known attribution issue), with compiler forbid-unsafe still
enforced; unfiltered reports are retained. This is not sanitizer/leak or comprehensive
native/transitive safety approval.

Three owner reviews, not independent: solver/objective/covariance authority; tau/
model/shape/loader ownership; invalid/recovery/feature/platform tests. Follow-up
adds pre-FFI list-dimension guard and explicit positive finite sigma² validation.
No unresolved bounded-slice defect; wider inference and packaging decisions remain.

## Costs and license/distribution

macOS 27.0.1 arm64 M1 Max, Rust 1.97.1, five fresh release processes, each with
explicit native init, first owned median-regression fit and 1,000 repeated fits.
Parsing precedes timers; compilation excluded, caches not evicted. Owned fit cost
includes checked OLS/inverse preparation, dataset/list creation, LP/plugin load on
first use, canonical density/covariance, result copies and RAII—not pure LP arithmetic.

Medians: init **18.734 ms**, first fit **1.393 ms**, repeated **241.053 µs**, RSS
**9,732,096 bytes (~9.28 MiB)**. First-fit raw values 60.924, 1.370, 1.348, 1.393,
1.448 ms preserve the initial load/cache outlier; median is not fully cold. Init
still includes missing-gretlcli otool diagnostic/subprocess. Executable 558,272 bytes,
core 3,379,608 bytes, module 72,808 bytes; transitive/system distribution excluded.

The translated rq sources carry University of Illinois/NCSA copyright 2002 Roger
Koenker, in addition to gretl's GPL context. [Notice](../third-party-notices/gretl-quantile-NCSA.txt)
preserves the upstream conditions/disclaimer (trailing whitespace normalized).
Source/binary redistribution must carry them and respect nonendorsement. Permissive
sublicensing is compatible in principle with AGPL intent, not legal/distribution
approval for the full native graph. No Cargo package, audit waiver or runtime
Python/R added. Native safety, relocation, other architectures and primary backend
acceptance remain separate gates.

```sh
# From pinned Python checkout:
uv run python ../tabdat-explore-rs/spikes/gretl-prototype/fixtures/quantile_python.py
uv run python ../tabdat-explore-rs/spikes/gretl-prototype/fixtures/quantile_lp.py
# From Rust repository:
Rscript spikes/gretl-prototype/fixtures/quantile_reference.R spikes/gretl-prototype/fixtures/quantile.csv
CARGO_TARGET_DIR=target/gretl-review cargo test --locked --manifest-path spikes/gretl-prototype/Cargo.toml --all-features --all-targets
CARGO_TARGET_DIR=target/gretl-review cargo build --release --locked --manifest-path spikes/gretl-prototype/Cargo.toml --features quantile-plugin --bin quantile-measure
/usr/bin/time -l target/gretl-review/release/quantile-measure
```
