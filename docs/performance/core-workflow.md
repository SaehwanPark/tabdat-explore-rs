# Local core-workflow measurements

`examples/measure_core_workflow.rs` measures the installed CSV/EDA/OLS/prediction
workflow using existing APIs and dev dependencies. It is a development tool, not
a production feature, stable CLI-v1 interface, CI timing gate or release benchmark
certification. There are no performance pass/fail thresholds.

## Run

Use the same source revision, Rust toolchain, flags and profile for the supplied
binary and linked measurement example. For useful optimization comparisons:

```sh
cargo build --locked --release --bin tabdat-explore-rs --example measure_core_workflow
target/release/examples/measure_core_workflow target/release/tabdat-explore-rs \
  --rows 10000 --repetitions 7 --label 'release; record revision/flags separately' \
  > target/core-workflow-release.json
```

Bundled native dependencies can make the first release build expensive. A debug
smoke check verifies the harness, **not release responsiveness**:

```sh
cargo build --locked --bin tabdat-explore-rs --example measure_core_workflow
target/debug/examples/measure_core_workflow target/debug/tabdat-explore-rs \
  --rows 10000 --repetitions 5 --label 'debug; matching source' \
  > target/core-workflow-debug.json
cargo test --locked --example measure_core_workflow
```

`--rows` accepts 4..=1000000; `--repetitions` accepts 1..=50. Defaults are 10000
and 7. Large combinations cost more time/memory. `--label` is supplied context,
not an inferred/verified optimization level; the harness records its own debug
assertion flag separately. Keep reports with `git rev-parse HEAD`,
`rustc --version`, build flags, CPU/OS details, row count and repetitions. Compare only
matching workloads/profiles on a reasonably idle machine; retain raw samples.

## What is timed

Each case has **one excluded warmup** followed by the requested samples. Fixture
creation and parent-side correctness validation are outside the timers. The JSON
artifact contains raw milliseconds plus minimum, median and maximum for:

| Case | Scope |
| --- | --- |
| `fresh_process_version` | Spawn through process completion/output capture of `--version` |
| `fresh_process_json_workflow` | Fresh CLI process: use → count → summarize x y → regress y x → predict fitted → head 3, including JSON reporting and teardown |
| `session_load_csv` | Execute load in a new session, including lazy DuckDB initialization |
| `session_summarize` | Execute ordinary summary after loading outside the timer |
| `session_summarize_detail` | Execute fixed finite type-7 percentiles plus ordinary statistics after loading outside the timer |
| `session_regress` | Execute OLS after loading outside the timer |
| `session_predict` | Execute prediction after loading/fitting outside the timer |

Every session sample starts a new session. Session times exclude parsing,
setup commands, result serialization, validation and teardown. Process times
include child reporting/teardown, so subtracting these cases does not isolate a
particular cost. The example's session cases use its linked runtime, not the
supplied executable. Version equality alone cannot prove matching source/profile.

**Fresh process is not OS-cold cache.** The tool does not flush filesystem caches,
control CPU frequency/thermal state, isolate scheduler activity, or measure
interactive rendering/input latency or per-command memory/peak RSS. No Parquet/large-string/wide-design/weighted
model coverage, general performance guarantee or cross-platform certification is
implied. Benchmark those separately when an actual need appears.

## Correctness and failures

A temporary CSV contains `i=0..N-1`, `x=i/N`,
`y=3+2*x+alternating(-1,+1)` and `group=i%10`; all values are finite.
The predictor is normalized to [0,1) so the baseline measures a well-conditioned
core workload, not accuracy under increasing predictor magnitude. The tool
validates CLI-v1 tags/counts, ordered means,
OLS/intercept/nonrobust metadata and sample size/DF, predictions and row order.
Model/prediction session cases also check exact retained indices `0..N`. Independent algebra gives:

- Even N: slope `2+6*N/(N*N-1)`, intercept `3-3/(N+1)`.
- Odd N: slope `2`, intercept `3-1/N`.

Detail validation checks the type-7 method, two ordered finite samples, and all
nine ordered percentiles. For x the sorted kth value is k/N. For y, all even-index
observations precede the odd-index ones: with E=ceil(N/2), the sorted kth value is
2+4k/N for k<E, otherwise 4+(4(k-E)+2)/N. Linear interpolation at p*(N-1)
therefore supplies expected percentiles without sorting data or querying DuckDB.
Even/odd hand-computed values and real sessions test this algebra. Missing/wrong
method, sample, order, count, mean, percentile count/IDs or null/bad values reject
the observation. The existing fresh-process workflow remains ordinary-summary-only;
detail is an additional linked-runtime session case, not a new process/PTY workload.

