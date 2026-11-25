# Current-row Poisson/NB2 probabilities

`predict p, pmf(k)` appends P(Y=k|current X,offset); `cdf(k)` appends
P(Y<=k|current X,offset). The cutoff may be an integer literal0..100000 or a numeric
current-row variable whose nonmissing values are exact integers0..100000. Missing
cutoffs produce SQL NULL; negative, fractional, or greater-than100000 values fail
before a new column is published. For cutoffs above99, one prediction may add at
most10,000,000 cutoff-work units across rows with usable model inputs. SE requests
count recurrence and gradient work; NB2 CDF SEs can charge up to four cutoff
passes. Exceeding the limit fails before publication. These are plug-in distributional probabilities, not
confidence/prediction intervals or a goodness-of-fit test. A robust/cluster
covariance does not justify a Poisson or NB2 law; NB2 uses retained alpha with
variance mu+alpha*mu².

For a current-row PMF or CDF, add `se` to write its pointwise delta-method
standard error instead of the probability, for example `predict p2_se, pmf(2) se`,
`predict cdf2_se, cdf(2) se`, or the corresponding current-cutoff forms. Run the
point prediction separately. The SE uses the covariance selected by the retained
fit; NB2 includes beta/lnalpha covariance. Current predictors and fixed
offsets/exposure are treated as known. These measure parameter uncertainty in
P(Y=k|X) or P(Y<=k|X), not future-count variation or confidence intervals.
Missing/nonfinite model inputs produce SQL NULL; invalid cutoffs, unrepresentable
covariance propagation, or work-limit failures publish no column.

Run in a fresh directory (paths below assume `root` is the repository):

```sh
cp "$root/examples/nb2-offset/nb2_offset.csv" .
"$root/target/debug/tabdat-explore-rs" --text -f "$root/examples/count-probability/predict.td"
```

This synthetic method extension demonstrates both laws, fixed offsets, changed
current rows, scalar and row-specific cutoffs, PMF/CDF parameter standard errors,
and CSV export. The corresponding
RAND Table20.5 mean models are a target application, but this example adds no
published table row or resolution of the printed Table20.6 NB2 discrepancy.
Exports refuse existing files; earlier successful script effects are not rolled back
after a later error.

The retained fit/sample never changes during prediction. Current outcomes and
cluster IDs are not required; the fitted predictor/offset names are. Missing or
nonfinite cells and nonfinite/overflowing indices/means produce SQL NULL. A
finite index whose exponential underflows to zero uses the limiting point mass
at zero. Empty data is supported. Bounds, missing/nonnumeric required columns,
existing targets, unsupported estimator/mode/options or probability convergence
failure fail before publication, preserving data/labels/fit. Bare predict remains
xb; `n` remains the mean and binary `pr` keeps its meaning. Probability
confidence intervals and future-count uncertainty remain unsupported.

Independent base-R4.6.1 `dpois`/`ppois`/`dnbinom`/`pnbinom` validate308 PMF/CDF
quantities over tiny/large/zero means, extreme supported alpha and k=0,1,9,99,
100,1,000 and100,000, including the NB2 fit-domain limit.
The existing audited offset-fit R parameters supply432 fixed-cutoff and144
row-specific current-row reference probabilities after predictor/offset shifts
and reversed order. The CLI checks both laws, three covariance choices, text/JSON
and identical exports. Numerical
tolerances are relative1e-10 for direct helper probabilities and relative1e-8
for workflow predictions (plus MIN_POSITIVE to accommodate underflow). The
underlying estimator/sample/covariance contracts are unchanged.

Regenerate development fixtures from the repository root:

```sh
Rscript crates/tabdat-stats/tests/fixtures/count_point_reference.R
Rscript crates/tabdat-stats/tests/fixtures/count_probability_se_reference.R \
  crates/tabdat-stats/tests/fixtures/count_probability_se_reference.csv
Rscript crates/tabdat-stats/tests/fixtures/count_cdf_se_reference.R \
  crates/tabdat-stats/tests/fixtures/count_cdf_se_reference.csv
```

The PMF SE kernel matches independent base-R central differences
for Poisson (`p=0.125408498627284`, `SE=0.0548118642581173`) and joint NB2
(`p=0.0810312073097155`, `SE=0.0128222345247593`) under full covariance matrices,
including beta/lnalpha cross-covariance. The PMF test uses relative1e-9 plus1e-13
absolute tolerance. The CDF SE kernel matches fifteen base-R `ppois`/`pnbinom`
central-difference cases across lower and upper tails, the largest cutoff, and
both ends of the NB2 alpha domain. Its test uses relative1e-8 plus2e-13 absolute
tolerance, with relative1e-7 for very small SEs. The independent CSV records
probabilities and SEs.

No production Python/R dependency is used.
