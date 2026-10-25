# Strike duration: bounded nonparametric survival workflow

## Implemented contract

Cameron–Trivedi (2005), §17.2 pp.574–575 and §17.5.1 pp.581–583,
`mma17p1km.do`: original Kennan strike data `strkdur.asc`, 566 complete
single-spell durations in days, all failures, entry at zero, no censoring.
The exact source has 24,691 days at risk, times1..235 and141 distinct times.
GDP is preserved but is not a survival covariate.

Deliver a composed native SQL/table/export/SVG workflow, not new `stset`/`sts`
syntax. Risk sets include all events at the tied time, with one post-event
Kaplan–Meier ordinate per distinct time. Greenwood SE and pointwise95%
log-minus-log intervals are undefined at the final all-fail time and exported
as NULL, not invented zero uncertainty. Nelson–Aalen is sum(d/n), not -log(KM),
with independent-event variance sum(d/n²). No Student-t/DF correction.

Acceptance:
1. Preserve original566 source ordinals and both input columns; native output
   reproduces every141-time risk/event/survival/uncertainty/hazard record from
   development-only R `survival::survfit` and rounded author `sts list` values.
2. Produce an interpretable table/CSV and right-continuous step SVG with
   pointwise bands (Figure17.1 equivalent, not WMF/layout parity), plus a native
   Nelson–Aalen step plot.
3. Execute the actual script on an independent tied-time control; terminal/JSON
   exports agree, ties and endpoint undefined uncertainty remain explicit.
4. Reject empty/nonpositive/missing/nonfinite durations before exporting any
   result; existing output files are never replaced and stop subsequent effects.

Exclusions: no censoring/truncation/delayed entry, weights, groups, regression,
Cox baseline equivalence, smoothed instantaneous hazards, simultaneous bands or
whole-chapter claims. General survival declarations/estimators and the separate
artificial censoring example remain required. The program's comment labels the
printed table both17.2 and17.3; use author `sts list` as the exact source locator
rather than silently claiming an unverified book table number. The author index
also uses Table17.3. That numbering discrepancy remains explicit; numeric source
output is validated, not a separate book-page transcription.

## Run the original analysis

Use a fresh working directory. Download author assets yourself (they are not
redistributed in this repository):

```sh
repo=/absolute/path/to/tabdat-explore-rs
mkdir strike-analysis && cd strike-analysis
base=https://cameron.econ.ucdavis.edu/mmabook
curl -fsSLO "$base/strkdur.asc"
curl -fsSLO "$base/mma17p1km.do"
curl -fsSLO "$base/mma17p1km.txt"
sh "$repo/examples/mma17p1km/prepare.sh" strkdur.asc strikes.csv
"$repo/target/debug/tabdat-explore-rs" --text -f "$repo/examples/mma17p1km/survival.td"
```

Build the binary with `cargo build --locked` at the repository root first.
`prepare.sh` validates two numeric fields, original row/day totals, and atomically
publishes a new CSV only after successful conversion. It accepts the original
CRLF data and preserves numeric precision/order through explicit `source_row`.
These shape checks alone are **not an authenticity certificate**; the source audit
below pins all three downloaded original SHA256 fingerprints.

The native script checks unique positive integer source ordinals and nonempty
complete positive finite durations before any export; it never deletes missing
or invalid observations. GDP does not determine the sample. It preserves the
original source columns/order in `strikes_source.csv`, displays/exports the full
141-time table in `strikes_survival.csv`, exports283 step coordinates in
`strikes_steps.csv`, and writes `strikes_survival.svg` plus `strikes_cumhaz.svg`.
The SVGs are native scatter-with-step-line overlays: point marks are also visible,
not literal Stata line-only rendering. The survival graph includes a (0,1) origin;
all step ordinates are post-failure, with tied-time vertical jumps. Its two
pointwise bands terminate at the last defined pre-terminal value, not at an
invented terminal bound. The cumulative hazard graph supplies point estimates,
not the author's extra confidence-band graph. Its SE is available in the CSV.
Finite cumulative-hazard endpoint does not imply nonzero terminal KM survival.