Means, coefficients, percentiles and preview predictions use a fixed absolute **1e-8** health
check tolerance for this normalized fixture up to 1000000 rows. This is not a new
estimator/inference validation contract, and it does not relax existing statistical
test tolerances.
An initial unscaled `x=i` one-million-row trial failed this guard: intercept
2.999996908512485 versus exact 2.999997000003 (about 9.15e-8 absolute drift).
Exact rational algebra and a NumPy centered-noise calculation agree on the latter;
naive large-offset numerical references also drift. Normalization is a deliberate
baseline-fixture choice, not a production numerical fix or an accuracy claim for
that stress case. No tolerance was widened.

Bad/malformed/missing results, unsuccessful child exits or child diagnostics abort
the run rather than contributing timings. Output is emitted only after all cases
succeed; errors exit 1, invalid options exit 2. Temporary fixtures are removed by
RAII on ordinary success/failure (not a process-kill cleanup guarantee).

## Controlling-PTY workflow observations

`tools/check_terminal_pty.py` also provides an opt-in, standard-library-only Unix
measurement mode. Its default acceptance invocation is unchanged. This is an
external development tool, not a production Python dependency or a stable JSON
machine contract.

```sh
cargo build --locked --release --bin tabdat-explore-rs
python3 tools/check_terminal_pty.py target/release/tabdat-explore-rs \
  --measure --repetitions 7 --label 'release; record source/tool revisions and flags' \
  > target/pty-workflow-release.json
python3 -m unittest discover -s tools -p 'test_check_terminal_pty.py'
```

`--repetitions` accepts 1..50 (default 7); it and `--label` require `--measure`.
Invalid options or a missing/nonexecutable binary exit 2. Measurement failures exit
nonzero without a success report; missing output/prompts use the existing 15-second
acceptance timeout. The report is emitted only after acceptance, numeric validation,
clean EOF and canonical/echo restoration. Ordinary failures
close/reap the child and remove the temporary fixture; process-kill cleanup is not
promised. Unit tests cover malformed arguments/results, raw sample aggregation,
warmup exclusion, cycle order and failed-child cleanup/no report. A debug invocation
is a harness smoke check only; use matching release builds for release observations.

The existing completion/editing/history/cancellation/model-retention acceptance flow
runs first in the same process, including literal `use`/`run` path completion,
quoted CSV option suffixes, bare registered-name completion with same-name file/directory
precedence, and current-schema column/registry refresh after transformations and partial
failed scripts. The two intentional recovery diagnostics are checked
exactly; missing, changed or extra errors fail acceptance. Historical observations
below pin the earlier tool revision/acceptance sequence; their samples are not
replaced by this coverage change. Then **one excluded warmup cycle** precedes
measured cycles of load → count → summarize x y → regress y x → predict fitted. Every cycle
reloads the same fixed four-row, two-column CSV, so prediction does not grow the
schema between samples. Untimed `head 4` verifies actual current rows/predictions
after each cycle. Counts, ordered means, model sample/design and coefficients are
checked outside the timers. For `x=[0,1,2,3]`, `y=[1,2,4,6]`, exact OLS algebra gives
intercept 7/10 and slope 17/10. Means/coefficients/predictions use absolute **1e-10**
fixture health guards, not changed statistical test tolerances.

Each observation uses the parent's monotonic clock from input submission through
recognition of the expected output **and next prompt**. It includes parsing,
execution, terminal rendering, transport, scheduling, and observer ANSI/marker
processing; numeric validation and post-prediction previews are outside the timers.
The JSON includes raw/min/median/max milliseconds, fixture content, warmup/sample
counts, host/Python details, fixed xterm-256color 24×80 PTY and supplied label. The
label does not verify source/profile. Record build commands, revisions and flags
separately; no OS cache, CPU frequency or scheduler isolation is performed.

This measures a **warmed persistent-session PTY round trip**, not physical display
paint, startup, human keystrokes, completion latency, large-data performance or a
cross-platform guarantee. Do not subtract the Rust example's session times from
these observations: workload, reporting, transport and timing boundaries differ.
No timing gate or optimization follows merely from these observations.

[Raw PTY observations and context](baselines/macos-aarch64-pty-release.json) retain
two sequential local release runs on Apple M1 Max/Darwin 27.0.0 arm64 with Rust
1.97.1. Rust source is `a84f69a`, measurement tool is `5ccf10c`; no Rust/Cargo changes
occur between them. Only machine-local binary paths were normalized. Each run has
seven measured samples after its excluded warmup and acceptance preconditioning.

