# Robust joint Anderson–Rubin inference

This synthetic method workflow fills a graduate-IV inference requirement; it is
not a newly replicated Cameron–Trivedi illustration. It uses 72 source rows,
two weakly instrumented endogenous regressors, one included exogenous control,
three excluded instruments and an intercept. The supplied joint null is
`(x1,x2)=(1.7,-0.6)`. There are no weights, factors or endogenous nuisance
coefficients. The deterministic generator and independent base-R reference are
[robust_ar_reference.R](../../crates/tabdat-stats/tests/fixtures/robust_ar_reference.R).

## Run

From the repository root:

```sh
cargo build --locked --bin tabdat-explore-rs
repo="$PWD"
work=$(mktemp -d)
cp crates/tabdat-stats/tests/fixtures/robust_ar_data.csv "$work/"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --text -f "$repo/examples/robust-ar/analysis.td")
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/robust-ar/analysis.td")
Rscript crates/tabdat-stats/tests/fixtures/robust_ar_reference.R "$work"
```

R is development-only. The native workflow needs only the committed CSV and Rust
binary. Compare the generated reference/data CSVs with their committed fixtures.
Source rows 2 (outcome) and 8 (instrument) are missing: HC1 retains 70 records in
source order. Cluster inference additionally excludes source row 17 (missing
group), retaining 69 records and all 12 interleaved groups. The tests check exact
retained source indices, including this difference, rather than merely N.

## Meaning and numerical evidence

Under the complete null, `y − 1.7*x1 + 0.6*x2` is regressed on `1,w,z1,z2,z3`.
All three excluded-instrument coefficients are jointly zero under the maintained
linear model and valid instruments. The unrestricted auxiliary residuals supply
the row or grouped scores. The auxiliary parameter count is 5, whereas the
parent IV parameter count is 4. HC1 multiplies the sandwich by `N/(N−5)`;
CR1 multiplies it by `G/(G−1)*(N−1)/(N−5)`. The full three-coefficient Wald
statistic has an **asymptotic chi-square(3)** reference. No F or G−1 t reference is
substituted. This follows the auxiliary-regression AR construction described in
[Stata's IV postestimation methods](https://www.stata.com/manuals/rivregresspostestimation.pdf),
with the corrections here explicitly selected; exact software-default parity is
not claimed.

| Covariance | Correction | Wald chi-square(3) | Upper-tail p |
| --- | ---: | ---: | ---: |
| HC1 | 1.07692307692308 | 3.53464562620592 | 0.316296420698153 |
| One-way CR1 | 1.15909090909091 | 35.2116876251244 | 1.099029699071e-7 |

Base R4.6.1 `lm.fit`, explicit sandwich matrix operations and `pchisq` independently
validate all six controlled/no-intercept/no-exogenous-control × HC1/cluster cases.
Correction, statistic and tail tolerances are 1e-10 relative (a 1e-30 numerical
floor for the smallest tail). The tests also vary instrument units and reverse
row/group order. Runtime/CLI tests check retained behavior after edits, output
metadata, null errors, missing groups and rank failures, script stopping before
later file effects, and unchanged fit/data state. The existing classical F and
scalar confidence-set tests remain in the suite.

Weak-identification robustness does not relax instrument validity or correct-model
assumptions. HC1 requires independent observations; CR1 permits within-group
dependence but requires independent groups and many-group asymptotics. Twelve
synthetic groups validate arithmetic, not nominal coverage or few-cluster
accuracy. There is no automatic covariance-rank reduction: rank-limited joint
tests fail. This slice supplies no robust confidence sets, endogenous-subvector
inference, HAC, survey, overidentification or instrument-validity test.
