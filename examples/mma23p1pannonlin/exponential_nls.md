# Chapter 23 patents: exponential-mean nonlinear least squares

This bounded workflow reproduces the NLS / independent Gaussian GEE mean
illustration in §23.3/Table 23.1, using native `nl` for the fit and predictions
and a separate SQL composition for the two author covariance conventions. It is
**not** a general GEE implementation or a native clustered-NLS option. The
[author program](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.do)
and [log](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.txt)
provide the comparison. Text and CLI-v1 JSON report the fit and auxiliary table;
the input-order CSV retains original columns, conditional mean and residual.
The native NLS fit remains the model state; the SQL covariance is an auxiliary
report, not restorable fit state.

## Sample and model

The pinned `patr7079.asc` SHA-256 is
`87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8`.
Firm ID is the original source-record position, not CUSIP; missing ARDSSIC
metadata do not remove observations. Use **all 346 firms/1,730 firm-years** in
1975–79, including the 22 all-zero patent firms. Fit
`mu_it=exp(a+b*LOGR_it)` by unweighted NLS on PAT, with starts `(2.5,0.5)`;
no additional covariates, time effects, offsets or weights are used. The native
solver establishes a regular **local** minimum, not global uniqueness.
Predictions are fitted conditional means, not discrete-count probabilities or
out-of-sample uncertainty.

Let `j_it=(mu_it, mu_it*LOGR_it)` and `s_i=sum_t j_it*(PAT_it−mu_it)`.
With `B=(sum_it j_it j_it')^-1`, the auxiliary table reports full 2×2
covariances, intercept/slope SEs, z statistics and normal 95% intervals:

- **Working:** `(RSS/N)*B`, the author's Gaussian-GEE independent-working
  covariance. This is not the native `nl` report's `(RSS/(N−2))*B` with
  approximate Student-t(N−2) inference; both are printed and labeled separately.
- **Firm-cluster:** `G/(G−1) * B*(sum_i s_i s_i')*B`, an independent-firm
  Gauss–Newton sandwich. There is no additional `(N−1)/(N−2)` factor and no
  few-cluster guarantee. The auxiliary z/interval reference is asymptotic normal;
  p-values are not computed. It is not the native NLS independent-row HC1 option.

Both bread conventions require a correctly specified conditional mean for their
usual interpretation; working covariance additionally assumes homoskedastic,
independent rows. The firm sandwich permits within-firm score dependence but
assumes independent firms. None establishes a causal R&D effect, mean adequacy,
a valid count distribution, a Gaussian likelihood or a robust bootstrap.

| Original source, float64 | Intercept | LOGR slope |
| --- | ---: | ---: |
| NLS coefficient | 2.5287289311 | 0.5084672579 |
| GEE working SE, RSS/N | 0.0544557826 | 0.0105636120 |
| Firm-cluster SE | 0.2176674021 | 0.0550459934 |

RSS is 3,565,052.7652817; the author log prints `3565052.8`, coefficients
`2.528729`/`.5084673`, working SEs `.0544558`/`.0105636`, and firm SEs
`.2176674`/`.0550460`. Comparisons use printed precision rather than claiming
identical historical float storage or optimizer internals.

## Numerical and failure boundaries

The script requires unique integer `id,year` keys, IDs 1–346, one row for each
of 1975–79, at least two firms, counts 0–1,000, finite `|logr|≤20`, and at most
10,000 candidate rows. Native `nl` applies its own formula, rank, work, positive
residual-variance and strict convergence guards; it never silently drops a bad
complete row or substitutes a pseudoinverse. Review its [help](../../crates/tabdat-reporting/src/help/topics/nl.md)
for bounds and retained-state behavior.

The SQL recovers `(a,b)` from the fitted `ln(mu)` relation solely to label the
auxiliary report, and checks reconstruction within `1e-12*(1+max|ln(mu)|)`;
it is not a replacement fit. Weighted-centered Jacobian cross-products avoid
subtracting near-equal uncentered determinants. Firm influences are accumulated
from full within-firm scores. SQL checks finite positive working variances and
rejects a zero or cancellation-dominated cluster diagonal when it is no larger
than `(128*machine_epsilon)^2` times its absolute-score propagation scale.
Rank/fit, arithmetic, invalid keys, too few firms or indeterminate covariance
abort before export. Some SQL guards currently report only `sql failed` with
script location. Input and pre-existing output files stay unchanged. Export
refuses to overwrite; use a fresh directory and check process exit status,
since a report can precede an export failure.

## Reproduce and independent evidence

```sh
repo=/absolute/path/to/tabdat-explore-rs
work=/absolute/path/to/empty/work-directory
curl -fsSL https://cameron.econ.ucdavis.edu/mmabook/patr7079.asc -o "$work/patr7079.asc"
shasum -a 256 "$work/patr7079.asc"  # compare with pinned hash above
(cd "$repo" && cargo build --locked --bin tabdat-explore-rs)
sh "$repo/examples/mma23p1pannonlin/prepare.sh" "$work/patr7079.asc" "$work/patents.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma23p1pannonlin/exponential_nls.td" > result.jsonl)
Rscript "$repo/examples/mma23p1pannonlin/exponential_nls_reference.R" "$work/patr7079.asc" "$work/r-reference"
python3 "$repo/examples/mma23p1pannonlin/check_exponential_nls.py" "$work" "$work/r-reference" "$work/patr7079.asc"
```

The development-only R 4.6.1 reference independently reshapes raw records,
profiles the exponential amplitude and solves the scalar score with `uniroot`;
it independently evaluates both full covariance matrices and all 346 firm
scores. A separate R `nls` fit at tolerance `1e-7` is an **approximate**
cross-check, not the precision oracle: stricter R `nls` iterations stalled on
both tested datasets and are not reported as converged. The checker pins the
source hash, compares 27 summary values and native NLS coefficients/SEs, all
1,730 source/design/prediction rows, 346 scores and seven selected author facts.
Numerical comparisons use `1e-9 + 1e-10*abs(reference)`; firm-score comparisons
use a bound propagated from the individual mean/residual tolerances, not an
unscaled near-zero absolute tolerance. Source identities and model inputs are
exact. A 12-firm synthetic R fixture checks both covariance matrices, all
predictions and scores, including two zero-total firms; tests cover reversed
input order, text/JSON, no-clobber, missing/invalid/rank-deficient data,
too few firms, and exact-zero/cancellation-dominated cluster scores. R/Python
and author data are development-only.

The [NLS whole-firm bootstrap](exponential_nls_bootstrap.md) and bounded
[exchangeable Poisson GEE](exchangeable_gee.md) are separate validated
compositions. The [GEE firm bootstrap](exchangeable_gee_bootstrap.md) is also separate. Random-effects Poisson, general GEE, RE firm bootstrap,
Figure 23.1 and remaining Table 23.1 outputs remain required. This workflow
does not complete the chapter, graduate-method core or full book inventory.
