# Chapter 23 patents: gamma random-effects Poisson

This bounded SQL composition fits the gamma random-effects Poisson column of
Cameron and Trivedi (2005), §23.3/Table 23.1, using the March 2005
[author program](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.do)
and [log](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.txt).
It is **not** a general native `xtpoisson` command. Text and JSON report the
same inference; `gamma_re_poisson_predictions.csv` contains source fields and
marginal means/residuals in `(id,year)` order.

## Sample, likelihood and inference

The pinned `patr7079.asc` SHA-256 is
`87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8`.
Firm `id` is original source-record position, not CUSIP. All **346 firms and
1,730 observations** in 1975–79 are retained, including 22 zero-total firms
and missing nonmodel ARDSSIC metadata. The only regressor is LOGR plus an
intercept; there are no year effects, weights or offsets.

Firms are independent. Conditional on a shared multiplicative firm effect,
`Y_it ~ Poisson(u_i*exp(a+b*x_it))` independently over time, with
`u_i ~ Gamma(shape=1/alpha, rate=1/alpha)` independently of the covariates.
Thus `E(u_i)=1`, `Var(u_i)=alpha`, and the **same** `u_i` induces within-firm
dependence. This is not independent row-level negative-binomial mixing. With
`r=1/alpha`, `Y_i=sum_t y_it` and `M_i=sum_t exp(a+b*x_it)`, each joint log
likelihood contribution, including factorial constants, is

```
lgamma(r+Y_i)-lgamma(r)+r*log(r)-(r+Y_i)*log(r+M_i)
  + sum_t[y_it*(a+b*x_it)-lgamma(y_it+1)].
```

The script estimates `(a,b,ln(alpha))` jointly. It reports the inverse **full
joint observed information**, including coefficient/dispersion cross terms,
normal z tests and two-sided p-values, and normal 95% intervals. Alpha's SE is
the delta-method `alpha*SE(ln(alpha))`; its interval exponentiates the log-scale
interval. It also compares the likelihood with the nested pooled Poisson fit:
`2*(LL_gamma−LL_Poisson)` uses the conventional equal mixture of chi-square(0)
and chi-square(1) under the boundary null `alpha=0`. The reference relies on
the model and usual boundary regularity, not a robust or few-firm correction.

Marginal predictions are `exp(a+b*LOGR)` and residuals are `PAT−mu`; they
integrate over the random-effect distribution rather than estimate firm-specific
posterior means. Neither these predictions nor model-based inference establish
a correct conditional distribution, exogeneity, causality or out-of-sample
accuracy. A separate [distinct-copy whole-firm bootstrap](gamma_re_bootstrap.md)
provides bootstrap summaries under explicitly declared ID and RNG conventions;
the normal-RE alternative remains required work.

| Original source, float64 | Estimate | Model-based SE |
| --- | ---: | ---: |
| Intercept | 2.3127049950 | 0.1247579916 |
| LOGR slope | 0.3487831736 | 0.0332254423 |
| ln(alpha) | 0.5454693087 | 0.0899144208 |
| alpha | 1.7254179461 | 0.1551399552 |

The log likelihood is `−5553.1787282240`; Poisson boundary log likelihood is
`−21030.5825204597`; LR statistic is `30954.80758447`. Its p-value underflows
to zero in float64; zero denotes numerical underflow, not a mathematically
zero probability. The full covariance of `(a,b,ln(alpha))` is
`[[0.01556455646, −0.00341333231, 0.00587625979],
[−0.00341333231, 0.00110393002, −0.00190731186],
[0.00587625979, −0.00190731186, 0.00808460306]]`.

## Historical precision distinction

The author log prints slope `.3487832`, intercept `2.312705`, ln(alpha)
`.5454692`, the three SEs `.0332254`, `.124758`, `.0899144`, and log likelihood
`−5553.1787`. These selected numbers agree at printed precision **except**
ln(alpha): converged float64 `.5454693087` differs by `1.087e-7` and rounds to
`.5454693`. The separately coded five-step audit gives `.5454692121` after
float32 conversion of LOGR, which rounds to the author value; float64 gives
`.5454693087`. The float32 result supports input precision as a contributor,
not identical Stata internals or an exact historical fit. The author reports
four full-model iterations; the independent full-Newton sequence is not
identical to all historical iteration values. No tolerance is widened to
claim seven-decimal ln(alpha) parity.

## Numerical, state and file boundaries

