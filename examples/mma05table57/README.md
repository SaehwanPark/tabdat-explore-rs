# Combined Table 5.7 comparison

This bounded composition compares the five specifications from Cameron and
Trivedi (2005), Table 5.7, using the original generated 10,000-row `y,x` data.
It produces two five-column named-report tables, one with working covariance
and one with independent-row robust covariance, plus original-scale predictions
and descriptive diagnostics. It is not a literal reproduction of all historical
output or a general weights API.

The pinned source SHA256 is
`460f5d731f1848c8695d78a1fbf0db348d0daced4b8b5b0c2734bffa20b3e31c`.
All rows must be ordered and uniquely identified by `source_row`, with positive
finite y, finite nonconstant x and y, and 3–10,000 observations. Fitting also
requires full rank, convergence and representable predictions and diagnostics.
The script stops on failure; earlier session changes are not rolled back, and
export refuses to overwrite an existing file.

OLS estimates the linear projection `y=intercept+x*beta`. Exponential ML uses
`lambda=exp(intercept+x*beta)` and mean `1/lambda`. Its all-one Poisson fit with
fixed `ln(y)` offset has the same rate score and information; its reported
count-pmf likelihood is **not** the exponential-density likelihood. The script
computes the latter separately. NLS fits `mean=exp(-a-b*x)`. WNLS and FGNLS
fix weights `1/mean` and `1/mean^2`, respectively, using the preliminary NLS
fit, and estimate transformed responses. Their named-report fit statistics
refer to the transformed response. The separate level R-squared values use
original y and predictions and are descriptive, not cross-family tests. Density
log likelihoods evaluated at NLS estimates assume an exponential density; they
are not NLS likelihoods. Coefficient names are preserved: OLS and Poisson use
`intercept,x`, while the three NLS fits use `a,b` (the exponential-rate and NLS
parameterizations agree in sign). Names are not silently aligned across models.

OLS uses classical or HC1 `N/(N-2)` covariance and Student-t `N-2` inference;
Poisson uses model information or independent-row `N/(N-1)` score sandwich and
normal inference. NLS uses working homoskedastic Gauss–Newton or HC1
`N/(N-2)` Gauss–Newton covariance with approximate Student-t `N-2` inference.
Weights are held fixed, without iterating or accounting for first-fit weight
estimation. See the [OLS/exponential](../mma05p1mle/README.md) and
[weighted NLS](../mma05p2nls/weighted.md) descriptions for assumptions and
historical output limitations.

From a fresh output directory (R and Python are development-only):

```sh
python3 /path/to/repo/examples/mma05p2nls/prepare.py /path/to/mma05data.asc nls.csv
/path/to/tabdat-explore-rs --json -f /path/to/repo/examples/mma05table57/comparison.td > result.jsonl
Rscript /path/to/repo/examples/mma05p1mle/reference.R /path/to/mma05data.asc linear-reference
Rscript /path/to/repo/examples/mma05p2nls/reference.R nls-reference /path/to/mma05data.asc
Rscript /path/to/repo/examples/mma05p2nls/weighted_reference.R weighted-reference /path/to/mma05data.asc
python3 /path/to/repo/examples/mma05table57/check.py /path/to/mma05data.asc result.jsonl table57_predictions.csv linear-reference .
```

The independent base-R references use QR and a profiled likelihood or nonlinear
root, and analytic covariance calculations. The audit checks the source identity,
all ordered rows and exported projections, all ten fits' coefficients and full
covariance matrices, snapshot/sample provenance, and level diagnostics. Its
primary absolute-plus-relative tolerance is `1e-11 + 1e-10*abs(reference)`;
nonzero p-values below `1e-20` require `1e-8` relative agreement. The analysis
uses source ASCII values rather than claiming exact reproduction of Stata's
original in-memory floats. Other Chapter 5 illustrations, historical numerical
marginal-effect differences, and broader model and inference workflows remain.
