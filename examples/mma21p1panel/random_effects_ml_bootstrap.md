# Chapter21 RE-ML whole-person bootstrap

**Validated bounded native SQL composition**, not a general `bootstrap` or
`xtreg` resampling API. This covers the coefficient-bootstrap target in the
[first-program inventory](inventory.md), C21-P1-REML-BOOT, author program
`mma21p1panfeandre.do` lines301–303. It does not reproduce historical Stata draws,
its duplicate-ID handling, all printed interval conventions or the whole table.

## Sample, design and inference contract

Original MOM.dat SHA256:
`2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77`.
Use the [existing converter](README.md#data-and-exact-samples); do not substitute
Chapter22's more precise data. Original N5,320/G532/T10, years1979–1988;
unweighted lnhr on intercept and lnwg. Source access is not redistribution
permission: author data/program/log are external, not bundled.

[`random_effects_ml_bootstrap.td`](random_effects_ml_bootstrap.td) checks raw
person/year/source-row keys before fitting. Modified input requires 3–532 people,
each with all ten years1979–1988, unique source rows in1..5320, finite model
values with absolute magnitude<=20 and exact bounded integer keys. There is no
missing-case deletion, imputation or gap bridging. Source order may be
interleaved; within-person moments use year order and draws index sorted IDs.
The synthetic controls use six-decimal inputs, deliberately interleaved people
and reversed times. Ancillary columns are preserved, not used by the model.

Fit native Gaussian marginal RE-ML once and retain it. Then draw G **whole people
with replacement** for each of B500 replications; each copy contributes its own
ten-row likelihood and determinant, including repeated original IDs. SQL
multiplicity weights implement these independent copies without combining them
into a longer panel. Both residual and unit variances are re-estimated on every
draw: this is not fixed-theta quasi-OLS resampling. Zero-unit-variance replicates
are retained as constrained point fits, not silently discarded or redrawn.

The base seed is10001. Each replicate r initializes its own xoshiro256++ stream
using SplitMix64 seed10001+r−1 and the existing fixed-width u64 rejection map:
`xoshiro256++/splitmix64/per_replicate_seed_plus_index/u64_rejection_v1`.
This matches the declared Chapter23 composition convention, not the OLS
bootstrap command's single continuing stream or historical Stata RNG. A seed
number alone does not establish identical draws. Rejections are bounded at
G+1024 steps per replicate; every accepted draw is exported with copy position,
original person and RNG step.

Coefficient uncertainty is the full B−1 replicate covariance, SEs, replicate
means/bias and fixed95% R-type7 percentile intervals. No analytic p-values,
bootstrap variance-component intervals, normal/bias-corrected/studentized/BCa
intervals or nominal/few-cluster coverage guarantee is supplied. Interpretation
requires independent people and a suitable marginal mean/model target; neither
resampling nor RE assumptions establish causal wage effects.

## Profile fitting and failure semantics

For each replicate (and original comparison), center person means on the
resampled grand means and retain summed within moments. At
eta=log(1+10*sigma_u²/sigma_e²), q=exp(−eta), fit the conditional mean using
within+10*q*between cross-products. Profile residual variance by RSS/N and use
the full marginal Gaussian determinant G*eta. Search eta0:.5:24 for
positive-to-negative profile-score crossings; bisect each bracket for48 steps,
include the zero-unit-variance boundary and select the largest likelihood.
**Search endpoints are DOUBLE**, not fixed-scale DECIMAL: recursive decimal
endpoints otherwise round instead of converging.

Require positive identified within/between wage variation and within response
variation (>1e-12), final residual variance/within RSS>1e-12, finite bounded
coefficients (absolute<=100), nonnegative between RSS/unit variance and an
unresolved-upper-bound rejection. Interior profile score must be within1e-8
and observed profile information>1e-8; the boundary requires nonpositive inward
score up to1e-8. Original composed coefficients must match the retained native
fit within1e-8. These are deliberate conservative admission bounds for this
balanced scalar-design composition, not general rank/variance thresholds.
The finite grid does not prove global optimality between grid points. No full
per-replicate information-matrix inference is claimed; profile information is a
numerical regularity certificate, not the bootstrap coefficient covariance.

Any failed replicate aborts the SQL operation without dropping/redrawing it.
No bootstrap exports are published before all500 fits pass. Errors identify the
script line; the current generic SQL error surface does not expose a failing
replicate number. Exports are ordered and **not a whole-script transaction**:
summary then draws then original predictions. A later output collision preserves
the existing file, stops later commands and leaves earlier successful exports.
Reloading MOM.csv at the end and predicting without refitting exercises retained
original native ML state, not the last bootstrap fit. Source files are never
rewritten. Predictions and SQL activation change session-local relations as
usual; the composed refits do not replace the retained native model. There is
no promise to roll back earlier session changes after a later script failure.

## Reproduce on original data

Build the current CLI as described in [the main workflow](README.md#reproduce).
From the repository root, prepare an external work directory containing MOM.dat
and MOM.csv, then run in fresh directories (outputs never replace files):

```sh
root=$(pwd)
# work must already contain the downloaded, checksum-verified MOM.dat and MOM.csv.
mkdir "$work/reml_json" "$work/reml_text"
cp "$work/MOM.csv" "$work/reml_json/MOM.csv"
cp "$work/MOM.csv" "$work/reml_text/MOM.csv"
(cd "$work/reml_json" && "$root/target/debug/tabdat-explore-rs" --json -f "$root/examples/mma21p1panel/random_effects_ml_bootstrap.td" > output.jsonl)
(cd "$work/reml_text" && "$root/target/debug/tabdat-explore-rs" --text -f "$root/examples/mma21p1panel/random_effects_ml_bootstrap.td" > output.txt)
Rscript "$root/examples/mma21p1panel/random_effects_ml_bootstrap_reference.R" "$work/MOM.dat" "$work/reml_reference"
python3 "$root/examples/mma21p1panel/check_random_effects_ml_bootstrap.py" "$work/reml_json" "$work/reml_reference" "$work/MOM.dat"
for kind in summary draws predictions; do
  cmp "$work/reml_json/panel_re_ml_bootstrap_$kind.csv" "$work/reml_text/panel_re_ml_bootstrap_$kind.csv"
done
```

Outputs:
- `panel_re_ml_bootstrap_summary.csv`: original coefficient points, B500/G,
  mean/bias/full coefficient covariance/SEs/type7 intervals and stream/method labels.
- `panel_re_ml_bootstrap_draws.csv`: G×500 ordered draws, copy identities,
  replicate coefficients/variances/eta/likelihood/score/profile information and
  original coefficient points. Original variance/likelihood results also appear
  in the native ML terminal/JSON result before resampling.
- `panel_re_ml_bootstrap_predictions.csv`: every original column/source row plus
  retained original population xb and residuals, in original order.

## Independent evidence and tests

R4.6.2 (2026-10-29) independently builds every copied raw panel, profiles the
full marginal likelihood through dense transformed-design `lm.fit` QR and
`uniroot`, and checks **all501 likelihoods** against direct ten-by-ten covariance
Cholesky calculations. It does not reuse SQL sufficient moments or native fits.
Profile information is separately checked using Richardson-extrapolated central
score differences at1e-4/5e-5 (including the smooth extension at the variance
boundary; not boundary uncertainty). Python's independent unsigned-integer
oracle checks every draw, including rejection steps and copy order.

On original MOM, the checker passes all**266,000 exact draws**, all**500 R
refits** (coefficients, components, eta, likelihood, score and profile-information
certificates), **17 summary fields**, all**5,320 prediction rows** and**42,560
exact raw fields** in both native and reference projections. Absolute ordinary
numerical tolerance is1e-9; numerical-derivative information tolerance is2e-6.
Maximum coefficient/component/eta differences are2.4e-14/7.7e-15/2.3e-13;
maximum likelihood discrepancy is2.1e-11. No tolerance was widened. Original
intercept/slope are7.345479144924077/.11954740235725428; bootstrap SEs
are**.14095503380052563/.05247476063850445**. JSON/text CSV bytes are identical.
These are native-stream results, not the author's printed bootstrap SEs.

Two independent16-person/T10 controls add**1,000 R refits**, **16,000 exact
draws** and direct covariance-likelihood certificates: every interior control
fit is interior; all501 boundary-control fits (including original) have zero
unit variance. R references use17-digit serialization so raw input values are
not silently shortened by `write.csv`. Generator and frozen small fixtures are
bundled; no author data are. Regenerate, audit and compare fixtures with:

```sh
python3 examples/mma21p1panel/random_effects_ml_bootstrap_synthetic.py interior "$work/synthetic"
Rscript examples/mma21p1panel/random_effects_ml_bootstrap_reference.R "$work/synthetic/MOM.dat" "$work/synthetic_reference"
(cd "$work/synthetic" && "$root/target/debug/tabdat-explore-rs" --json -f "$root/examples/mma21p1panel/random_effects_ml_bootstrap.td" > output.jsonl)
python3 examples/mma21p1panel/check_random_effects_ml_bootstrap.py "$work/synthetic" "$work/synthetic_reference" "$work/synthetic/MOM.dat" --synthetic
# Repeat in fresh paths with boundary; replicate_rank_failure is a rejection control.
```

Four actual-script integration tests in `tests/mma21_reml_bootstrap_workflow.rs`
compare every frozen synthetic refit and all summary fields, original native
components/likelihood, exact source/order, retained predictions, repeated copied
people and JSON/text bytes. Failure cases cover duplicate/missing/incomplete
input, a valid original fit followed by an unidentified bootstrap draw, and
all three no-clobber export stages. Six audit-corruption probes reject a changed
person ID, coefficient, profile information, covariance, interval label or raw
source value. Existing RE-ML workflow integration checks also pass. No production
Python/R dependency, shared estimator/API change, whole-chapter, platform or
release certification is introduced. Component-refitting RE-GLS bootstrap,
FE level-intercept uncertainty, Gaussian PA and historical parity remain separate
required work.
