# Balanced classical panel FE/RE feasibility

PR [#173](https://github.com/SaehwanPark/tabdat-explore-rs/pull/173), base
`62b0bd5`. This is an isolated spike, **not product xtreg acceptance**.

## Contract and references

`PanelProblem` owns outcome/predictor names, row-major outcome/predictors, periods
and explicit canonical zero-based entity/time keys. Only balanced finite complete
entity-major/time-minor panels are accepted, with at least two periods and more
entities than predictors plus one. Keys must match every input row exactly.

`run_panel` exposes owned coefficients, standard errors, full covariance, exact
sample rows and keys. FE exposes within slopes only: the native normalized average
intercept is intentionally omitted because entity effects absorb the constant.
RE uses classical Swamy-Arora with an explicit intercept. Gretl requires a constant
in its baseline fit. **Python's default no-constant RE command is not validated**:
the pinned executor is invoked with an explicit constant column. Python `const`,
R `(Intercept)` and native reserved variable zero represent the same design and
are exposed as `intercept`. No pinned oracle source modification or tolerance
relaxation was made. General R-squared/likelihood/inference-DF normalization is not
part of this result.

Fixture: seed 8147, 24 entities × 8 periods, two independent Gaussian predictors,
Gaussian entity effects and residuals. Data SHA-256:
`43bf13fe20c0fa047eab72ed203fa638442ebcb8fc3797bcab8d9d8c24e4cac1`.
Pinned oracle identity remains the [migration record](../migration/README.md).
The external Python script invokes `panel id time` then `xtreg ..., fe/re` and
extracts linearmodels fits. Independent R 4.2.2 / plm 2.6-2 runs in the existing
Debian container, using within and `random.method='swar'`. Python's linearmodels
unadjusted covariance and R plm classical covariance are retained unchanged.

Predeclared comparison: absolute `1e-9` + relative `1e-8*abs(reference)`; exact
sample, key order, coefficient labels and output shape. All coefficients/SEs/full
covariance pass. macOS maximum absolute differences across exposed fields:

| Reference | FE | RE |
| --- | ---: | ---: |
| pinned Python | 2.220e-16 | 4.441e-16 |
| independent R plm | 8.882e-16 | 1.998e-15 |

The same comparisons pass under emulated Linux x86_64; this is not physical-host
performance evidence. Both static/default and shared/count-plugin routes are
checked separately.

## Boundary and verification

Native dataset structure and period are set/restored within one serialized shim
call. No raw pointer, MODEL or mutable global panel option escapes the adapter.
The existing model RAII/destructor and process-lifetime serialization apply.
Successful fits must retain the exact number of observations and all baseline
coefficients, preventing silent rank drops. Native errors remain owned errors;
no failed-fit covariance access occurs. Native classical VCV is materialized
before exposing owned copies.

Tests cover FE/RE reference values, shape/names/keys/sample, ragged/nonfinite data,
duplicate/missing keys and names, invalid periods, native collinear/constant design
failures, recovery and four-thread repeated FE/RE calls. Low-level malformed lists,
missing constants and invalid periods panic before FFI, not through C.

macOS locked format/check/test/Clippy, strict audit (zero vulnerabilities/warnings),
unchanged deny policy and root format/check/test/Clippy pass. Final suites contain
26 shared-feature and 21 default tests, including the four new panel checks.
Full adapter geiger inventory contains 54 used unsafe expressions, including one
new reasoned panel FFI call. Full nested facade inventory repeats these 54 under
the facade (the previously recorded attribution problem); the report is retained,
not filtered. Ordinary facade compiler prohibition remains in force. This is an inventory, not
complete native safety, sanitizer or redistribution approval. No Cargo dependency,
plugin, advisory waiver or runtime Python/R dependency was added. Initialization
still emits the previously recorded missing-gretlcli otool/ldd diagnostic.

Three owner review passes (not independent reviews): contract/reference alignment;
FFI lifetime/metadata/error handling; test/recovery/platform/dependency scope.
The boundary pass added an explicit minimum list length check before the shim.
Follow-up checks run after that fix. No unresolved blocker in this bounded slice.

## Reproduction and limitations

From the pinned Python checkout:

```sh
uv run python ../tabdat-explore-rs/spikes/gretl-prototype/fixtures/panel_python.py
```

R reference uses `/work` in `tabdat-phase1-linux`:

```sh
docker exec tabdat-phase1-linux Rscript /work/spikes/gretl-prototype/fixtures/panel_reference.R
CARGO_TARGET_DIR=target/gretl-review cargo test --locked --manifest-path spikes/gretl-prototype/Cargo.toml --all-targets
# Repeat with --features count-plugin for shared-core coverage.
```

No unbalanced panels, arbitrary IDs/gaps, missing sample deletion, robust/cluster,
time effects, Hausman, prediction/session state or dynamic-panel acceptance. RE
without an explicit intercept needs a separate oracle/backend decision. Native
packaging/lifetime, other architectures and license/distribution gates remain
open. Passing this bounded fixture does not select libgretl as the primary backend.
