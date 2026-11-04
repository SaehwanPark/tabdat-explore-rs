# Chapter21 per-draw component-refitting RE-GLS bootstrap

**Validated bounded native SQL composition**, not a general resampling API.
This covers C21-P1-REGLS-BOOT in the [first-program inventory](inventory.md):
`mma21p1panfeandre.do` lines285–287/Table21.2. It is **different from** the
[fixed-theta transformed-OLS bootstrap](README.md#fixed-transform-re-gls-whole-person-bootstrap),
which remains unchanged and reports a transformed-scale intercept.

## Sample, model and resampling

Original MOM.dat SHA256:
`2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77`.
Use the [existing conversion](README.md#data-and-exact-samples), not Chapter22's
more precise source. Original N5,320/G532/T10, years1979–1988; unweighted lnhr on
intercept and lnwg, no other controls. Author data/program/log are external and
not redistributed. Source access does not establish redistribution permission.

[`random_effects_gls_refit_bootstrap.td`](random_effects_gls_refit_bootstrap.td)
guards raw person/year/source-row keys before fitting. Modified input requires
3–532 people, each with all ten distinct years1979–1988, unique source rows
in1..5320, finite model values with absolute magnitude<=20 and exact bounded
integer keys. There is no missing-case deletion, imputation, gap bridging or
unbalanced-panel support. Source rows may be interleaved or time-reversed;
within-person moments use year order and the sampler indexes sorted IDs.
Ancillary columns are preserved, not included in the model.

Fit native Swamy–Arora RE-GLS once, then draw G **whole people with replacement**
for each of B500 replications. Repeated copies of the same source person count
as separate people: frequency weights contribute multiplicity to the sample,
grand means and degrees of freedom. Both variance components and theta are
re-estimated on **every** draw. This is neither fixed-theta resampling nor a
Gaussian marginal-ML bootstrap.

The base seed is10001. Replicate r starts an independent xoshiro256++ stream,
SplitMix64-expanded from seed10001+r−1, using the declared fixed-width u64
rejection convention:
`xoshiro256++/splitmix64/per_replicate_seed_plus_index/u64_rejection_v1`.
Rejection work is bounded at G+1024 steps/replicate. Every accepted draw exports
replicate/copy position/RNG step/original person ID. This matches the declared
composition convention used by the separate RE-ML workflow, **not** the OLS
bootstrap command's single continuing stream or historical Stata draws.

## Per-draw estimation and inference

For each copied panel:

1. Fit the within slope and equal-copy-weighted Between regression on person
   means. Residual variance is within RSS/(N−G−1); raw unit variance is
   Between RSS/(G−2) minus residual variance/10.
2. Set unit variance=max(0,raw unit variance), retaining its signed raw value in
   the audit. Negative raw estimates are the Swamy–Arora zero truncation,
   **not** an ML boundary test or a reason to drop a replication.
3. Set q=e/(e+10*u)=(1−theta)². Fit the wage slope from
   `(Wxy+10*q*Bxy)/(Wxx+10*q*Bxx)`, with W within cross-products and B centered
   person-mean cross-products. The population intercept is resampled grand
   mean(y)−slope*grand mean(x), **not** a quasi-transformed OLS intercept.
4. Sum raw squared residuals directly for preliminary components and final
   transformed RSS. This avoids subtracting nearly equal quadratic forms when
   a Between fit is nearly exact. Repeated copies have separate panel roles,
   even though weighted sums avoid physically expanding all copied records.

Report full B−1 coefficient covariance, SEs, replicate means/bias and fixed95%
R-type7 percentile intervals. No bootstrap p-values, component-bootstrap
intervals, normal/bias-corrected/studentized/BCa intervals or finite-sample/
few-cluster coverage guarantee is supplied. Independent-person sampling and
suitable mean/exogeneity assumptions still matter; resampling does not make a
wage effect causal or repair mean-model misspecification.

Admission requires finite within/between moments, within wage/response and
between wage variation>1e-12, residual component>1e-12, nonnegative finite
Between residual variance, finite raw/truncated unit components, theta in[0,1),
positive finite q and transformed RSS>1e-12, and final coefficient magnitudes<=100.
Original composed coefficients must match the retained native fit within1e-8.
These are conservative bounds for this fixed balanced scalar design, not a
new general rank/variance policy. There is no iterative optimizer to converge;
any unidentified/invalid draw aborts without redraw or silent deletion.

## Terminal, structured output and failure behavior

For a terminal-first econometrics user, the native original fit is followed by
a compact `_cons`/`lnwg` estimate/bootstrap-SE/percentile95 table; the same table
is a CLI-v1 `TableResult`. Users need not open the full draw file to interpret
coefficient uncertainty. Full reports are exported as:

- `panel_re_gls_refit_bootstrap_summary.csv`: B/G, original coefficient points,
  bootstrap means/bias, full covariance/SEs/intervals and method/stream labels.
- `panel_re_gls_refit_bootstrap_draws.csv`: G×500 ordered copies and all replicate
  coefficients, signed/truncated components, theta, preliminary slopes,
  transformed RSS and original coefficient points (about64MB on original MOM).
- `panel_re_gls_refit_bootstrap_predictions.csv`: all original fields/source rows
  plus retained original population `regls_xb` and residuals in original order.

All500 fits must pass before any bootstrap export. SQL failures identify the
script line, not an individual failing replicate number. Exports are ordered
summary→draws→predictions and refuse replacement. This is **not** a whole-script
transaction: a later collision leaves earlier successful exports and preserves
the existing target bytes. Native prediction columns/SQL relation activation
change session-local data as usual; earlier session changes are not rolled back
on a later error. Source files are never rewritten. Reloading MOM.csv at the end
and predicting without refitting verifies that the original native RE-GLS model
remains retained, not the last resampled model or transformed OLS state.

## Reproduce

Build the current CLI as described in the [main workflow](README.md#reproduce).
Prepare external MOM.dat/MOM.csv under `$work`, with source checksum verified;
use fresh run/reference directories because outputs never replace files:

```sh
root=$(pwd)
mkdir "$work/regls_refit_json" "$work/regls_refit_text"
cp "$work/MOM.csv" "$work/regls_refit_json/MOM.csv"
cp "$work/MOM.csv" "$work/regls_refit_text/MOM.csv"
(cd "$work/regls_refit_json" && "$root/target/debug/tabdat-explore-rs" --json -f "$root/examples/mma21p1panel/random_effects_gls_refit_bootstrap.td" > output.jsonl)
(cd "$work/regls_refit_text" && "$root/target/debug/tabdat-explore-rs" --text -f "$root/examples/mma21p1panel/random_effects_gls_refit_bootstrap.td" > output.txt)
Rscript "$root/examples/mma21p1panel/random_effects_gls_refit_bootstrap_reference.R" "$work/MOM.dat" "$work/regls_refit_reference"
python3 "$root/examples/mma21p1panel/check_random_effects_gls_refit_bootstrap.py" "$work/regls_refit_json" "$work/regls_refit_reference" "$work/MOM.dat"
for kind in summary draws predictions; do
  cmp "$work/regls_refit_json/panel_re_gls_refit_bootstrap_$kind.csv" "$work/regls_refit_text/panel_re_gls_refit_bootstrap_$kind.csv"
done
```

## Independent evidence and tests

R4.6.2/plm2.6.7 independently expands every raw copied panel with **fresh copied
unit IDs**, fits `plm(...,model='random',random.method='swar')`, within and Between
preliminary regressions, and checks each coefficient against a separate direct
transformed-design QR. It audits all501 original/replicate fits, components,
theta, preliminary slopes and transformed RSS. Python's independent unsigned-
integer oracle checks every draw/index/rejection step and copy order. Neither
reference calls native fits or reuses SQL moments. References serialize17 digits;
no R/Python dependency is added to production.

Original MOM passes **266,000 exact draws**, all**500 R/plm refits** (nine fields
each), **17 summary fields**, **5,320 predictions** and **42,560 exact raw source
fields** in native and reference projections. All ordinary numerical comparisons
use unchanged absolute1e-9; maximum coefficient/component/theta/RSS differences
are1.4e-14/6.7e-16/2.4e-15/2.9e-12. Original intercept/slope are
7.346040587061543/.11933224394062582; bootstrap SEs are
**.13806309014098955/.05136516797629342**. JSON/text CSV bytes are identical,
and the actual native readable uncertainty table was inspected. These are
native-stream results, not the author's historical printed bootstrap SEs.

Two sixteen-person/T10 controls add**1,000 R/plm refits** and**16,000 exact
draws**: all501 interior-control fits are interior; all501 zero-unit-control fits
retain raw negative components and truncate to unit variance/theta0. Shared
six-decimal input fixtures/generator from the RE-ML controls are reused as data
only; new GLS reference outputs are independently regenerated and frozen in
`tests/fixtures/mma21_regls_refit_bootstrap/`. For example:

```sh
python3 examples/mma21p1panel/random_effects_ml_bootstrap_synthetic.py interior "$work/control"
Rscript examples/mma21p1panel/random_effects_gls_refit_bootstrap_reference.R "$work/control/MOM.dat" "$work/control_reference"
(cd "$work/control" && "$root/target/debug/tabdat-explore-rs" --json -f "$root/examples/mma21p1panel/random_effects_gls_refit_bootstrap.td" > output.jsonl)
python3 examples/mma21p1panel/check_random_effects_gls_refit_bootstrap.py "$work/control" "$work/control_reference" "$work/control/MOM.dat" --synthetic
# Repeat in fresh paths with boundary; replicate_rank_failure is a rejection control.
```

Four new actual-script tests compare all frozen refits, full summaries, native
original components/theta/RSS, typed/readable uncertainty rows, original source/
order/predictions, repeated copied people and JSON/text exports. They also test
missing/duplicate/incomplete input, an identified original followed by a failed
bootstrap draw, and all three output-collision stages. Both existing fixed-theta
workflow tests still pass. Seven corruption probes reject changed person ID,
coefficient, theta, signed unit variance, covariance, method label or raw source
value. Scoped Clippy, formatting, syntax/link and diff checks pass. No shared
kernel/parser/runtime/API change, whole-workspace/platform/release claim or
whole-table/chapter completion follows. The separate [Gaussian population-averaged
GEE composition](gaussian_pa.md) now validates a bounded coefficient/covariance/
normal95 target; FE normalized-intercept uncertainty and historical parity remain
required work.
