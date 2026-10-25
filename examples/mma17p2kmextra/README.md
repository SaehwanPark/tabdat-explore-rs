# Artificial right-censoring: bounded survival workflow

## Implemented contract

Cameron–Trivedi (2005), §17.5.1 pp.581–582, author March2005
`mma17p2kmextra.do`, Table17.2 KM/NA portions. Extract its exact explicit input:
80 single-record spells, entry zero,40 failures/40 right-censored exits,3170 days
at risk,9 distinct exit times10..50. `failed=1` means failure. Comments incorrectly
call the32 exits at45 failures and the26 exits at50 censored; the executable
input and printed results unambiguously specify the opposite. Follow the executed
sample, not those comments. No replacement random generation.

Acceptance:
1. Preserve all80 source ordinals/duration/event cells; match independent R
   survival and both author printed KM/NA tables, including uncertainty.
2. Native table/CSV and right-continuous step SVGs carry estimates across
   censor-only times; at tied event/censor times the risk set includes both before
   failures, then removes both. Censoring is not treated as a failure.
3. The actual script also matches an independent control with an initial censor,
   a failure/censor tie and a censored tail; KM can end above zero. A terminal
   all-failure risk set yields KM0/undefined KM SE/CI, not invented certainty.
4. Reject empty/missing/nonfinite/nonpositive times, nonbinary/missing event flags,
   duplicate/invalid source keys and zero observed failures before any export.
   Source conversion is atomic; output collisions preserve existing files and
   stop later script effects, without pretending all files form one transaction.

Estimator: KM product(1-d/n), Greenwood sum d/[n(n-d)], pointwise95% log-log
intervals; NA sum d/n with variance sum d/n² and positive log-Wald95% intervals.
Before any observed failure use S1/SE0/bounds1 and H0/SE0/bounds0. No DF correction.
Interpretation assumes independent noninformative right-censoring and independent
spells; it is not a causal guarantee or proof of survival beyond observation.

Exclusions: no delayed entry/left or interval censoring/truncation, weights,
groups, regression/Cox/parametric models, simultaneous bands, retained fit state,
new `stset`/`sts` syntax, or full survival-family/book coverage. Native SVG layout
is not Stata WMF parity. This is an author artificial-data illustration, not an
empirical dataset or a substitute for required empirical survival regressions.

## Run and audit the original input

Use a fresh working directory; original copyrighted programs/logs are downloaded
by the contributor, not redistributed here:

```sh
repo=/absolute/path/to/tabdat-explore-rs
mkdir censored-analysis && cd censored-analysis
base=https://cameron.econ.ucdavis.edu/mmabook
curl -fsSLO "$base/mma17p2kmextra.do"
curl -fsSLO "$base/mma17p2kmextra.txt"
sh "$repo/examples/mma17p2kmextra/prepare.sh" mma17p2kmextra.do censored.csv
"$repo/target/debug/tabdat-explore-rs" --text -f "$repo/examples/mma17p2kmextra/survival.td"
Rscript "$repo/examples/mma17p2kmextra/reference.R" censored.csv reference.csv
python3 "$repo/examples/mma17p2kmextra/check_source.py" \
  mma17p2kmextra.do mma17p2kmextra.txt . reference.csv
```

Build the binary first with `cargo build --locked` at the repository root.
Only reference generation/auditing require R/Python; native execution does not.
The strict extractor reads exactly one `input duration failed` block, accepts CRLF,
checks80 spells/40 failures/3170 days, preserves order with explicit `source_row`,
and atomically publishes a new CSV via a completed temporary file/hard link.
These checks establish shape, not authenticity: the source audit additionally
pins raw SHA256 bytes and verifies every input cell against the executable block.

The script rejects invalid input rather than doing complete-case deletion. It
requires unique positive integer source ordinals, numeric positive finite complete
times, exact numeric0/1 flags, at least one failure, and at most4999 distinct
times (so at most9999 native SVG vertices per series, within the existing10,000-row
plot bound). GDP/covariates are not part of this artificial sample. It exports
`censored_source.csv`, the full9-time `censored_survival.csv`,19 pre/post/origin
records in `censored_steps.csv`, and two native SVGs. State ends at the step table;
no retained survival fit is installed. At larger inputs the displayed table is a
200-row preview; the CSV still contains every distinct time.