Execution uses ordered script effects, **not a multi-file transaction**. Existing
CSV/SVG files are never replaced; a collision stops later commands but does not
roll back earlier successful exports. Delete outputs intentionally or choose a
fresh directory before rerunning. The active dataset becomes the step-coordinate
table, not the source data. No retained survival-estimation state is installed.
Existing SVG limits apply (at most10,000 candidate rows); the source has283.
This bounded script is not advertised as an unrestricted positive-duration API.

## Independent reference and observed evidence

Development-only commands (R/Python are not runtime dependencies):

```sh
Rscript "$repo/examples/mma17p1km/reference.R" strikes.csv reference.csv
python3 "$repo/examples/mma17p1km/check_source.py" \
  strkdur.asc mma17p1km.do mma17p1km.txt . reference.csv
```

Reference: R4.6.1, survival3.8.6, `survfit(Surv(dur,1)~1)` with explicit
`stype=1`, `ctype=1`, `conf.type="log-log"`, level.95. `summary()` supplies
SE(S), not the raw fit's SE(log S). At S=0 the workflow follows the author table's
undefined SE/CI convention rather than R summary's zero SE. All141 source event
times, complete risk sets and ties are compared; native/R numeric tolerance is
1e-11 absolute +1e-10 relative. Published log facts use only half a printed
four-decimal unit (0.00005); integers and missing endpoint entries are exact.
The Python audit additionally reconstructs empirical survival, risks, Greenwood
and Nelson–Aalen from source durations, constructs each step independently from R,
and checks actual SVG raw ordinates **and rendered pixel coordinates** (<.001px).

Current original-source audit passes:566 original rows/all1,698 numeric source
cells,1,269 R table cells,1,128 printed facts,283 step records and1,130 finite SVG
line vertices. First-time survival556/566=.982332155477032; final KM0 at day235,
NA6.87144577672354 (not infinity). Original JSON/text output files are
byte-identical in the observed source run. Raw original fingerprints:

- `strkdur.asc`: `cf5ed7b73d0c08c5295493720bc2b8d2f271ad752b922acfc893f6fc4a3f8059`
- `mma17p1km.do`: `4e27dde09e230f369f8fb17f7e7b459ce63e39a40153431e58900700767c253f`
- `mma17p1km.txt`: `e0621037ec861ad4cf891e8bce77263e6f9aaa9413e60b3412b1edea4e115d50`

Permanent integration target `mma17_strike_survival_workflow` executes the actual
script on a separate unsorted8-spell/5-time R fixture and a3-spell all-tied
closed-form control. It checks both CLI modes, every numeric/step/SVG record,
undefined endpoint SE/CI, corruption rejection (including shared wrong native/R
survival), malformed input, every output-collision stopping boundary, CRLF
conversion and failed-conversion/no-overwrite behavior. The synthetic fixture is
not a replacement for original source evidence. Regenerate its reference with:

```sh
Rscript examples/mma17p1km/reference.R \
  tests/fixtures/strike_survival/input.csv /tmp/strike-reference.csv
```

Affected root-package validation passes368 tests across86 current binaries,
including all four new tests. The current Rust target passes Clippy with
`-D warnings`; changed Rust formatting and diff checks pass. Two broader nextest
attempts timed out during discovery (not passing runs). A sampled process was
stalled at macOS `_dyld_start` before test-harness entry. SHA256-identical copies
of the exact Cargo-inventoried binaries executed successfully; all original
artifact hashes were reconciled afterward. This is local macOS evidence, not CI,
other-platform or release certification. No production crates changed, so the
unaffected non-root workspace and Rust doctests were not rerun.

No source uncertainty discrepancy is hidden by tolerance changes. General
survival declarations, Cox/parametric models, Cox-baseline equivalence and
remaining illustrations stay required. A separate [bounded artificial
right-censoring workflow](../mma17p2kmextra/README.md) now validates the author's
80-spell example; it does not install general survival-estimation syntax.
