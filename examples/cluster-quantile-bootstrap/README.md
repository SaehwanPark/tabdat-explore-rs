# Whole-cluster quantile bootstrap

This synthetic workflow demonstrates joint conditional quantiles with the
whole-cluster pairs bootstrap. It retains all complete-finite rows from each
sampled cluster, draws the same 20 clusters for each requested quantile, and
uses the B-1 covariance and marginal type-7 percentile intervals. The export
shows that the bootstrap report leaves the preceding OLS predictions available.

The fixture has 103 candidate rows. Missing outcome, predictor and cluster-key
rows are excluded, leaving 100 observations in 20 first-appearance-ordered
clusters with sizes `4,5,6,5` repeated. The 80 fixed cluster draws produce
replicate sample sizes from 93 to 108. Create a fresh working directory so the
script's export destination is unused:

```sh
mkdir /tmp/tabdat-cluster-qreg
cp /path/to/repo/crates/tabdat-stats/tests/fixtures/cluster-quantile-bootstrap/inputs.csv /tmp/tabdat-cluster-qreg/data.csv
cd /tmp/tabdat-cluster-qreg
/path/to/repo/target/debug/tabdat-explore-rs --json -f /path/to/repo/examples/cluster-quantile-bootstrap/cluster_quantiles.td > cluster_quantiles.jsonl
```

This is method evidence on generated data, not the Vietnam household survey
analysis. The method assumes independent sampled clusters and many-cluster
asymptotics. The three-cluster minimum is only an admission rule; no few-cluster
or survey-design inference is claimed. The `cluster()` method here is whole
cluster resampling. It is distinct from the Hagemann wild-gradient cluster
bootstrap exposed by `R quantreg::boot.rq`'s `cluster` argument.

## Independent reference

The test fixture's R script uses R 4.6.1 and `quantreg` 6.1 `rq(method="br")`
with the committed cluster draw matrix. Recreate the reference from the
repository root:

```sh
mkdir /tmp/tabdat-cluster-qreg-reference
Rscript crates/tabdat-stats/tests/fixtures/cluster-quantile-bootstrap/cluster_quantile_reference.R /tmp/tabdat-cluster-qreg-reference > /tmp/tabdat-cluster-qreg-reference/reference.csv
cmp /tmp/tabdat-cluster-qreg-reference/cluster_quantile_inputs.csv crates/tabdat-stats/tests/fixtures/cluster-quantile-bootstrap/inputs.csv
cmp /tmp/tabdat-cluster-qreg-reference/reference.csv crates/tabdat-stats/tests/fixtures/cluster-quantile-bootstrap/reference.csv
```

The Rust stats tests compare 54 coefficient summary values, all 81 joint
covariance entries and six sample/draw metadata values at absolute tolerance
`2e-8`. A separate two-quantile run compares the slope contrast's estimate,
B-1 standard error, asymptotic normal statistic/p-value and normal-Wald interval
against the same R coefficient draws. Runtime and CLI tests cover parser
admission, complete-case cluster handling, JSON/text output, deterministic
seeds, few-cluster failure and preservation of the retained fit.