The script requires unique integer `(id,year)`, IDs 1–346, exactly one row in
each year 1975–79, at least two firms, nonnegative integer counts at most 1,000,
and finite `|LOGR|≤20`. Every firm is retained or the workflow fails. Native
pooled Poisson supplies the start; recovered start coefficients must reconstruct
all native linear predictions within `1e-12*(1+max|pooled_xb|)`. Full observed-
information Newton steps start at `ln(alpha)=0` and are bounded by 100 steps,
`|a|,|b|≤20`, `|ln(alpha)|<8` and linear predictors in `[−40,40]`. At each
step the full five-row/firm counts are checked against the original sample,
information must be positive definite by scale-aware Schur-complement guards,
and the likelihood must not materially decrease. Gamma-score and curvature
differences use bounded integer-count recurrences, not cancellation-prone
subtractions of special functions. The final scaled step is at most `1e-12`;
all final scores obey the `1e-10` absolute-term-scale criterion, the full
covariance is finite and positive on its diagonal, and propagated score
roundoff must be below `1e-9+1e-10*abs(coefficient)` componentwise. The LR
must be numerically distinguishable from zero. Full Newton steps are not a
global optimizer: nonascending, boundary, ill-conditioned or otherwise
nonconvergent cases fail rather than silently returning a local/boundary fit,
regularizing the information or dropping firms. These limits are specific to
this composition.

For normal tails, the implementation uses an 80-step alternating series for
`|z|≤3` and a 100-step Mills-ratio continued fraction for `|z|>3`; probabilities
can underflow to zero in float64. A 13-point independent R test covers the
transition and tails through `|z|=39`. The underlying SQL guards sometimes
surface only `sql failed` and a script location.

The native pooled Poisson fit and `pooled_xb` prediction are performed before
the SQL fit. If later SQL fails, those earlier changes remain; the failed SQL
statement does not replace the active data or previous fit. After success the
active data are the exported prediction relation, but the retained native fit
is still pooled Poisson, **not** gamma RE. In particular, later native
`predict` is not an RE prediction. There is no restorable gamma-RE model. The
input file is unchanged, and export refuses to overwrite. Use a fresh output
directory and check exit status: report output can precede an export failure.

## Reproduce and independent evidence

```sh
repo=/absolute/path/to/tabdat-explore-rs
work=/absolute/path/to/empty/work-directory
curl -fsSL https://cameron.econ.ucdavis.edu/mmabook/patr7079.asc -o "$work/patr7079.asc"
shasum -a 256 "$work/patr7079.asc"  # compare with the pinned hash above
(cd "$repo" && cargo build --locked --bin tabdat-explore-rs)
sh "$repo/examples/mma23p1pannonlin/prepare.sh" "$work/patr7079.asc" "$work/patents.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma23p1pannonlin/gamma_re_poisson.td" > result.jsonl)
Rscript "$repo/examples/mma23p1pannonlin/gamma_re_poisson_reference.R" "$work/patr7079.asc" "$work/r-reference"
python3 "$repo/examples/mma23p1pannonlin/check_gamma_re_poisson.py" "$work" "$work/r-reference" "$work/result.jsonl" "$work/patr7079.asc"
Rscript "$repo/examples/mma23p1pannonlin/gamma_re_poisson_historical_audit.R" "$work/patr7079.asc" "$work/precision-probe.csv"
```

Development-only R 4.6.1 independently maximizes the integrated likelihood
with BFGS and refines the score to below `1e-8` using full observed information.
A numerical derivative of the analytic score checks the full covariance, and
an independent negative-binomial total plus conditional-multinomial
factorization checks the likelihood (original difference `1.01e-11`). The
Python audit checks the pinned hash, exact source and model-design values, all
1,730 marginal predictions/residuals and 37 independent report values, including
all covariance terms. Floating-point comparisons use `1e-9+1e-10*abs(reference)`;
nonzero p-values use relative tolerance `1e-9`, including very small tails.
The original maximum observed marginal-mean difference is about `3.5e-12`.

The 12-firm synthetic fixture checks all 60 source/prediction rows, 37 report
values and full covariance, retaining two zero-total firms. Integration tests
cover text/JSON, reversed input, no-clobber, invalid/missing/nonfinite or
out-of-range values, duplicate keys and singular design, an actual dispersion
boundary, forced predictor/iteration/sensitivity limits and incomplete
iteration sample, and preserved prior fit/data/files. During development, a
multiple-reference recursive SQL plan could read a partial first-iteration
sample under materialization; the composition now aggregates each iteration
from one panel scan and explicitly checks firm/row counts on every step. R,
Python and author data are development-only.

This does not cover normal random effects, general GEE or Figure 23.1. The
separate [gamma-RE bootstrap](gamma_re_bootstrap.md) uses distinct firm copies;
historical bootstrap semantics and the rest of the book inventory remain
unresolved, so the chapter and roadmap are not complete.
