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
