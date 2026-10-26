# Exponential-regression marginal effects, Chapter 5.9.4

This bounded workflow reproduces the point-effect calculations in Cameron and
Trivedi (2005), pp. 162–163, using the original generated 10,000-row `mma05data.asc`.
It fits the exponential density and reports the average analytical derivative,
forward numerical derivative, one-unit finite difference, and derivative at mean
x. It does not supply effect uncertainty, a general exponential estimator, or
complete Table 5.7.

The original [author program](https://cameron.econ.ucdavis.edu/mmabook/mma05p4margeffects.do)
and [output](https://cameron.econ.ucdavis.edu/mmabook/mma05p4margeffects.txt)
have SHA256 `b319a2d67d0b77a111c52c12c7c1e4b31f39fe34b10416788317f7f62fb5db51`
and `c859aad5b4cd892ecfcf5ca3e9b8fd4a23773b666237caaad188e55da6fb1e5d`,
respectively. The source ASCII SHA256 is
`460f5d731f1848c8695d78a1fbf0db348d0daced4b8b5b0c2734bffa20b3e31c`.
The existing preparation script verifies that hash and preserves all source tokens
and ordinals; data and author files are acquired separately.

For `y>0`, rate `lambda=exp(a+b*x)` and mean `mu=1/lambda`, the density
log likelihood is `sum(a+b*x-y*lambda)`. Fitting the existing Poisson model to
an all-one response with offset `log(y)` has the same score and observed
information, and its log likelihood differs by the data-only constant
`sum(log(y))`. The intercept and slope therefore have exponential-density
model-based covariance under the correctly specified independent exponential
model. The native result is still labeled Poisson: its likelihood, deviance,
Pearson dispersion and other count diagnostics are **not** exponential-model
adequacy measures. No count distribution is assumed for the source data.

The sample is all source rows in source order; finite x, positive finite y,
unique dense ordinals, nonconstant x and 3–10,000 observations are enforced.
The current fit requires full rank, convergence and representable predictions.
For each row the script predicts at x, x+1, x+h and mean(x), restores x, and
computes `-b*mu`, `(mu(x+h)-mu(x))/h`, `mu(x+1)-mu(x)`, and
`-b*mu(mean(x))`, with `h=0.0001*sd(x)` using sample standard deviation.
The slope is recovered from the ratio of the two positive integrated-rate
predictions at x+1 and x. Output values must be finite before export. Scripts
are sequential, not transactional; output uses existing no-clobber semantics.

Independent R 4.6.1 profiles the exponential-density intercept analytically
and solves the slope score with `uniroot`; it computes inverse observed
information and all effects directly. On the pinned original data the fitted
(a,b) are (1.9829208730721246, -0.9896275707485669). The replay verifies all
10,000 fit indices, all 90,000 numeric exported fields, covariance and four
summary values. The primary field tolerance is `2e-11*(1+abs(reference))`;
the subtraction-sensitive forward derivative uses `2e-10*(1+abs(reference))`.
No tolerance was widened to reconcile the author output. Summary values are:

| Effect | Native double precision | Author printed |
| --- | ---: | ---: |
| Average analytical derivative | 0.6097659753 | 0.60976598 |
| Average forward numerical derivative | 0.6097962964 | 0.60949044 |
| Average one-unit difference | 1.0414485376 | 1.0414485 |
| Derivative at mean x | 0.3717419999 | 0.371742 |

The historical numerical derivative differs materially; its cause is not
established here and historical precision parity is not claimed. A finite
step is not the analytical derivative, and these effects are descriptive model
projections, not causal effects or uncertainty estimates.

Run from a fresh directory with the original source and a built binary:

```sh
python3 /path/to/repo/examples/mma05p2nls/prepare.py /path/to/mma05data.asc nls.csv
/path/to/tabdat-explore-rs --json -f /path/to/repo/examples/mma05p4margeffects/workflow.td > results.jsonl
Rscript /path/to/repo/examples/mma05p4margeffects/reference.R /path/to/mma05data.asc reference.csv
python3 /path/to/repo/examples/mma05p4margeffects/check.py /path/to/mma05data.asc results.jsonl exponential_effects.csv reference.csv
python3 /path/to/repo/examples/mma05p4margeffects/check_failures.py /path/to/tabdat-explore-rs
```

JSON/text CSVs were byte-identical. Invalid response, duplicate/missing source
identity and constant design each fail before export. This evidence is scoped
to the original sample and the documented calculations; it does not establish
precision or convergence on arbitrary designs.
