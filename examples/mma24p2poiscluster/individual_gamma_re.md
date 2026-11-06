# Chapter 24 individual shared-gamma Poisson random effects

This bounded native workflow covers the shared-gamma RE portion of the author
program for Cameron–Trivedi (2005), §24.7/Table24.6. It is not the complete
chapter/table, complex-survey inference or general Stata compatibility. Source
hashes, preparation and numeric-column interpretations are in [README](README.md).
The [conditional FE workflow](individual_conditional_fe.md), pooled comparison
and existing Chapter23 scalar SQL compositions remain separate.

## Sample, model and inference

Outcome `PHARVIS`; original numeric predictors, in order: `LNHHEXP INSURANCE
SEX AGE MARRIED ILLDAYS ACTDAYS INJURY ILLNESS EDUC`, plus an intercept. No
factor expansion, weights or offsets. Source rows identify cross-sectional
people, not temporal measurements. All27,766 publisher source records are audited;
row27,734 is entirely missing and removed before structural-key fitting. All
**27,765 people /194 communes** remain, including commune64's **94 all-zero
observations**. Commune sizes51–206; total count14,209; zero counts20,668.
Unlike conditional FE, zero-total groups must not be dropped.

Groups are independent, and group effects are assumed independent of the
regressors. Conditional on a shared group effect,
`Y_i ~ Poisson(u_g*exp(X_i*beta))`, independently within group, with
`u_g ~ Gamma(shape=1/alpha,rate=1/alpha)`. E(u)=1 and Var(u)=alpha. This is
shared-group mixing, NOT independent-row NB2. For r=1/alpha, group count total
S and marginal total M=sum(exp(X beta)), the integrated contribution is:

```text
lgamma(r+S)-lgamma(r)+r*log(r)-(r+S)*log(r+M)
  +sum(y*X beta-lgamma(y+1))
```

All coefficients and ln(alpha) are estimated jointly. Covariance is inverse
**full joint observed information**, including coefficient/dispersion cross terms,
without a DF multiplier. Coefficients and ln(alpha) have normal95 inference;
alpha has delta SE and an exponentiated log-scale CI. The ln(alpha) z test has
null alpha=1, NOT alpha=0. Slope-only Wald uses the slope covariance block and
chi-square reference. The separate zero-variance LR compares the re-estimated
same-sample pooled Poisson with the conventional 50:50 chi-square(0,1) mixture.
These are model-based references, not robust/few-cluster or survey corrections.

Original ASCII: log likelihood−23419.13188988957, ln(alpha)−1.0398391652175825,
alpha0.3535115343374449, LNHHEXP−0.1013746329270513 and intercept−1.276973841736605.
An interior fit is not evidence of exogeneity, correct distribution or causality.

## Execute and independently reproduce

Build with `cargo build --locked --bin tabdat-explore-rs`. In an empty working
directory, with `repo` set to the repository path:

```sh
python3 "$repo/examples/mma24p2poiscluster/prepare.py" /path/to/vietnam_ex2.asc vietnam_individual.csv
"$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma24p2poiscluster/individual_gamma_re.td" > result.jsonl
Rscript "$repo/examples/mma24p2poiscluster/gamma_re_reference.R" /path/to/vietnam_ex2.asc reference
python3 "$repo/examples/mma24p2poiscluster/check_gamma_re.py" . reference /path/to/vietnam_ex2.asc
```

Run `--text` in a separate empty directory. Both interfaces export byte-identical
`individual_gamma_re_predictions.csv`: all27,765 complete source records, selected
commune totals, explicit all-true inclusion flags and marginal mean/index/residual.
The original input is unchanged. R/Python are optional preparation/validation
tools, not production dependencies.

The independent R4.6.2 reference uses base-R BFGS plus observed-information
refinement, direct special-function likelihood/score/information, central score
derivatives to check the full covariance and an independent NB-total/conditional-
multinomial likelihood decomposition. Rust instead uses bounded integer-gamma
recurrences, centered information and a guarded Newton search. The initial R
probe's unnormalized BFGS objective could overshoot toward unstable near-zero
dispersion; optimizing objective/score per group avoids that probe failure
without changing the optimum. No tolerance is widened to conceal disagreement.

The strict ASCII auditor checks **236 quantities**, every source record and every
marginal prediction record. Fixed tolerances: absolute1e-9, log likelihood1e-8,
nonzero-p relative1e-8 and exact zero underflow. Maximum quantity discrepancy
was below 1.051e-10. Native fitting used five information evaluations, with final
scaled score below 2.265e-13 and undamped step below 1.942e-15.

