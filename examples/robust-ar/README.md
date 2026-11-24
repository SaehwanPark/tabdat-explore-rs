# Robust joint Anderson–Rubin inference

This synthetic method workflow fills a graduate-IV inference requirement; it is
not a newly replicated Cameron–Trivedi illustration. It uses 72 source rows,
two endogenous regressors, one included exogenous control,
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
row/group order, and hold the adjusted outcome fixed while removing first-stage
relevance entirely. This checks the absence of a relevance denominator; the
fixture itself is not a simulation of weak-IV size or power. Runtime/CLI tests check retained behavior after edits, output
metadata, null errors, missing groups and rank failures, script stopping before
later file effects, and unchanged fit/data state. The existing classical F and
homoskedastic scalar/projected confidence-set tests remain in the suite beside
the robust scalar confidence-set tests documented below.

Weak-identification robustness does not relax instrument validity or correct-model
assumptions. HC1 requires independent observations; CR1 permits within-group
dependence but requires independent groups and many-group asymptotics. Twelve
synthetic groups validate arithmetic, not nominal coverage or few-cluster
accuracy. There is no automatic covariance-rank reduction: rank-limited joint
tests fail. The joint-test slice does not supply multiple-instrument robust
confidence sets, endogenous-subvector inference, HAC/survey inference,
overidentification or instrument-validity tests.

## Robust scalar confidence set

The separate [`scalar_ci.td`](scalar_ci.td) and
[`scalar_ci_clustered.td`](scalar_ci_clustered.td) workflows invert the HC1 or
one-way CR1 auxiliary Wald test for exactly one endogenous regressor and one
excluded instrument. Included exogenous coefficients remain unrestricted. This
is synthetic method evidence, not a Cameron–Trivedi illustration or a weak-IV
coverage simulation.

From the repository root:

```sh
repo="$PWD"
work=$(mktemp -d)
cp crates/tabdat-stats/tests/fixtures/robust_ar_ci_data.csv "$work/"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --text -f "$repo/examples/robust-ar/scalar_ci.td")
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/robust-ar/scalar_ci_clustered.td")
```

The base R 4.6.2 generator and independent reference are
[`robust_ar_ci_reference.R`](../../crates/tabdat-stats/tests/fixtures/robust_ar_ci_reference.R).
It uses direct `lm.fit`, explicit observation/group score sandwiches and
`polyroot`; R is development-only. Source rows 2 and 8 are missing the outcome
and instrument, so HC1 retains 78 of 80 rows. The clustered script also excludes
source row 17 for its missing group, retaining 77 rows and 10 groups. Both
scripts change the current data after fitting and report the same set again,
confirming that inference reuses the retained sample and does not refit.

| Covariance | Auxiliary K / residual DF | Correction | 95% set |
| --- | ---: | ---: | --- |
| HC1 | 3 / 75 | 1.04 | [1.36445194119086, 1.96845342740648] |
| CR1, 10 groups | 3 / 74 | 1.14114114114114 | [1.20537141708336, 2.13637942939771] |

The reference distribution is asymptotic chi-square(1), with critical value
3.84145882069413. The one-instrument test inversion is a quadratic inequality;
the implementation solves it analytically without a finite grid. Tests check
sample indices, corrections, endpoints, test membership and retained-state
behavior within absolute 1e-10. HC1 assumes independent rows. CR1 assumes
independent groups with many-group asymptotics; neither gives finite-sample
coverage guarantees. This one-instrument result does not cover multiple
endogenous regressors or robust projection.

## Robust two- and three-instrument confidence sets

[`two_instrument_ci.td`](two_instrument_ci.td) and
[`two_instrument_ci_clustered.td`](two_instrument_ci_clustered.td) use one
endogenous regressor and two excluded instruments. The auxiliary regression
has K=4 and residual DF=16; the joint null uses chi-square(2), with 95% critical
value 5.99146454710798. The deterministic 20-row fixtures and independent base
R reference are in
[`robust_ar_multi_ci_reference.R`](../../crates/tabdat-stats/tests/fixtures/robust_ar_multi_ci_reference.R).
HC1 returns two rays and a bounded component; CR1 returns two rays. The
all-real scripts retain a small nonzero first-stage loading.

[`three_instrument_ci.td`](three_instrument_ci.td) and
[`three_instrument_ci_clustered.td`](three_instrument_ci_clustered.td) extend
the same workflow to three excluded instruments. These 36-row fixtures have
K=5, residual DF=31, and the chi-square(3) critical value 7.81472790325118.
The independent Base R 4.6.2 reference and generated data are in
[`robust_ar_three_ci_reference.R`](../../crates/tabdat-stats/tests/fixtures/robust_ar_three_ci_reference.R).
It validates a three-component HC1 set, a disjoint CR1 set, an all-real HC1
set, and a six-root CR1 boundary with four accepted components. The last case
is runnable with
[`three_instrument_ci_six_roots_clustered.td`](three_instrument_ci_six_roots_clustered.td).
Tests compare all endpoints and direct Wald probes, corrections, rank and
rescaling behavior, retained fit state, and text/JSON reporting. HC1 uses
correction 1.16129032258065; CR1 uses 1.2316715542522 for 12 groups.
R is development-only; native scripts need only the committed CSV and Rust
binary. See [`multiple-instrument.md`](multiple-instrument.md) for reference
endpoints and statistical limits.

Run the examples from a temporary directory after building the binary:

```sh
repo="$PWD"
work=$(mktemp -d)
cp crates/tabdat-stats/tests/fixtures/robust_ar_multi_ci*csv "$work/"
cp crates/tabdat-stats/tests/fixtures/robust_ar_three_ci*csv "$work/"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/robust-ar/two_instrument_ci.td")
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/robust-ar/two_instrument_ci_clustered.td")
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --text -f "$repo/examples/robust-ar/two_instrument_ci_all_real.td")
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/robust-ar/three_instrument_ci.td")
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/robust-ar/three_instrument_ci_clustered.td")
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --text -f "$repo/examples/robust-ar/three_instrument_ci_six_roots_clustered.td")
```

Quartic and degree-six boundaries are solved by recursively isolating
derivative roots and checking every intervening region, without a finite grid.
Repeated or ambiguous roots and singular or ill-conditioned score covariance
fail explicitly. This remains synthetic method evidence, not a
Cameron–Trivedi illustration or a claim of instrument validity. Four or more
excluded instruments, multiple endogenous regressors, robust projection,
HAC/survey covariance and few-cluster inference remain unsupported.
