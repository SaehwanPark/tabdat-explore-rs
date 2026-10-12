# Collapsed one-step difference-GMM feasibility

[PR #178](https://github.com/SaehwanPark/tabdat-explore-rs/pull/178), base
`f78be4a`; [ADR 0018](../adr/0018-dynamic-panel-entity-covariance.md).
**Pass for bounded dynamic-panel feasibility**, not general/default xtabond
inference, product command or primary-backend acceptance.

## Contract and independent references

`run_dynamic_panel(&PanelProblem)` owns results for one-step first-difference
Arellano–Bond with one lagged outcome, collapsed lag-2-only level outcome instrument
and strictly exogenous predictors instrumented by their differences. Individual
effects removed, no time effects/intercept/system/two-step/Windmeijer route.
Result owns beta `L1.y`, x1/x2, full covariance/SE, exact native sample rows/keys,
native included-group and instrument counts. Entity-summed one-step sandwich with
**no finite-N correction**, not independent-row heteroskedasticity. No reference
distribution/CI/AR/Hansen/Sargan/prediction/session output in this bounded result.

Fixture seed 29183: 64 entities × 8 periods, two Gaussian predictors, outcome
AR .55 with individual effect and .65 innovation SD, 32 initial burn-in periods.
Finite complete entity-major/time-minor zero-based keys; usable t=2..7, **384 rows**,
64 native groups / **3 instruments**. CSV SHA-256
`4bf6566ac1ea01a9042d79e012403f7cc6946494a0863e4579e75adb314234f2`.
Pin identity in [migration record](../migration/README.md), statsmodels 0.14.6 /
SciPy 1.17.1 / pinned linearmodels. Independent container R **4.2.2 Patched / plm
2.6-2** uses pgmm(one-step, individual, difference, collapse), explicit lag-2 and
normal x instruments, vcovHC; separate Python entity-moment algebra.

Predeclared abs **1e-9 + rel 1e-8*abs(reference)** for every beta/SE/full 3×3 VCV;
exact names/shape/sample/group/instrument metadata and finite values.

| Comparison | Maximum absolute difference |
| --- | ---: |
| native vs independent Python entity-moment algebra | 1.110e-16 |
| native vs R plm pgmm/vcovHC | 4.441e-16 |
| native entity SE vs original independent-row executor SE | 6.924e-2 |

Let X contain differenced lagged y and x, Z contain level y(t-2) and differenced x.
For this just-identified case beta=`solve(Z'X,Z'y)`; covariance is
`G^-1 [sum_i (Z_i'u_i)(Z_i'u_i)'] G^-T`, G=Z'X. Native one-step weighting uses
first-difference error structure; just identification makes points independent of
weighting. **No overidentified weighting/multi-lag acceptance** follows.

Original pinned executor uses pooled IVGMM and defaults to two GMM steps; robust
covariance treats transformed rows as independent, not entity clusters. Original
points pass, original SE/VCV remain separately preserved. ADR 0018 accepts explicit
entity inference, never old/default SE parity or widened tolerance. R fallback
has different defaults too; broader runtime route normalization remains open.

## Native boundary and owner review

Independent dpanel-plugin selects source-matched dpanel.so sharing verified core,
lazy serialized owned lifecycle and private bounded path. Static default/other
single-feature builds unchanged. No prebuilt shortcut/path setter/unload, libR or
Cargo dependency/waiver. Dataset pd/structure set/restored within one call on both
success/failure. Native MODEL/arrays/names remain RAII owned.

Initial native attempt failed singular (code43): supplied GMM specification disables
implicit x instruments. Corrected with checked second separator and explicit
repeated exogenous instrument roles. This is design correction, not data/tolerance
change. Closed lag/spec/indices/constant/list roles checked before C; native name
characters cannot inject another GMM expression. Prototype caps explicit: periods
4..32, entities8..4096, predictors1..16, low-level dataset variables<=32. General
native dimension/arithmetic acceptance is not inferred from these bounded caps.

Owned residual-mask copy validates successful MODEL type DPANEL/full_n/allocation
before bounded reads/writes. Exact actual mask agrees with expected IDs, not just N.
Scalar metadata/shape probes expose no foreign ownership. Low-level malformed input
panics before numerical call; facade returns owned invalid/native errors. Allocation
failure remains prototype behavior. Native dpmod initialization/allocation cleanup
and comprehensive fault-injection/leak proof remain unaccepted.

Tests cover full references/keys, invalid periods/key duplication/ragged/nonfinite/
reserved-name/rank cases, recovery, wrong list/lag/role/index, wrong-length and
non-DPD sample buffers, failed getters and actual dataset metadata restoration,
same-dataset recovery, parallel fits and seven-way mixed plugin/core threads.
Actual linked missing dpanel.so fresh process returns owned error, restoration in
finally and new process fits reference; dpanel-only directory excludes other modules.

macOS **50 combined / 25 dpanel-only / 21 default**, same emulated
[Linux x86_64](linux-revalidation.md) **50 / 25 / 21** pass. Final Linux all50
includes native group getter, MODEL-type guard and mixed-thread follow-up. No
Linux missing-module or physical-host performance claim. Root full locked gates,
spike/adapter fmt, check/Clippy, strict audit zero vulnerabilities/warnings and
unchanged deny pass. Full geiger retains **90 used adapter unsafe expressions**,
known nested-facade attribution90/compiler forbid enforced, not native certification.

Three owner reviews, not independent: covariance/just-identification/reference
scope; model/dataset/metadata/sample buffer/closed-list ownership; feature/loader/
platform/concurrency and evidence. Initial instrument-design correction and native
MODEL-type/native-group follow-up pass. Two composite local batches timed out:
Linux final was waiting on ongoing default rebuild; release batch expired during
fresh native build. Original Linux combined/only/default completed; final all and
release measurement explicitly replayed successfully. No timeout counted as pass.

## Release cost and distribution

macOS 27.0.1 arm64 M1 Max / Rust 1.97.1, five fresh release processes, explicit init,
first owned 512-input-row/384-sample fit, 1,000 repeats. Parsing outside timers,
compilation excluded, OS caches not evicted. Includes native first module load,
data/list, fit, full covariance/metadata/sample copying and RAII.

Medians: init **19.485 ms**, first **1.045 ms**, repeated **107.526 µs**, RSS
**9,879,552 bytes (~9.42 MiB)**. First raw ms: 52.714, .981, 1.038, 1.185, 1.045;
initial-load outlier preserved, median not fully cold. Executable **552,112**, core
**3,379,608**, module **74,056 bytes**, excluding system/transitive artifacts.
Known missing-gretlcli otool/ldd diagnostic remains. Full GPL/native source/notice/
license distribution, relocation and other architectures remain open.

```sh
# From pinned Python checkout:
uv run python ../tabdat-explore-rs/spikes/gretl-prototype/fixtures/dynamic_panel_python.py
# From Rust repository / external R with plm:
Rscript spikes/gretl-prototype/fixtures/dynamic_panel_reference.R spikes/gretl-prototype/fixtures/dynamic-panel.csv
CARGO_TARGET_DIR=target/gretl-review cargo test --locked --manifest-path spikes/gretl-prototype/Cargo.toml --all-features --all-targets
CARGO_TARGET_DIR=target/gretl-review cargo build --release --locked --manifest-path spikes/gretl-prototype/Cargo.toml --features dpanel-plugin --bin dynamic-panel-measure
/usr/bin/time -l target/gretl-review/release/dynamic-panel-measure
```

No unbalanced/arbitrary-ID/gapped/missing sample, extra lags/ranges, endogenous x,
uncollapsed/overidentified/system/two-step/Windmeijer/time effects/intercept,
weak-instrument/extreme-design, product covariance options or prediction/model
state support. Comprehensive native/transitive integer/allocation/sanitizer/leak,
licenses/relocation and primary-backend selection remain open. Phase 1 partial.