### Separate DTA and printed-author checks

DTA binary floats are more precise than the rounded publisher ASCII. In another
empty work directory, use the existing `dta_reference.R` converter for native
CSV import, but generate the reference **directly from DTA** with `foreign`:

```sh
Rscript "$repo/examples/mma24p2poiscluster/dta_reference.R" /path/to/vietnam_ex2.dta vietnam_individual.csv
"$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma24p2poiscluster/individual_gamma_re.td" > result.jsonl
Rscript "$repo/examples/mma24p2poiscluster/gamma_re_reference.R" /path/to/vietnam_ex2.dta dta-reference --dta
python3 "$repo/examples/mma24p2poiscluster/check_gamma_re.py" . dta-reference /path/to/vietnam_ex2.dta --dta
python3 "$repo/examples/mma24p2poiscluster/check_gamma_re_author.py" /path/to/vietnam_ex2.dta /path/to/mma24p2poiscluster.txt dta-reference result.jsonl
```

The DTA numerical audit checks the same236 quantities and every typed R-source
and native prediction record at the same tolerances; maximum quantity discrepancy
was below 2.247e-10. Both R and native match **29 printed author facts**: eleven
coefficient/SE pairs, ln(alpha)/SE, alpha/SE, log likelihood, slope Wald and boundary
LR. Printed comparisons use half the last printed digit, not the strict numerical
tolerance. Other printed fields, historical iterations and whole-table/book
parity are not claimed. Native DTA import support is not implied by optional
external conversion.

## State, predictions, failure and resource boundaries

`predict ..., n` is the marginal population mean exp(intercept+X beta); xb
includes the intercept; residuals are current y minus marginal mean. No current
ID is needed. These are NOT posterior group effects or conditional-on-estimated-
effect forecasts. Missing/nonfinite required current inputs give NULL; residuals
require finite current y. Unrepresentable finite-input requested predictions
fail atomically; xb does not compute an unnecessarily overflowing mean. Genuine
floating-point exponential underflow can yield zero. Predictions never refit.

A failed new fit clears stale models without changing data. Parser failures
preserve old state. Reporting and successful/failed export retain the owned fit.
Export refuses to overwrite an owner file. Scripts are intentionally
nontransactional: a failed export does not undo earlier table/model effects.
Use a fresh directory or another output path rather than deleting others' files.

Generic native fits use joint-finite model cases and structural raw-key auditing,
even on omitted model rows. The publisher script additionally requires positive
unique BIGINT source-row keys, positive BIGINT commune IDs and complete finite
model records except wholly missing rows; partial records fail before fitting.

Bounds: candidate rows<=40000, slopes1..32, rows*(slopes+2)²<=20million,
raw groups<=10000, selected groups>slopes+2, exact integer individual counts and
selected group totals<=100000. The group-total cap bounds recurrence work/cache;
this original sample's maximum is310. ln(alpha) must be strictly inside(-12,12).
RMS-rescaled design and per-group score/information require both max-absolute
score and full undamped-step tolerances1e-10. Intermediate damping does not change
final undamped observed covariance. Rank, curvature, boundary and convergence
failures are explicit: no silently clipped alpha, dropped groups or pooled model
mislabeled as RE. Automatic zero-variance fallback is unsupported.

The checked-in synthetic reference has279 source records/278 cases/48 groups,
four zero-total groups and two singletons. Tests check all236 quantities/every
prediction, scaling/order invariance, shared-vs-row mixing and the all-singleton
NB2 identity including full covariance; invalid keys/counts/sample/rank/weights/
bounds and genuine dispersion-boundary failures; CLI text/JSON, current-data
schema/missingness, atomic prediction failures, export-state and source/owner
preservation. Verification commands/results are recorded in the PR; hosted CI
and independent code review are not claimed.

The separate [shared-gamma RE whole-commune bootstrap](individual_gamma_re_bootstrap.md)
refits all joint parameters for each distinct group copy without replacing this
retained fit. Outside this estimator: normal RE, robust/cluster/survey covariance,
weights/offsets, posterior effects/uncertainty and generic margins/post-estimation.
Other GEE workflows, genuine survey design, remaining household outputs and
textbook illustrations remain required; this bounded implementation does not waive them. Eighteen ASCII audit forgeries and four
native-author forgeries were rejected during auditor validation.
