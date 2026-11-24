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

## Robust two-, three-, four- and five-instrument confidence sets

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

[`four_instrument_ci.td`](four_instrument_ci.td),
[`four_instrument_ci_clustered.td`](four_instrument_ci_clustered.td) and
[`four_instrument_ci_all_real.td`](four_instrument_ci_all_real.td) extend the
same workflow to four excluded instruments. These 48-row fixtures have K=6,
residual DF=42 and the chi-square(4) critical value 9.48772903678115. The
independent Base R 4.6.2 reference and generated data are in
[`robust_ar_four_ci_reference.R`](../../crates/tabdat-stats/tests/fixtures/robust_ar_four_ci_reference.R).
HC1 and CR1 produce bounded sets for the multiple case, an empty set for the
all-real-labelled case, and a two-component CR1 set with four real boundary
roots. Corrections are 1.14285714285714 for HC1 and 1.22077922077922 for CR1
with 12 groups. The degree-eight method supports up to eight roots and five
accepted components; tests compare independent endpoints and probes, retained
state, text/JSON, rescaling and failure paths.

[`five_instrument_ci.td`](five_instrument_ci.td) and
[`five_instrument_ci_clustered.td`](five_instrument_ci_clustered.td) extend the
same workflow to five excluded instruments. The 100-row fixture has K=7 and
residual DF=93, with a 95% chi-square(5) critical value of 11.0704976935164.
Its independent Base R 4.6.2 reference and generated datasets are in
[`robust_ar_five_ci_reference.R`](../../crates/tabdat-stats/tests/fixtures/robust_ar_five_ci_reference.R).
For the multiple design, HC1 gives [1.06421221080996, 1.39773070439002] and
CR1 with 20 groups gives [1.16236801651081, 1.39363336463301]. Corrections are
1.0752688172043 and 1.12054329371817. A second design yields an empty CR1 set
and a bounded HC1 set. The degree-ten method supports up to ten boundary roots
and six set components. Tests compare endpoints and direct Wald probes,
rescaling, retained state, text/JSON and six-instrument rejection. Six or more
excluded instruments remain unsupported.

## Robust projected coefficient sets

[`projected_ci.td`](projected_ci.td) and
[`projected_ci_clustered.td`](projected_ci_clustered.td) project the full
chi-square(2) joint AR region for two endogenous regressors and two excluded
instruments onto either named coefficient. The other endogenous coefficient
remains unrestricted; no estimate is plugged in. The 48-row synthetic design
has an intercept, one included control, two endogenous regressors, two excluded
instruments and 12 four-row groups. K_aux=4, residual DF=44, and the 95% critical
value is 5.99146454710798. HC1/CR1 corrections are 1.09090909090909 /
1.16528925619835.

The committed data are `projected_ar_data.csv`. The independent development-only
Base R 4.6.2 generator/reference is
[`robust_ar_projected_reference.R`](../../crates/tabdat-stats/tests/fixtures/robust_ar_projected_reference.R).
It computes direct auxiliary-OLS HC1/CR1 Wald statistics, profiles the nuisance
coefficient over a broad pilot grid, refines local minima with `optimize`, and
brackets target endpoints with `uniroot`. Regenerate the data and reference CSV
from the repository root with:

```sh
work=$(mktemp -d)
Rscript crates/tabdat-stats/tests/fixtures/robust_ar_projected_reference.R "$work" \
  > "$work/robust_ar_projected_reference.csv"
cmp examples/robust-ar/projected_ar_data.csv "$work/projected_ar_data.csv"
cmp crates/tabdat-stats/tests/fixtures/robust_ar_projected_reference.csv \
  "$work/robust_ar_projected_reference.csv"
```

The four bounded 95% sets are:

| Covariance | Coefficient | 95% set |
| --- | --- | --- |
| HC1 | x1 | [0.789041366121279, 1.65982451868902] |
| HC1 | x2 | [-1.10737758826908, -0.234526183277661] |
| CR1, 12 groups | x1 | [0.331931575649699, 2.27083168350681] |
| CR1, 12 groups | x2 | [-2.01629567401708, 0.208762765550004] |

The Rust solver partitions target values using polynomial resultant roots and
roots where nuisance-polynomial coefficients vanish; it does not use a finite
search grid. The reported result is the closed conservative envelope, which can
include a target value approached only as the nuisance coefficient diverges.
Numerical boundary or candidate covariance ambiguity yields a marked all-real
fallback; a score covariance singular throughout the null plane is an error.
Polynomial tests additionally cover disjoint and singleton projection shapes.
CR1 needs at least three retained groups and independent groups with many-group
asymptotics; these fixtures do not establish few-cluster reliability or coverage.
This method evidence does not cover the three-endogenous/three-instrument DATA66
design in the textbook schooling example.

Run the examples from a temporary directory after building the binary:

```sh
repo="$PWD"
work=$(mktemp -d)
cp crates/tabdat-stats/tests/fixtures/robust_ar_multi_ci*csv "$work/"
cp crates/tabdat-stats/tests/fixtures/robust_ar_three_ci*csv "$work/"
cp crates/tabdat-stats/tests/fixtures/robust_ar_four_ci*csv "$work/"
cp examples/robust-ar/robust_ar_five_ci*csv "$work/"
cp examples/robust-ar/projected_ar_data.csv "$work/"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/robust-ar/two_instrument_ci.td")
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/robust-ar/two_instrument_ci_clustered.td")
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --text -f "$repo/examples/robust-ar/two_instrument_ci_all_real.td")
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/robust-ar/three_instrument_ci.td")
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/robust-ar/three_instrument_ci_clustered.td")
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --text -f "$repo/examples/robust-ar/three_instrument_ci_six_roots_clustered.td")
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/robust-ar/four_instrument_ci.td")
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/robust-ar/four_instrument_ci_clustered.td")
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --text -f "$repo/examples/robust-ar/four_instrument_ci_all_real.td")
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/robust-ar/five_instrument_ci.td")
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/robust-ar/five_instrument_ci_clustered.td")
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/robust-ar/projected_ci.td")
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --text -f "$repo/examples/robust-ar/projected_ci_clustered.td")
```

Quartic, degree-six, degree-eight and degree-ten boundaries are solved by
recursively isolating derivative roots and checking every intervening region,
without a finite grid.
Repeated or ambiguous roots and singular or ill-conditioned score covariance
fail explicitly. This remains synthetic method evidence, not a
Cameron–Trivedi illustration or a claim of instrument validity. Six or more
excluded instruments, robust projection outside the bounded two-endogenous/
two-instrument slice, larger endogenous systems, HAC/survey covariance and
few-cluster inference remain unsupported.