Outputs are not replaced. A collision stops later commands but does not roll back
earlier successful file effects. Use a fresh directory or deliberately remove
outputs before rerunning. Scatter marks remain visible on the step-line SVGs.
The author program comments out its graph calls: these are explicitly **native
reporting extensions**, not claims of a historically generated/published graph.
Both plots now carry pointwise intervals; no simultaneous-band guarantee.

## Statistical and reference details

R4.6.1/survival3.8.6 `survfit(Surv(duration,failed)~1, stype=1, ctype=1,
conf.type="log-log", conf.int=.95)` supplies all times via
`summary(..., censored=TRUE)`. KM SE comes from summary's SE(S), not the fit's
SE(log S). NA point/SE are independent package outputs; positive log-Wald NA
bounds use base-R `qnorm(.975)` and those package values. These match the author
printed NA intervals, not KM-derived `-log(S)` or a normal additive interval.

A risk set includes everyone still observed at a time, including that time's
censored exits, before applying d/n. Censor-only times leave estimates and
uncertainty unchanged. Initial S1/bounds1 and H0/bounds0 are computational
conventions, not proof of no hazard. At final S0 the author table's SE/CI are
undefined: export NULL, despite R summary returning zero SE. Final NA and its
uncertainty remain finite. The hypothetical survival curve beyond last follow-up
is not identified by a last censored exit.

Original accepted source: first S=.925/H=.075; at day45 S=.817132703272671,
then26 actual failures at day50 take S to0 and H to1.195635788487073.
Native/source/R audit passes all240 source cells/ordinals,108 table cells,
144 facts across both printed tables, six explicit first-three-time manual
arithmetic gates,19 step records and112 finite line vertices/actual pixel
coordinates. All five JSON/text original artifacts are byte-identical. Numeric
R tolerance1e-11 absolute+1e-10 relative; printed four-decimal quantities get only
half a printed unit (.00005), exact integers/NULLs. No tolerance widening.

Pinned fingerprints (raw source byte identity matters, including line endings):

- `mma17p2kmextra.do`: `947e32d5c91bcc590be975be284fa4b97200a419b12a3e2c63cab520dd86cf90`
- `mma17p2kmextra.txt`: `bae7c6334c103c25a69a66f35cc99633c4e2f15aaedceb011be4755348853f80`

The separate6-spell/5-time synthetic R fixture has an initial censor, a failure/
censor tie and a nonzero censored tail:60 table cells/66 SVG vertices. It does not
replace the original80-spell evidence. Regenerate it with:

```sh
Rscript examples/mma17p2kmextra/reference.R \
  tests/fixtures/censored_survival/input.csv /tmp/censored-reference.csv
```

Permanent `mma17_censored_survival_workflow` integration tests execute the actual
script in both CLI modes, check all numeric/step/pixel records, terminal all-fail
uncertainty, invalid/zero-failure/oversized samples, every no-overwrite stopping
boundary, CRLF and malformed/repeated-input-block extraction. Corruption controls
reject changed source flags, table/steps, actual SVG geometry with unchanged data
metadata, and shared wrong native/R survival. The reusable step/SVG audit lives
beside the strike example; all four strike tests and its original566-row source
audit still pass after extraction of that helper.

Validation:8 focused tests across the two affected workflow targets pass. Full
current root coverage reconciles372 passing tests across87 artifacts:85 unchanged
SHA256 artifacts retain previous observed passes, and both changed/new targets
were rerun. Current root CLI main SHA256 is unchanged. Root all-target Clippy
`-D warnings`, changed Rust formatting and diff checks pass. No production crates
changed; unaffected non-root tests/doctests were not rerun. Actions remain disabled;
local evidence is not CI, other-platform or release certification.
