# Native patents Poisson GEE

A bounded native load/prepare → fit → compare working/group-sandwich inference →
owned prediction → report/export companion for Cameron–Trivedi (2005)
§23.3/Table23.1. Unlike the [SQL composition](exchangeable_gee.md), the current
estimator really is **Poisson GEE**, not its internal pooled Poisson start.
The guarded source workflow, independent checks and full current workspace
regression pass in [PR#435](https://github.com/SaehwanPark/tabdat-explore-rs/pull/435).
No whole-table/book claim.

## Reproduce

Build the ordinary Rust CLI; Python and R are optional development auditors,
not production dependencies. From the repository root, with the author's
`patr7079.asc` available locally:

```sh
work=$(mktemp -d)
sh examples/mma23p1pannonlin/prepare.sh /path/patr7079.asc "$work/patents.csv"
bin="$PWD/target/debug/tabdat-explore-rs"
script="$PWD/examples/mma23p1pannonlin/native_poisson_gee.td"
(cd "$work" && "$bin" --json -f "$script" > result.jsonl)
# Optional independent R oracle; needs development-only gee4.13-30.
Rscript examples/mma23p1pannonlin/exchangeable_gee_reference.R \
  /path/patr7079.asc "$work/reference"
python3 examples/mma23p1pannonlin/check_native_poisson_gee.py \
  "$work" "$work/reference" /path/patr7079.asc
```

Use `--text` for readable reporting, in a separate fresh directory because
`native_poisson_gee_predictions.csv` is never overwritten. Reference generation
also requires a fresh destination. Controls use `--control` as the auditor's
fourth argument. Original ASCII SHA256:
`87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8`.

## Exact sample and files

All **1730 observations/346 firms**, years1975–79, remain, including all22
zero-total firms and every record with missing nonmodel ARDSSIC. Mean is
exp(intercept+slope*LOGR), with no weights, exposure, year effects or added
covariates. Firm ID is the original source ordinal, **not CUSIP**.

`prepare.sh` supplies the canonical nine-field CSV
`id,year,cusip,ardssic,scisect,logk,sumpat,logr,pat`. The source script checks
unique positive BIGINT IDs<=346, BIGINT years1975..79, all five distinct years
per loaded firm, finite integer PAT0..1000, finite |LOGR|<=20 and finite other
metadata except nullable ARDSSIC. SUMPAT stays source metadata; it does not
select rows. Extra prediction-name columns fail without being overwritten.
At least three firms are needed by the native G>parameter-count requirement;
the source row guard alone is not a successful-fit guarantee.

Output has the original nine fields followed by:
`selected_firm_total,included_in_native_gee,native_gee_mean,native_gee_xb,native_gee_residual`.
Rows are ordered by ID/year, membership is true on every source row, and totals
are recomputed from the selected five years, not copied from SUMPAT. Final active
table is `native_poisson_gee_cohort`. Source CSV bytes/types/order are preserved
where promised; exports normalize numeric spelling without changing values.
The script is ordered, **not whole-transactional**: later errors do not roll back
earlier named-table/fit/prediction work. Existing export owners are preserved.

## Model and statistical conventions

Generic syntax:

```
xtgee y xvars, family(poisson) link(log) corr(exchangeable) id(group)
predict mean, n
predict index, xb
predict residual, residuals
```

For each group, eta=X beta, mu=exp(eta), e=(y−mu)/sqrt(mu). The explicit
legacy balanced Pearson moment is

```
rho = (sum_g (sum_t e_gt)^2 − sum_gt e_gt^2) / ((T−1)*sum_gt e_gt^2).
```

Pearson RSS/N is only a **correlation normalizer**; working variance scale is
**fixed1**, not RSS/N or RSS/(N−K). R=(1−rho)I+rho*11', A=diag(mu),
V=sqrt(A) R sqrt(A), D=A X. Group score U_g=D_g'V_g^-1(y_g−mu_g),
bread B=sum_g D_g'V_g^-1D_g. This is expected GEE sensitivity, **not** an
observed likelihood Hessian or an arbitrary observed derivative of U.

Full covariances, coefficient z/normal95 and slope-Wald chi-square inference:

- Working: B^-1, assuming the working variance/correlation are appropriate.
- Group sandwich: G/(G−1)*B^-1(sum_g U_g U_g')B^-1, requiring independent
  groups and correct marginal mean, not a correctly specified joint count law.

No extra (N−1)/(N−K), G−1 t reference, rho SE, joint likelihood/LR/R², survey,
few-group reliability or causal interpretation. Both covariance choices are
reported separately; neither is silently selected. Gaussian GEE's distinct
RSS/N/pair moment and **uncorrected** sandwich remain unchanged.

Uses an internal pooled Poisson start, RMS-scaled design, compensated moments,
symmetric whitening (including negative rho) and full QR expected-Fisher steps.
No damping, clipping, ridge, pseudoinverse or failed iterate as a fit. Fit eta
must stay[-40,40], at most100 full steps, final relative RMS-coordinate step
<=1e-12 and score<=1e-10; score is coordinatewise divided by one plus absolute
whitened row contributions. Nonpositive/near-SPD-boundary correlation gaps
<=128*EPSILON, zero/roundoff-dominated Pearson or sandwich variance, rank failure,
escaped/nonfinite updates or nonconvergence fail conservatively. Convergence
verifies the bounded local estimating-equation root, not uniqueness or global
optimality.

## Supported boundaries and state

The generic native fit has intercept/1..32 numeric slopes, no weights/offsets,
selected integer counts0..100000, at most40000 candidate rows/10000 raw groups,
candidate rows*(slopes+1)^2<=20million and selected G>slopes+1. Joint-finite
outcome/predictor/group rows are selected first; **selected groups must share
T>=2**. Missingness is not a request to drop incomplete firms until balanced.
The source workflow is stricter: no missing model fields, all five years remain.
The current source guards are not a general data-cleaning or calendar-time API.

Current predictors, not current IDs, drive owned marginal n/xb. Residuals also
need current y. Missing/nonfinite predictors produce NULL output; an
unrepresentable positive mean/residual fails before adding any destination
column. No posterior firm effects, count PMF/CDF, intervals/std/saved draws,
named estimate snapshots/covariance selectors, generic post-estimation or native
automatic GEE bootstrap. The existing SQL bootstrap remains separate.

A failed **new Poisson GEE fit clears stale estimation** before validation but
preserves data/labels/stored reports. Parser/prediction/store errors preserve the
current fit and registry. Legacy Gaussian GEE deliberately keeps its prior fit
on failure; shared data preparation does not change that existing exception.
Old Gaussian command/family wires stay unchanged; Poisson uses additive typed
command/result/state/family variants with the same canonical command name.

## Independent evidence and limits

Original converged native point: intercept2.06751524798454689,
slope.55953007037554769, rho.89836667589854957;16 full steps, scaled
score4.57e-13/step6.33e-13. Independent R4.6.2/gee4.13.30 uses explicit pooled
starts, v4.4compat=TRUE, scale.fix=TRUE/scale.value=1, tol1e-12/maxiter100.
Modern/default moment conventions are not substituted.

Local final regression: `cargo nextest run --locked --workspace --all-targets
--no-tests fail` passes2105 tests across393 binaries with two pre-existing skips;
new/affected focused checks, scoped Clippy and fmt/diff also pass. No Rust doc
examples changed; doctests were not rerun. CI is disabled; this is local evidence.

The unchanged R oracle and separate Python equations check27 report fields,
both full covariances, all1730 source/prediction records and346 firm scores.
Normal/Wald tails, SE/CI, rho/normalizer, score root and inverse working bread/
group-sandwich identities are independently recomputed. Fixed scalar gates are
1e-9+1e-10*abs(reference), group scores1e-8+1e-10*abs(reference), nonzero p
relative1e-8. No tolerance was widened. Original/control JSON/text exports are
byte-identical. A supplementary direct-Rust/base-R probe also checks six
R-derived normal/Wald tails at the same predeclared gate.

[Dense multivariate/rescaling controls](../../crates/tabdat-stats/tests/fixtures/poisson_gee/README.md)
were frozen **before kernel implementation**:60 rows/12 groups/three parameters,
66 quantities/full3x3 working and sandwich matrices, every prediction/group score.
R package and independent dense block-V equations agree; natural rescaling and
permutation hold. These are numerical controls, not extra book regressors.

Eight stats integration/three private guard cases, six runtime cases, additive
wire/paired-parser tests and five actual-script Rust cases cover missing samples,
balance, count/weight/rank/resource/iteration failures, near/exact correlation
bounds, roundoff, actual owned predictions without IDs/current y as appropriate,
labels/stored reports, source/export/prediction collisions and26 invalid/singular
sources. Python2/2 includes real positive/negative-rho/permuted/all-null-metadata
controls and33 rejected forgeries, including shared wrong native/R covariance,
SE/z/p/CI, mean and rho. Source/moment/information equations reject common
corruptions rather than accepting native/reference equality as authority.

The author's printed slope **.5595302** differs from converged float64
.5595300704. Existing nine-step/float32 probes support stopping/storage as
contributors, not exact historical internals. Historical bootstrap SE/repeated-ID
conventions, remaining illustrations, native unbalanced/general GEE and the full
book inventory remain required. No whole-table/chapter/graduate-core completion,
independent code review, release or platform-support claim.

Optional development archive gee4.13-30 SHA256:
`65f9fe9c906a200bd89a6c053e8c33cce356b626a0de639cc69d804bc3326b95`.
On this host R4.6.2 FLIBS pointed at Darwin27 but gfortran16.2 private libraries
resolved under Darwin25, causing missing emutls_w. Compare `R CMD config FLIBS`
with `gfortran -print-file-name=libemutls_w.a` / `libgfortran.dylib`; validate the
files exist and use compiler-resolved directories in a temporary per-install
R_MAKEVARS_USER Makevars. That installation and all R oracles pass without global
Makeconf/package edits. Do not generalize this fix to arbitrary linker failures
or hardcode host paths in production. No R/Fortran dependency ships in the app.
