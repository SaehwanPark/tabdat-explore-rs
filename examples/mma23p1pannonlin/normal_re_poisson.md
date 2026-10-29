# Chapter 23 patents: normal random-intercept Poisson

This bounded SQL composition fits the normal random-intercept Poisson
illustration in Cameron and Trivedi (2005), §23.3/Table 23.1, using the March
2005 [author program](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.do)
and [log](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.txt). It
is **not** a general native `xtpoisson` command. Text and JSON report the same
inference; `normal_re_poisson_predictions.csv` contains source fields and
marginal means/residuals in `(id,year)` order. The historical normal-RE output
is materially different and has missing standard errors; it is not silently
counted as reproduced (see below).

## Sample, likelihood and inference

Pinned `patr7079.asc` SHA-256:
`87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8`.
Firm `id` is source-record position, not CUSIP. All **346 firms and 1,730
observations** in 1975–79 are retained, including 22 zero-total firms and
missing nonmodel ARDSSIC metadata. The only regressor is LOGR plus an
intercept; there are no year effects, weights or offsets.

Firms are independent, and conditional on a shared random intercept,
`Y_it ~ Poisson(exp(a+b*x_it+u_i))` independently over time, with
`u_i ~ N(0,sigma²)` independently of the covariates. The same `u_i` affects all
five outcomes for firm `i`. The firm likelihood integrates the product of
those five Poisson probabilities against the normal density. This parameterizes
a *mean-zero log effect*; unlike [gamma RE](gamma_re_poisson.md), its
multiplicative effect has mean `exp(sigma²/2)`, not one. Consequently the
population marginal prediction is `exp(a+b*LOGR+sigma²/2)`, and the residual is
`PAT−mu`. These predictions integrate over the random-effect distribution;
they are not firm-specific posterior predictions. The [current Stata manual](https://www.stata.com/manuals/xtxtpoisson.pdf)
describes the same normal-intercept model and log-variance parameterization;
this is not evidence about the historical Stata version's computations.

The script estimates `(a,b,ln(sigma²))` jointly and reports the inverse **full
joint observed information**, including coefficient/variance cross terms,
normal z tests and two-sided p-values, and normal 95% intervals. Sigma's SE is
`(sigma/2)*SE(ln(sigma²))`; its interval transforms the log-variance interval.
The reported z test for `ln(sigma²)=0` tests `sigma²=1`, **not** zero variance;
the boundary LR test below addresses zero variance. The likelihood comparison
with nested pooled Poisson uses the conventional
50:50 chi-square(0)/chi-square(1) boundary reference for `sigma²=0`, relying on
the stated model and boundary regularity. These are model-based, not robust or
few-firm corrections. Neither inference nor prediction establishes the
conditional distribution, exogeneity, causality or out-of-sample accuracy.

| Original source | Estimate | Model-based SE |
| --- | ---: | ---: |
| Intercept | 1.4142531252 | 0.0991669221 |
| LOGR slope | 0.2929878941 | 0.0350192988 |
| ln(sigma²) | 0.9897262362 | 0.1151192856 |
| sigma | 1.6402736999 | 0.0944135683 |

The log likelihood is `−5554.0170602134`, the pooled Poisson log likelihood is
`−21030.5825204597`, and the boundary LR statistic is `30953.13092049`. Its
p-value underflows to zero in float64; zero indicates numerical underflow, not
a mathematically zero probability. The joint covariance of `(a,b,ln(sigma²))`
is `[[0.00983407843, −0.00137800500, 0.00208196250],
[−0.00137800500, 0.00122635129, −0.00263060314],
[0.00208196250, −0.00263060314, 0.01325244992]]`.

## Historical output discrepancy

The historical log prints slope `.815977`, intercept `1.156293`,
`ln(sigma²)=-1.310299`, log likelihood `−6261.9825`, and **missing SEs**. It
shows only two full-model iterations. Independently evaluating the declared
integrated normal-intercept likelihood at those *rounded printed parameters*
gives log likelihood `−5871.3505055` and score approximately
`(−273.5971, −981.7950, 246.0723)`. These values are not a stationary point of
the declared likelihood; the discrepancy is far larger than displayed rounding.
The old program's quadrature, convergence and other implementation details
are not established, so this does not prove exactly why its values differ or
reconstruct its internals. The historical coefficient/SE output remains
unresolved and is **not** counted as exact table parity.

## Numerical, state and file boundaries

The input must have unique integer `(id,year)`, IDs 1–346, exactly one row in
each year 1975–79, at least two firms, integer PAT counts 0–1,000 and finite
`|LOGR|≤20`. Every firm is retained or the workflow fails. Native pooled
Poisson supplies the start, and recovered coefficients must reconstruct each
native linear prediction within `1e-12*(1+max|pooled_xb|)`; `ln(sigma²)` starts
at zero. The first joint Newton step is halved because the original sample's
full first step decreases the likelihood; subsequent steps are full. This is a
bounded local procedure, not a global optimizer or an automatic line search.
It permits at most 100 steps; bounds `|a|,|b|≤20`, `|ln(sigma²)|<8`, conditional
linear predictors to `[−40,40]`, marginal linear predictors to `[−700,700]`,
and quadrature nodes in random-effect space to `[−100,100]`. Nonascending,
boundary, singular, cancellation-dominated, out-of-range or nonconvergent
fits fail rather than being clipped or regularized.

At each step, every firm and row is checked against the original five-year
sample. The strictly concave posterior mode is bracketed in `[−60,60]` and
bisected 55 times, with a scale-aware root check. Adaptive Gauss–Hermite
quadrature uses **96 physicists' nodes** around that mode and curvature; a
separate 80-node evaluation checks each firm's log integral, first moments and
centered posterior covariance moments. The maximum absolute difference divided
by `1+abs(96-node value)` must not exceed `1e-9`. This is a convergence check,
not a universal quadrature error bound. Posterior covariances are computed
from centered products, not cancellation-prone raw second-moment
subtraction. The script rejects nonfinite or nonpositive scale-aware
information pivots and propagation-sensitive coefficients. The final scaled
step is at most `1e-12` and all scores obey the `1e-10` absolute-term-scale
criterion. An isolated high-variance starting-point probe produces an actual
80/96-node discrepancy of `5.63e-9` and is rejected. Some SQL guards surface
only `sql failed` and a script location.

The native pooled Poisson fit and `pooled_xb` prediction occur before the SQL
fit. If later SQL fails, those earlier session changes remain; the failed SQL
statement does not replace active data or the prior fit. After success the
active data are the exported prediction relation, but the retained native model
is still pooled Poisson, **not** normal RE; later native `predict` is therefore
not an RE prediction. There is no restorable normal-RE model. The input file is
unchanged and export refuses to overwrite. Use a fresh output directory and
check exit status: a report can precede an export failure.

## Reproduce and independent evidence

```sh
repo=/absolute/path/to/tabdat-explore-rs
work=/absolute/path/to/empty/work-directory
curl -fsSL https://cameron.econ.ucdavis.edu/mmabook/patr7079.asc -o "$work/patr7079.asc"
shasum -a 256 "$work/patr7079.asc"  # compare with the pinned hash above
(cd "$repo" && cargo build --locked --bin tabdat-explore-rs)
sh "$repo/examples/mma23p1pannonlin/prepare.sh" "$work/patr7079.asc" "$work/patents.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma23p1pannonlin/normal_re_poisson.td" > result.jsonl)
Rscript "$repo/examples/mma23p1pannonlin/normal_re_poisson_reference.R" "$work/patr7079.asc" "$work/r-reference"
python3 "$repo/examples/mma23p1pannonlin/check_normal_re_poisson.py" "$work" "$work/r-reference" "$work/result.jsonl" "$work/patr7079.asc"
Rscript "$repo/examples/mma23p1pannonlin/normal_re_poisson_reference.R" "$work/patr7079.asc" "$work/historical.csv" historical
Rscript "$repo/examples/mma23p1pannonlin/normal_re_quadrature_nodes.R" "$work/nodes.sql"
# Optional 96/128-node refinement in a separate no-clobber directory.
Rscript "$repo/examples/mma23p1pannonlin/normal_re_quadrature_nodes.R" "$work/nodes96-128.sql" 96 128
python3 "$repo/examples/mma23p1pannonlin/normal_re_refinement.py" "$repo/examples/mma23p1pannonlin/normal_re_poisson.td" "$work/nodes96-128.sql" "$work/refined.td"
mkdir "$work/refine"
cp "$work/patents.csv" "$work/refine/patents.csv"
(cd "$work/refine" && "$repo/target/debug/tabdat-explore-rs" --json -f "$work/refined.td" > result.jsonl)
python3 "$repo/examples/mma23p1pannonlin/check_normal_re_poisson.py" "$work/refine" "$work/r-reference" "$work/refine/result.jsonl" "$work/patr7079.asc"
```

Development-only R 4.6.1 independently performs direct adaptive improper
integration for each firm's likelihood and posterior moments, maximizes the
integrated likelihood with BFGS, and refines the joint score and observed
information. Richardson-refined numerical differentiation of the independent
score checks the full covariance; on the original sample the maximum covariance
difference is `5.09e-12`. The audit checks the pinned hash, all 1,730 exact
source/design and marginal-prediction rows, and **42** independent report
quantities, including every joint-covariance element. Comparisons use
`1e-9+1e-10*abs(reference)`; nonzero p-values use relative tolerance `1e-9`.
Original maximum mean/residual differences are about `1.2e-11`. An additional
96-versus-128-node fit comparison gives maximum parameter difference
`2.03e-13` and maximum covariance difference `5.86e-14`; the independent
R-integral audit also passes for the 128-node fit. The R node generator uses
orthogonal-polynomial recurrence for tiny tail weights rather than unreliable
tiny eigenvector components.

The 40-firm synthetic fixture retains eight zero-total firms and checks all 200
source/prediction rows and 42 report quantities. Integration tests cover
text/JSON, reversed input, no-clobber, missing/duplicate/nonfinite or out-of-
range data, a real identical-panel zero-variance boundary, singular design,
start reconstruction and retained pooled-fit/data/file invariants. They exercise
the quadrature nodes' normal moments, actual high-variance quadrature
disagreement, forced mode/predictor/iteration/information/sensitivity bounds
and incomplete iteration sample. Independent R and Python are development-only.

Normal-RE bootstrap, general nonlinear panels, Figure 23.1 and the rest of the
book inventory remain separate required work. This slice does not complete
Chapter 23, the graduate methods core or the full textbook target.
