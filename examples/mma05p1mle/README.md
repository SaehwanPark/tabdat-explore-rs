# Table 5.7: OLS and exponential maximum likelihood

This bounded native composition reproduces the first two columns of Cameron and
Trivedi (2005), Table 5.7, p. 161: OLS and exponential rate maximum likelihood,
each with classical and independent-row robust covariance, using the 10,000
original generated observations. It includes row-level fitted values, residuals,
the exponential-density log likelihood and level-scale R-squared. The data are
a generated example, not an empirical cohort or the entire Table 5.7.

The author [program](https://cameron.econ.ucdavis.edu/mmabook/mma05p1mle.do)
and [output](https://cameron.econ.ucdavis.edu/mmabook/mma05p1mle.txt) have
SHA256 `02101c354f573c0cfdce37c2a706aadb19391a99bee0d6e80c986df8d274112a`
and `1080aa0d106d29303e5ee61e3a4d7b111b448ddf78e07b2cd2ec2d3bd5585650`.
The separately acquired original ASCII y,x data SHA256 is
`460f5d731f1848c8695d78a1fbf0db348d0daced4b8b5b0c2734bffa20b3e31c`;
[the preparation tool](../mma05p2nls/prepare.py) checks it and preserves source
ordinals and numeric tokens. No production R or Python dependency is added.

The design is an intercept and x, in that parameter order, with all rows used.
The executable preflight enforces unique dense ordinals, 3–10,000 finite rows,
positive y and nonconstant x and y. Rank, convergence, representability,
positive variance, and existing native estimator bounds apply. Four fits are
independent commands: OLS classical, OLS HC1, exponential model-based, and
exponential independent-row score sandwich. All predictions and summary sums
must be finite before the no-clobber export. Earlier script effects are not
rolled back on later failure.

OLS fits `y=a+b*x` as a linear projection; the source conditional mean is
nonlinear, so OLS is not an estimate of the exponential mean function. Its
classical covariance uses RSS/(N-2) times `(X'X)^-1` and assumes homoskedastic
errors. HC1 uses `N/(N-2)` times the residual-score sandwich, with Student-t
N-2 coefficient inference. The reported Gaussian log likelihood is a working
model evaluated at RSS/N, not the exponential-density likelihood.

For the exponential model `lambda=exp(a+b*x)`, `E[y|x]=1/lambda` and the
individual density log likelihood is `a+b*x-y*lambda`. Native Poisson on an
all-one response with offset `ln(y)` has the same score and observed
information; its count-pmf log likelihood adds `sum(ln(y))` to the density
log likelihood. This is an exact likelihood identity, not a count DGP. The
classical covariance is the inverse observed information; the robust score
sandwich uses `N/(N-1)` and asymptotic normal inference. The corresponding
likelihood-ratio reference is model-based only. Poisson Pearson/deviance and
other count diagnostics are not exponential-model adequacy tests.

The script reports the exponential-density log likelihood and level residual
`1-RSS/TSS` separately from the native Poisson result. Level R-squared is a
descriptive measure, not likelihood R-squared or a cross-family hypothesis
test. On the source ASCII data the R and native density log likelihood is
-208.713838979224, Poisson primitive log likelihood -15752.1897395779,
exponential level R-squared 0.3906230435, and OLS R-squared 0.2326195819.
Fitted intercept/slope are (-0.0092544603, 0.6198181572) for OLS and
(1.9829208731, -0.9896275707) for the exponential rate. These match the
selected four-decimal author coefficient/SE cells; the ASCII round trip is not
claimed to reproduce every original in-memory Stata float or formatting digit.

R 4.6.1 uses `lm.fit` and an independently profiled exponential likelihood with
Brent slope root, direct information and score outer products. It checks the
score, full covariance, coefficients, standard errors, statistics, p-values,
normal intervals for the exponential fit, both likelihood conventions and all
100,000 numerical projection fields. The checker verifies all prepared rows
against the pinned source and complete retained indices for Poisson; OLS reports
counts, and its joint-complete source sample is enforced before fitting. The
primary comparison tolerance is `2e-11*(1+abs(reference))`; nonzero p-values
below `1e-20` additionally require `1e-8` relative agreement. No tolerance was
widened to hide a discrepancy. JSON/text exports match byte for byte. A separate
six-row R control covers four fits and both covariance modes; invalid response,
duplicate identity, constant x/y and output collision are checked by CLI tests.

From a fresh output directory:

```sh
python3 /path/to/repo/examples/mma05p2nls/prepare.py /path/to/mma05data.asc nls.csv
/path/to/tabdat-explore-rs --json -f /path/to/repo/examples/mma05p1mle/comparison.td > result.jsonl
Rscript /path/to/repo/examples/mma05p1mle/reference.R /path/to/mma05data.asc reference
python3 /path/to/repo/examples/mma05p1mle/check.py /path/to/mma05data.asc result.jsonl comparison_predictions.csv reference
```

The result is a bounded workflow composition, not a general exponential
estimator, factor or weights API, clustered inference, fit restoration, or
whole-table/chapter completion. Other method columns and the historical
forward-difference discrepancy in the marginal-effects example remain separate.

## Named OLS/exponential comparison

[`named_reports.td`](named_reports.td) stores the OLS and offset-Poisson fits
and compares all four reporting snapshots without refitting them. Each Poisson
column explicitly names its fixed `log_exposure` offset and selected
covariance correction. Its likelihood and McFadden value remain those of the
Poisson count-pmf primitive, not the exponential density; use `comparison.td`
for the separately calculated density likelihood. OLS and Poisson outcomes
and likelihood conventions differ even though the source rows coincide.

```sh
/path/to/tabdat-explore-rs --text -f /path/to/repo/examples/mma05p1mle/named_reports.td
```

For source verification, generate the R reference described above in a fresh
`reference` directory, run the named example in JSON mode to `named_result.jsonl`,
and audit it with:

```sh
python3 /path/to/repo/examples/mma05p1mle/check_named_reports.py /path/to/mma05data.asc named_result.jsonl reference
```

The original 10,000-row replay checked every source identity, all four stored
fits against their live coefficient/covariance results and the independent R
reference, including the fixed offset and Poisson likelihood convention. The
reports preserve fit-time samples, coefficients and covariance but cannot
restore fits or establish current-data identity. The remaining NLS and weighted
columns are not part of this comparison.
