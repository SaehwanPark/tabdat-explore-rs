# Chapter21 exchangeable-working GEE companion report

## Target and conventions

P4-EXCH, [source inventory](gls_inventory.md), `mma21p4pangls.do` lines87–94:
Gaussian identity-link `lnhr ~ 1 + lnwg`, the published complete MOM panel,
5,320 person-years/532 people, 1979–1988. The author reports exchangeable
working and person-semi-robust covariance, normal z/95 and slope Wald chi-square.
This is not automatically the same estimator as RE-GLS.

The complete same-session workflow runs [gaussian_pa.td](gaussian_pa.td), then
[exchangeable_p4.td](exchangeable_p4.td). It reuses the independently validated
[legacy Pearson exchangeable GEE](gaussian_pa.md) fit, original four exports,
source-keyed predictions, score/convergence and retained pooled-OLS state. Its
parameter/scale convention is phi=RSS/N, exchangeable Pearson moment rho,
per-person working covariance `V_i=phi*((1-rho)I+rho*11')`, and bread
`H=sum_i X_i' V_i^-1 X_i`. The original raw person sandwich is
`H^-1 sum_i (X_i' V_i^-1 r_i)(X_i' V_i^-1 r_i)' H^-1`. The P4 continuation
multiplies this **full2x2** matrix by `G/(G-1)`; it does not add the OLS CR1
`(N-1)/(N-2)` factor. It preserves the original uncorrected output rather than
replacing or relabelling it.

The new fit/report exports both full working and adjusted population-parameter
covariances, their raw sandwich, multiplier, normal z/two-sided p/normal95 and
slope Wald chi-square(1). Only the slope receives a Wald statistic; no joint
covariance-rank test is claimed. On original MOM, wage slope0.11954740236 has
working SE0.01365069968, raw person-sandwich SE0.0514941 and adjusted person
SE0.05154259223 (p0.02037386851). Working inference assumes the specified second
moments; the sandwich assumes independent people and a correct marginal mean,
not independent person-years. These marginal associations are not causal,
formal correlation-specification tests, survey inference or few-cluster guarantees.

The predecessor accepts 3–532 complete ten-year people with exact finite unique
keys and bounded finite model fields, strictly identified within/between designs,
positive working eigenvariances and convergence/score certificates; its exact
bounds are in [gaussian_pa.md](gaussian_pa.md). The P4 continuation also rejects
unresolved marginal person-sandwich variances: each raw variance must exceed
128 machine epsilons times the corresponding working variance. This prevents
numerically unresolved near-zero unit scores from producing spurious significance;
it is a conservative workflow-specific admission rule. Guards can surface as
generic `sql failed`. The continuation requires the predecessor's named
`gaussian_pa_fit` in the **same session**; it is not a general saved-fit importer.
Neither script adds owned GEE state or a general GEE API; `predict` after the
workflow uses the retained pooled OLS, not exchangeable GEE.

## Reproduction and outputs

Prepare pinned data in a **fresh** work directory following
[shared preparation](README.md#reproduce). From the checkout root, after building
the binary:

```sh
root="$PWD"; work=/path/to/fresh/work
sh "$root/examples/mma21p1panel/prepare.sh" /path/to/MOM.dat "$work/MOM.csv"
printf 'run "%s/examples/mma21p1panel/gaussian_pa.td"\nrun "%s/examples/mma21p1panel/exchangeable_p4.td"\n' "$root" "$root" > "$work/p4_run.td"
(cd "$work" && "$root/target/debug/tabdat-explore-rs" --json -f p4_run.td > output.jsonl)
R_LIBS_USER=/path/to/R/library Rscript "$root/examples/mma21p1panel/gaussian_pa_reference.R" /path/to/MOM.dat "$work/gaussian_reference"
Rscript "$root/examples/mma21p1panel/exchangeable_p4_reference.R" "$work/gaussian_reference" "$work/p4_reference"
python3 "$root/examples/mma21p1panel/check_exchangeable_p4.py" "$work" "$work/gaussian_reference" "$work/p4_reference" /path/to/MOM.dat
```

Use `--text` in a separate fresh directory for readable batch results. The first
script's four no-clobber exports remain byte-identical to its standalone run;
they include its fit, original coefficient report, predictions and unit scores.
The continuation adds, in order, `panel_exchangeable_p4_fit.csv` (full matrices,
multiplier, sample/correlation and certificates) and
`panel_exchangeable_p4_coefficients.csv` (four inference rows, also a readable
and typed CLI-v1 table). Final active table is `gaussian_pa_predictions` and the
retained model is pooled OLS. This is **not** a whole-script transaction: a later
collision preserves its existing file but prior exports and state can remain.
Do not run the continuation in a different session or from a stale named fit.

## Independent evidence and limits

R4.6.2/gee4.13.30 with legacy `v4.4compat=TRUE`, independent dense per-person
matrix inversion, scores and base-R normal tails independently check18 original
baseline fit fields, all5,320x13 source/prediction cells and532x6 score/influence
cells; the P4 conversion adds19 fit and26 inference quantities. The checker binds
all42,560 raw fields to the SHA256-pinned `MOM.dat`. Absolute1e-9 and nonzero
p-value relative1e-8 apply, with exact zero for underflow; original maximum P4
fit/inference discrepancies are1.02e-12/4.79e-10. Both JSON/text runs emit six
byte-identical exports, and the original four match an independent standalone
predecessor run byte for byte. Twenty-five original author-log facts match their
printed precision without float32 adjustment or widened tolerances.

Two32-person synthetic controls, with rho0.56675 and rho−0.04574 and interleaved
people/reversed time, independently check both full covariance modes, score and
prediction rows, tails and G/(G-1) scaling. Tests also cover missing prerequisite,
duplicate/incomplete keys, rank/singular correlation, injected exhausted
iteration, **injected** unresolved unit variance, all six export collisions,
retained OLS/JSON/text behavior, six numeric forgeries and a shared-wrong source
forgery. The injected guards are failure-path tests, not evidence that original
MOM fails either condition. Neither R nor Python is a production dependency.

The source's exchangeable B500 bootstrap is a separate target: the log requests
500 but displays72 coefficient replications, and its historical RNG/duplicate
ID semantics remain unresolved. AR1/unstructured working models, other P4
bootstraps and the full mixed table remain required. No likelihood, prediction
uncertainty, weighted/unbalanced panels or literal historical iteration replay
is claimed; source header Table21.6 versus body Table21.7 conflict is recorded
in the [inventory](gls_inventory.md).
