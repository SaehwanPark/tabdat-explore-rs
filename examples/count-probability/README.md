# Current-row Poisson/NB2 probabilities

`predict p, pmf(k)` appends P(Y=k|current X,offset); `cdf(k)` appends
P(Y<=k|current X,offset). The cutoff is an integer literal0..99, reusing the
existing bounded probability helper through its first right-tail count1..100.
These are plug-in distributional probabilities, not confidence/prediction
intervals or a goodness-of-fit test. A robust/cluster covariance does not justify
a Poisson or NB2 law; NB2 uses the retained alpha with variance mu+alpha*mu².

Run in a fresh directory (paths below assume `root` is the repository):

```sh
cp "$root/examples/nb2-offset/nb2_offset.csv" .
"$root/target/debug/tabdat-explore-rs" --text -f "$root/examples/count-probability/predict.td"
```

This synthetic method extension demonstrates both laws, fixed offsets, changed
current rows and CSV export. The corresponding documented RAND Table20.5 mean
models are a target application, but this example adds no published table row
or resolution of the printed Table20.6 NB2 discrepancy. Exports refuse existing
files; earlier successful script effects are not rolled back after a later error.

The retained fit/sample never changes during prediction. Current outcomes and
cluster IDs are not required; the fitted predictor/offset names are. Missing or
nonfinite cells and nonfinite/overflowing indices/means produce SQL NULL. A
finite index whose exponential underflows to zero uses the limiting point mass
at zero. Empty data is supported. Bounds, missing/nonnumeric required columns,
existing targets, unsupported estimator/mode/options or probability convergence
failure fail before publication, preserving data/labels/fit. Bare predict remains
xb; `n` remains the mean and binary `pr` keeps its meaning. No arbitrary-cutoff,
current-variable cutoff, survival option or parameter uncertainty is implied.

Independent base-R4.6.1 `dpois`/`ppois`/`dnbinom`/`pnbinom` validate288 PMF/CDF
quantities over tiny/large/zero means, extreme supported alpha and k=0,1,9,99.
The existing audited offset-fit R parameters supply432 current-row reference
probabilities after predictor/offset shifts and reversed order. The CLI checks
both laws, three covariance choices, text/JSON and identical exports. Numerical
tolerances are relative1e-10 for direct helper probabilities and relative1e-8
for workflow predictions (plus MIN_POSITIVE to accommodate underflow). The
underlying estimator/sample/covariance contracts are unchanged.

Regenerate development fixtures from the repository root:

```sh
Rscript crates/tabdat-stats/tests/fixtures/count_point_reference.R
```

No production Python/R dependency is used.
