# Fresh Linux x86_64 revalidation (2026-10-12)

Local Docker Desktop on Apple Silicon, **linux/amd64 emulation**, not a physical
x86 host or a performance benchmark. Agent-owned container `tabdat-phase1-linux`;
repository mounted read-only at /work, generated target artifacts at /results.
Image `rust:1.97.1-bookworm`, digest
`sha256:0e2bcaef56d041a486784e54104a81aebe0da44bd03019bd70bc0401e42e4a97`.
Rust 1.97.1 (8bab26f4f, 2026-07-14). No hosted Actions or Python/R runtime.

Installed validation/build packages: pkg-config, gettext, libxml2-dev,
libglib2.0-dev, libfftw3-dev, libgmp-dev, libmpfr-dev, libcurl4-openssl-dev,
liblapack-dev, libblas-dev and fonts-dejavu-core from Debian Bookworm repositories.
This is a development image, not a release package/license inventory.

## Passed

- Shared gretl 2026b source build + source-matched count module with `count-plugin`:
  **19 tests pass**, one ignored ordinary manual loader probe. Includes NIST OLS,
  Python/R robust/cluster and binary/Poisson, refined-Python/R NB2, native errors,
  metadata and mixed independent-call serialization. No prebuilt shortcut.
- Native SVG/resvg: **8 tests pass** with explicit DejaVuSans bytes, all three
  chart semantics, exact canvas sizes and actual colored raster pixels. The
  macOS font is not copied or required. Cross-platform *pixel identity* is not
  tested or claimed; this establishes headless Linux build/render feasibility.

Commands inside the container:

```sh
CARGO_TARGET_DIR=/results/gretl CARGO_BUILD_JOBS=2 cargo test --locked \
  --manifest-path spikes/gretl-prototype/Cargo.toml --features count-plugin --all-targets
CARGO_TARGET_DIR=/results/svg cargo test --locked \
  --manifest-path spikes/native-svg-prototype/Cargo.toml --all-targets
```

Source at the NB2 slice PR #171, based on ba24111. Local raw log:
`/tmp/tabdat-linux-gretl-svg.log`; image/commands/counts are the durable evidence.
Root, ReadStat, sanitizers, package relocation and additional estimator families
need their own checks; these passes do not complete all platform/safety gates.

## Subsequent IV and balanced panel runs

At PR #172 the source-built suites reached 22 shared-feature and 17 default
tests. PR #173 repeats both routes with balanced FE/explicit-intercept RE full
Python/R plm references, native key/rank/list/period recovery and concurrent fits:
**26 shared-feature and 21 default tests pass**, one ordinarily ignored manual
plugin probe. Same container/image/architecture; no host-performance claim.
External R 4.2.2 Patched and plm 2.6-2 were installed only for independent panel
references. Native build continues to set `--without-libR`; no validation-tool
runtime dependency was introduced. Native init's missing-gretlcli ldd diagnostic
remains recorded. Raw final log `/tmp/panel-linux-final.log`.
See [IV](iv2sls.md) and [panel](panel.md) for bounded contracts and tolerances.

## Tobit isolated module and combined features

PR #174 revalidates the refactored shared-linkage/module selection on the same
emulated Linux image: **31 combined, 26 Tobit-only, 26 count-only and 21 default**
source-built tests pass. Tobit left-only/two-sided observed-information comparisons,
full covariance/error/bounds/family guards and threaded recovery pass. Ordinary
manual loader tests remain ignored; the actual missing-interval probe is separately
exercised on macOS. No Linux cold-performance, missing-module or redistribution
claim follows from these checks. Final log `/tmp/tobit-linux-final.log`.
[Report](tobit.md), [ADR 0014](../adr/0014-tobit-observed-information-and-plugin-scope.md).

## Quantile LP/canonical iid and mixed plugins

PR #175 revalidates **36 combined, 25 quantile-only and 21 default** tests on the
same emulated Linux image, including .25/.5/.75 full canonical reference fields,
original IRLS objective excess, checked tau/probability and mixed Poisson/Tobit/
quantile/OLS threading. Default/count/Tobit configurations also pass locally on
macOS after module selection changes. No physical-host performance or Linux
missing-module claim; actual linked missing quantreg.so probe is separately
exercised on macOS. Final logs `/tmp/quantile-linux-final.log` and
`/tmp/quantile-linux-last.log`. [Report](quantile.md),
[ADR 0015](../adr/0015-quantile-lp-and-iid-covariance-normalization.md).

## PH survival and low-level censor protections

PR #176 validates **41 combined, 26 survival-only, 21 default** tests on the same
emulated Linux image, including exponential/Weibull full PH observed covariance/
likelihood, exact event IDs, time-unit Jacobian, all-event redundant-dummy normalization
and five-way mixed-plugin threading. Actual missing-duration probe runs separately
on macOS; no Linux missing-module/host-performance or distribution claim.
Final log `/tmp/survival-linux.log`; [report](survival.md),
[ADR 0016](../adr/0016-survival-ph-and-observed-information.md).

## Proper two-step selection and failed-fit dataset restoration

PR #177 validates **46 combined / 26 Heckman-only / 21 default** source tests on
same emulated Linux image. Corrected outcome/observed probit blocks, exact masks,
sigma/rho, malformed/oversized lists, successful first-stage then failed second-
stage dataset restoration/recovery and six-way mixed-plugin threading pass.
Mac missing-module probe remains separate; no physical-host performance or Linux
loader/relocation claim. Logs `/tmp/heckman-linux.log` and
`/tmp/heckman-linux-final.log`; [report](heckman.md),
[ADR 0017](../adr/0017-heckman-two-step-covariance-and-cleanup.md).
