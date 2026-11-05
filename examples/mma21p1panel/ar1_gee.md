# Chapter21 AR(1)-working Gaussian GEE

## Target and interpretation

P4-AR1: [source inventory](gls_inventory.md), `mma21p4pangls.do` lines100–107.
The original MOM sample contains 532 people, all ten years1979–1988 (5,320
person-years), `lnhr ~ 1 + lnwg`, Gaussian identity link, no weights or case
deletion. `ar1_gee.td` explicitly validates unique id/year/source_row keys and
uses **keyed calendar order**, not physical adjacency, for all 4,788 annual pairs.
It fits a marginal association, not fixed effects or a causal effect. No formal
serial-correlation test or correct-mean/few-cluster guarantee is implied.

The script composes native SQL, pooled OLS, predictions, named tables and exports;
there is no general native `xtgee` or new estimator state. The retained model is
**pooled OLS**, so later generic `predict` is pooled OLS, not AR1 GEE. GEE means,
residuals and scores are explicit in-sample columns/exports. The printed OLS t/F
inference differs from the GEE report; a preceding scope note distinguishes them.

## Exact estimation and inference

For G people, ten years each, N=10G and residual r_it, estimate
`phi=sum(r_it^2)/N` and `rho=sum_i,t=2..10(r_it*r_i,t-1)/(9G*phi)`.
Set `V_i=phi*R`, `R_ts=rho^|t-s|`; its inverse is tridiagonal with endpoint
diagonal1, interior diagonal `1+rho^2`, off-diagonal `-rho`, all divided by
`phi*(1-rho^2)`. Start at native pooled OLS; alternate Pearson rho and both
population GLS coefficients. Update through residual scores, avoiding catastrophic
cancellation in the uncentered normal-equation right-hand side. Convergence
requires both relative coefficient steps `abs(delta)/(1+abs(coefficient))<=1e-12`
before the100-update boundary; recompute all final moments and require both
inverse-bread-scaled score steps `<=1e-10*(1+abs(coefficient))`.

With `H=sum_i X_i' V_i^-1 X_i` and `U_i=X_i' V_i^-1 r_i`, the full working
covariance is `H^-1`; the full unit sandwich is
`G/(G-1)*H^-1 sum_i U_i U_i' H^-1`. No extra OLS CR1 `(N-1)/(N-2)` factor
is used. Independent people and a correctly specified marginal mean are needed
for sandwich inference, which permits misspecified within-person covariance.
Both parameter rows report normal z, two-sided p and normal95; the slope reports
Wald chi-square(1), not OLS t/F. No joint covariance-rank test or prediction
uncertainty is claimed. Original rho0.47959006751, phi0.08031294333 and slope
0.08437767906 have working SE0.01234280623 and unit-sandwich SE0.03717640044
(p0.02322839972). These are not tests of whether AR1 itself is correct.

Admission is deliberately bounded: 3–532 complete people with unique finite
integer id1..532/year1979..1988/source_row1..5320, finite lnhr/lnwg of magnitude
at most20, pooled centered wage sum of squares `>1e-12`, residual RSS `>1e-12`,
`1-rho^2>1e-10`, identified positive weighted design and determinant `>1e-12`,
finite coefficient updates of magnitude at most100, convergence/score bounds and
both unit-sandwich marginal variances greater than 128 machine epsilons times
their working variances. Reject rather than clip rho or silently change the
sample. These are conservative workflow guards, not universal GEE admission
rules. Ancillary fields are preserved; the original/control audits additionally
check all raw fields. Guard failures can surface as generic `sql failed`; inspect
the sample and guards above.

## Reproduction and outputs