| Command | Run A median ms | Run B median ms |
| --- | ---: | ---: |
| Load four-row CSV | 1.222 | 1.290 |
| Count | 0.096 | 0.111 |
| Summarize x y | 0.570 | 0.631 |
| Regress y x | 0.215 | 0.244 |
| Predict fitted | 0.830 | 0.903 |

The release build and PTY acceptance/health guards passed. A full release Rust test
suite was **not** run for this tooling-only slice; normal-profile repository checks
remain separate evidence. The two repeats are not controlled independent machines
or an optimization comparison. No production optimization is justified from this
small warmed fixture alone.

## Local detail-summary release observations

[Raw detail-summary samples and build context](baselines/macos-aarch64-summary-detail-release.json)
retain two sequential 10000-row runs (seven measured samples each) and one
1000000-row run (three samples) on Apple M1 Max/macOS 27.0.1/aarch64, Rust 1.97.1.
The locked release binary and example were built together at `f640b35`; only the
measurement example differs from application source `2bf7357` (#255). Default
release profile, inspected flags/config, hashes, native dependency context and
raw/min/median/max samples are recorded; only machine-local binary paths were
normalized. Every case has one excluded warmup and passes its correctness guards.

| Session execute-only median ms | 10000 rows, A | 10000 rows, B | 1000000 rows |
| --- | ---: | ---: | ---: |
| Ordinary summarize x y | 0.674 | 0.733 | 6.319 |
| Summarize x y, detail | 3.696 | 3.842 | 236.992 |

These are different workloads in fresh sessions of the same linked harness, not
an optimization comparison or isolated sorting-overhead measurement. Detail adds
native percentile aggregation to ordinary statistics; do not subtract cases to
attribute a particular internal cost. The two repeats are not independent hosts,
and the larger row count is not a regression comparison. This fixture is finite,
two-variable and normalized; null-heavy, wide, other-format and memory behavior
remain unmeasured. No optimization, timing gate or performance/platform promise
follows from this one-host record. Earlier baseline artifacts below remain pinned
and unchanged; source, tool and preconditioning differ, so they are not controlled
before/after comparisons with these observations.

## Local release baseline

[Raw samples and build context](baselines/macos-aarch64-core-release.json) record
matching binary/example release builds of code revision `255d596` (measurement
checkout `f9186ea`, an empty starting commit with no code changes). This is one
local **macOS 27.0.1 / aarch64 / Apple M1 Max** observation, using Rust/Cargo
1.97.1, the default Cargo release profile, no observed environment/repository or
global Cargo profile/flag overrides, and unchanged bundled DuckDB dependencies.
Available host Clang and exact Cargo package versions are recorded in the artifact.
Machine-local binary paths alone were normalized to repository-relative paths for
publication; no samples or other harness fields were changed.

Both binary and example built together with the locked release command in Run.
The complete
`cargo test --locked --release --workspace --all-targets` run passed **850 tests,
zero failures across 128 suites**, including the five measurement-example tests.
Every measured run passed the existing count/mean/design/coefficient/prediction
and sample-identity guards; no tolerance or application code changed.

Median times in milliseconds (rounded here; raw/min/max retained in the artifact):

| Case | 10000 rows, run A (7 samples) | 10000 rows, run B (7 samples) | 1000000 rows (3 samples) |
| --- | ---: | ---: | ---: |
| Fresh-process version | 12.526 | 12.543 | 13.171 |
| Fresh-process JSON workflow | 39.255 | 40.024 | 310.703 |
| Session CSV load | 18.312 | 18.750 | 75.761 |
| Session summary | 0.765 | 0.861 | 6.055 |
| Session OLS | 1.675 | 1.686 | 163.471 |
| Session prediction | 1.554 | 1.559 | 8.690 |

CSV sizes were 184365 and 21989580 bytes respectively. Each case had one excluded
warmup. The two small-workload runs were sequential repeats, not independently
controlled machines; the larger workload is a different workload, not a regression
comparison. Ordinary scheduler/thermal/cache effects were not isolated. Do not
subtract process/session times to infer a specific overhead or extrapolate from
this two-parameter finite-numeric fixture to other datasets/designs.

The recorded release workflow medians are about 39–40 ms for 10000 rows and
311 ms for 1000000 rows on this host. They provide a repeatable local starting
point, **not an optimization result, performance promise, timing gate or evidence
of release readiness**. Debug measurements in PR #218 validate the harness but
are a different profile, not a comparable regression baseline. No code
optimization is justified solely by this one-host record; collect matching
before/after or workflow-specific evidence when a real responsiveness issue appears.
