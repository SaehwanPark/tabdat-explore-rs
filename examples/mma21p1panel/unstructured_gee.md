# MMA §21.5.5 period-standardized unstructured Gaussian GEE

## Target and chosen method

The original March 2005 [author program](https://cameron.econ.ucdavis.edu/mmabook/mma21p4pangls.do)
fits `lnhr` on an intercept and `lnwg` over the 532-person, ten-year MOM panel
with Gaussian identity link and unstructured working correlation. The native
`xtgee` workflow uses all 5,320 complete person-years, `id(id)`, and
`time(year)`. It is a population-averaged association, not a fixed-effects or
causal estimate.

This product slice uses a period-standardized residual moment. For the final
residuals `e[g,t]`, first form

`M[s,t] = sum_g e[g,s] * e[g,t] / G`,

then set `R = D^(-1/2) M D^(-1/2)`, where `D` contains the diagonal of `M`.
The diagonal of `R` is one. The algorithm alternates Gaussian GLS coefficients
with this correlation estimate until the coefficient-step tolerance passes.
It then uses scale `phi = RSS/(N-K)`, working covariance
`phi * (sum_g X_g' R^(-1) X_g)^(-1)` and an uncorrected person-cluster sandwich
with multiplier1.0. Coefficient inference is normal95 with a slope Wald
chi-square test. No finite-sample correction is applied.

The matrix must be positive definite. Incomplete panels, duplicate entity/time
keys, missing/nonfinite model or time fields, rank failures, nonconvergence and
non-positive-definite matrices fail without changing the previous fit or active
data. The bounded implementation accepts balanced panels with2–10 shared time
values and keeps current-row marginal `xb` and residual predictions plus
explicit working/sandwich named reports.

This selected diagonal standardization is a distinct product definition. It
differs from the pooled residual-square denominator in the current official
[Stata `xtgee` manual](https://www.stata.com/manuals/xtxtgee.pdf), and its MOM
results do **not** reproduce the historical [author output](https://cameron.econ.ucdavis.edu/mmabook/mma21p4pangls.txt).
The command and machine-readable result identify the selected method. Do not
use it as evidence of Stata estimator parity.

## Original MOM result and source discrepancy

The downloaded `MOM.dat` source has SHA256
`2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77`. The
native result and published 2005 author-log values are:

| Quantity | Native period-standardized fit | Author Stata log | Native minus author |
| --- | ---: | ---: | ---: |
| Intercept | 7.4196367842 | 7.4262620 | −0.0066252158 |
| `lnwg` slope | 0.0910990841 | 0.0910023 | +0.0000967841 |
| Working SE, `lnwg` | 0.0134302714 | 0.0137712 | −0.0003409286 |
| Person-sandwich SE, `lnwg` | 0.0393844960 | 0.0501344 | −0.0107499040 |
| Gaussian scale | 0.08034835997 | 0.0803575 | −0.00000914003 |

The author program header labels the table 21.6 while its executable output
labels it 21.7. Its unstructured bootstrap command is commented out and the
last `matrix list unstrbootse` fails because the matrix was never defined.
Neither the table-number discrepancy nor bootstrap failure is silently
resolved here.

The current Stata manual defines its unstructured alpha using shared-panel
Pearson-residual cross-products divided by a pooled residual-square quantity.
On MOM, that pooled-denominator matrix is not positive definite at the initial
pooled-OLS coefficient; direct exploratory GLS updates cycle. CRAN `gee` 4.13-30
and `geepack` 1.3.13 under R4.6.2 did not give a converged fit for that
Stata-formula calculation on MOM. A separate synthetic G=60, T=4 `geepack`
fit converges, but uses the pooled-denominator estimator and is not a reference
for this selected product method. Exact Stata source parity and an independently
validated reference for the author output remain open.

## Reproduction and calculation cross-check

The dense [Base-R reference](unstructured_gee_reference.R) implements the
selected period-standardized calculation independently. The
[checker](check_unstructured_gee.py) compares the native JSON fit with that
reference, including100 correlation entries,8 working/sandwich covariance
entries,24 coefficient-inference quantities and31,920 ordered prediction
values. It verifies all5,320 rows and checks a raw-source MD5 recorded by R in
addition to the published SHA256 above. This is an implementation cross-check;
Rust/Base-R agreement does not establish agreement with Stata or a separate
statistical package.

From the checkout root, with Rust, R4.6.2 and Python3 available:

```sh
root="$PWD"
work=$(mktemp -d)
curl -fsSL https://cameron.econ.ucdavis.edu/mmabook/MOM.dat -o "$work/MOM.dat"
sh "$root/examples/mma21p1panel/prepare.sh" "$work/MOM.dat" "$work/MOM.csv"
cargo build --locked --bin tabdat-explore-rs
(cd "$work" && "$root/target/debug/tabdat-explore-rs" --json -f "$root/examples/mma21p1panel/unstructured_gee.td" > output.jsonl)
Rscript "$root/examples/mma21p1panel/unstructured_gee_reference.R" "$work/MOM.dat" "$work/reference"
python3 "$root/examples/mma21p1panel/check_unstructured_gee.py" \
  "$work/output.jsonl" "$work/unstructured_gee_predictions.csv" "$work/reference" "$work/MOM.dat"
```

The checker prints author-log quantities as diagnostics; it does not assert
source parity. Production Rust has no R/Python package dependency. Synthetic
stats/runtime tests cover balanced T=2 and T=10, physical row permutation,
correlation rank failure, incomplete/duplicate periods, retained fit state,
current-row prediction and named covariance reports.