Prepare the pinned publisher data as in [shared preparation](README.md#reproduce)
in fresh separate JSON/text directories. From the checkout root, after building
the binary:

```sh
root="$PWD"; work=/path/to/fresh/work
sh "$root/examples/mma21p1panel/prepare.sh" /path/to/MOM.dat "$work/MOM.csv"
(cd "$work" && "$root/target/debug/tabdat-explore-rs" --json -f "$root/examples/mma21p1panel/ar1_gee.td" > output.jsonl)
R_LIBS_USER=/path/to/R/library Rscript "$root/examples/mma21p1panel/ar1_gee_reference.R" /path/to/MOM.dat "$work/reference"
python3 "$root/examples/mma21p1panel/check_ar1_gee.py" "$work" "$work/reference" /path/to/MOM.dat
```

For synthetic positive/negative controls, run `gaussian_pa_synthetic.py positive`
(or `negative`) with a fresh directory; it writes both `MOM.csv` and `MOM.dat`.
The zero-lag input is the inspectable
[`zero_inputs.csv`](../../tests/fixtures/mma21_ar1_gee/zero_inputs.csv); copy it
to a fresh `MOM.csv` and convert the eight data columns to a reference file using
`awk -F, 'NR>1 {for(i=2;i<=NF;i++) printf "%s%s",$i,(i==NF?"\n":" ")}' MOM.csv > MOM.dat`.
Run the same native and R commands on each, then add `--synthetic` to the checker.

Use `--text` in a separate fresh directory for readable batch results. Four
ordered no-clobber exports are `panel_ar1_gee_fit.csv` (sample, rho/phi, full
covariances and score certificate), `panel_ar1_gee_coefficients.csv` (four
normal-inference rows and a readable/typed CLI-v1 table),
`panel_ar1_gee_predictions.csv` (all source and pooled/GEE predictions) and
`panel_ar1_gee_unit_scores.csv` (ordered unit scores and influences). The final
active table is `ar1_gee_predictions` and retained state is pooled OLS. Exports
are **not** a whole-script transaction: a later collision preserves its existing
file but prior files/session effects can remain.

## Independent evidence and limits

R4.6.2/gee4.13.30 `corstr='AR-M', Mv=1, v4.4compat=TRUE`, plus a separate dense
10x10 working-covariance/score iteration, independently check the original
sample:17 fit quantities,26 inference quantities, all5,320x13 source/prediction
cells and532x6 score/influence cells. The checker also hashes MOM.dat, binds all
42,560 raw source fields, reconstructs all4,788 keyed lag pairs and independently
checks rho, score/influence and sandwich identities. Fixed absolute1e-9,
nonzero p-value relative1e-8 and exact underflow zero apply. Original maximum
native/R differences are9.09e-13 fit,8.67e-12 inference,2.88e-13 predictions
and7.82e-13 scores. Twenty-five author facts match printed precision without
float32 adjustment or tolerance widening. JSON/text CSV outputs are byte-identical.

The independent dense solution and native residual-score algorithm each took5
updates; R gee at `tol=1e-12` converged in8. Iteration counts are algorithm
metadata, not historical Stata parity. A preliminary R gee `tol=1e-13` run exhausted
100 iterations; its nonconverged iterate is not a reference. Three independent
32-person controls give rho0.52742/−0.13977/about0, reverse/interleave physical
input order and validate full covariances, predictions and unit scores. The
mathematically zero-lag construction reduces the coefficients to pooled OLS
within floating-point resolution; it does not imply all covariances are equal.
Actual-script tests cover both singular rho=±1 cases, missing/duplicate/incomplete keys,
nonfinite/oversized fields, rank/perfect-fit failures, injected convergence and
unit-variance guard failures, all four export collisions, retained OLS, text/JSON,
six numeric forgeries and a shared-wrong source forgery. Injected failures are
not claims of natural nonconvergence or unresolved variance in original MOM.
Neither R nor Python is a production dependency.

The separate source AR1 B500 bootstrap omits `t(year)` in its bootstrap command;
resampling/refitting, duplicate-ID, RNG and historical exact behavior remain
open. Unstructured correlation, full mixed reporting, exact historical iteration
and remaining chapter/book illustrations also remain required.
