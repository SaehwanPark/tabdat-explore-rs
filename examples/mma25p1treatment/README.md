# NSW/PSID treatment comparison

This bounded workflow reproduces the first two rows of Cameron and Trivedi
(2005), Table 25.4, using the authors' [program](https://cameron.econ.ucdavis.edu/mmabook/mma25p1treatment.do),
[output](https://cameron.econ.ucdavis.edu/mmabook/mma25p1treatment.txt) and
[original data](https://cameron.econ.ucdavis.edu/mmabook/nswpsid.da1). It reports
pre-treatment group means, the 1978 earnings difference, a covariate-adjusted
linear comparison, robust uncertainty and individual fitted values.

The 2,675 people comprise 185 NSW treated participants and 2,490 PSID controls;
the control sample is observational, not the NSW randomized control group.
The unadjusted coefficient is the difference in observed group means. The
adjusted coefficient is the constant treatment indicator coefficient in a
linear model with intercept, age, age squared, education, an indicator for less
than 12 years of education, Black and Hispanic indicators, and 1974 and 1975
real earnings. It is a model-dependent conditional comparison. A causal effect
interpretation additionally requires a well-defined treatment, appropriate
sampling, conditional exchangeability/no unmeasured confounding, positivity and
correct conditional mean specification. Group means show imbalance but do not
establish overlap or remove confounding. The output is not evidence that training
caused either estimate.

Both models use the complete, unweighted, ordered sample and HC1 independent-row
covariance `N/(N-K)` with Student-t `N-K` inference. Their treatment coefficient
estimates are -15,204.776 (SE 655.914) and 217.944 (SE 767.881); rounding of
source ASCII and historical storage can differ. `lincom treat` reports 95%
intervals. The author notes that source U74/U75 are reversed; this bounded
analysis uses neither and does not silently reinterpret them. The source output
labels RE75/RE78 as 1974 in one comment; the variables are 1975 and 1978 earnings.

The executable guards dense ordered IDs, finite analysis fields, binary treatment
and indicator columns, both groups and full-rank estimability. Exports do not
clobber existing files; a failed script is not a transaction. Pinned source
SHA256: `810f6dbfbe0ffbcc52676946c330f49d4d8bf5cafc06980e912ca945ac7f4e44`.
The ASCII data are fetched separately and are not redistributed here.

From a fresh directory:

```sh
python3 /path/to/repo/examples/mma25p1treatment/prepare.py /path/to/nswpsid.da1 nswpsid.csv
/path/to/tabdat-explore-rs --json -f /path/to/repo/examples/mma25p1treatment/comparison.td > result.jsonl
Rscript /path/to/repo/examples/mma25p1treatment/reference.R /path/to/nswpsid.da1 reference
python3 /path/to/repo/examples/mma25p1treatment/check.py /path/to/nswpsid.da1 result.jsonl treatment_predictions.csv reference
```

The development-only base-R reference uses `lm` QR and independently assembled
HC1 covariance. The audit checks all source rows and predictions, both full
covariance matrices, coefficient inference and treatment intervals, group means,
and stored sample/report contents. Numerical tolerance is `1e-8 +
1e-9*abs(reference)` with an additional `1e-7` relative check for tiny nonzero
p-values. R and Python are not runtime dependencies. This slice does not include
the before/after, difference-in-differences, propensity-score matching or other
Chapter 25 analyses, nor a general treatment-effects command.
