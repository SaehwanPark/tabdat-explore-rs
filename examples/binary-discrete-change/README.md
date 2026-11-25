# Binary-response discrete probability changes

This synthetic method example applies a finite 0-to-1 contrast to a binary
predictor after logit or probit. It does not treat a 0/1 predictor as
continuous. The effect is the average fitted-probability difference over the
retained estimation rows:

```text
mean_i [Pr(y_i=1 | x_i, treated=1) - Pr(y_i=1 | x_i, treated=0)]
```

Run the examples from a fresh working directory with the built binary and a
copy of `binary_discrete_change.csv`:

```sh
repo="$PWD"
work=$(mktemp -d)
cp examples/binary-discrete-change/binary_discrete_change.csv "$work/"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json \
  -f "$repo/examples/binary-discrete-change/logit_clustered.td")
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --text \
  -f "$repo/examples/binary-discrete-change/probit_robust.td")
```

`estat margins, discrete(treated)` requires a retained numeric predictor named
`treated` whose observed fit values include exactly 0 and 1. Both counterfactual
rows retain every other observed predictor value. The calculation uses the
model's retained sample after later data edits; it does not refit or read current
rows. It is conditional on the observed design and does not account for
uncertainty in the predictor distribution.

The standard error uses the full delta-method gradient of the average contrast
and the fitted covariance: logit inverse Fisher or probit inverse observed
information; after `robust`, the corresponding independent-row score sandwich
with N/(N-1); after `cluster(group)`, grouped scores with G/(G-1). Inference is
asymptotic normal with a fixed 95% interval. Cluster inference assumes
independent groups and many-group asymptotics; the correction does not provide
a few-cluster guarantee.

This additive fixture supports one exact numeric 0/1 predictor per call. It does
not cover factor-variable encoding, multilevel contrasts, weights or a causal
interpretation. Discrete changes for sources in continuous product terms are
validated separately in
[`binary-interaction-discrete-change`](../binary-interaction-discrete-change/README.md).
The included Base R reference generates the fixed 120-row fixture and
independently checks point changes and delta inference for logit/probit under
model-based, independent-row robust and 20-group covariance.
Regenerate from the repository root:

```sh
work=$(mktemp -d)
Rscript examples/binary-discrete-change/reference.R \
  "$work/binary_discrete_change.csv" > "$work/reference.csv"
cmp examples/binary-discrete-change/binary_discrete_change.csv \
  "$work/binary_discrete_change.csv"
cmp crates/tabdat-stats/tests/fixtures/binary_discrete_change_reference.csv \
  "$work/reference.csv"
```
