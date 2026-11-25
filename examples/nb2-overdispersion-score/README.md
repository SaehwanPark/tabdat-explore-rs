# Poisson NB1/NB2 overdispersion score methods

This synthetic workflow tests the Poisson mean/variance boundary after a
retained Poisson fit. The default `estat overdispersion` tests the NB2 direction
`Var(Y_i | X_i) = mu_i + alpha*mu_i^2`; `estat overdispersion, nb1` tests the
linear NB1 direction `Var(Y_i | X_i) = mu_i + alpha*mu_i`. Both use `alpha > 0`
alternatives and the retained Poisson fitted means, including any fixed offset.
They are mean-variance score tests, not NB1/NB2 likelihood fits or validation
of a complete negative-binomial distribution.

The per-row score is `0.5*((y_i-mu_i)^2-y_i)` for NB2 and
`((y_i-mu_i)^2-y_i)/(2*mu_i)` for NB1. NB1-direction testing requires positive
retained means. The total score uses NB2 model information
`0.5*sum(mu_i^2)` or NB1 information `N/2`; robust fits use the row-score
sandwich with `N/(N-1)`, and `cluster(group)` aggregates scores before applying
`G/(G-1)`. The signed statistic uses the upper tail of the asymptotic standard
normal. Cluster inference assumes independent groups and enough groups for this
approximation; robust score variance does not validate the Poisson mean.

The Base R reference independently fits the Poisson mean and computes both
score directions and all three variance conventions. The included fixture is
method evidence, not a Cameron–Trivedi textbook replication.

Run from the repository root after building the CLI and generating the fixture:

```sh
repo="$PWD"
Rscript examples/nb2-overdispersion-score/reference.R
work=$(mktemp -d)
cp examples/nb2-overdispersion-score/score_reference.csv "$work/"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json \
  -f "$repo/examples/nb2-overdispersion-score/model_based.td")
```

Replace `model_based.td` with `robust.td` or `clustered.td` to inspect the two
score-sandwich variants. Use `nb1.td` to select the linear NB1 direction. All
scripts read the same generated fixture and retain the fitted offset in their
Poisson means.

The reference independently fits Poisson GLMs and computes each score and
variance convention in base R 4.6.2. It includes independent gamma-Poisson rows
for model-based and robust checks plus shared-gamma groups for the clustered
check. Both reference CSVs regenerate byte-identically with the data:

```sh
work=$(mktemp -d)
Rscript examples/nb2-overdispersion-score/reference.R \
  "$work/score_reference.csv" "$work/poisson_nb2_score_reference.csv" \
  "$work/poisson_nb1_score_reference.csv"
cmp examples/nb2-overdispersion-score/score_reference.csv \
  "$work/score_reference.csv"
cmp crates/tabdat-stats/tests/fixtures/poisson_nb2_score_reference.csv \
  "$work/poisson_nb2_score_reference.csv"
cmp crates/tabdat-stats/tests/fixtures/poisson_nb1_score_reference.csv \
  "$work/poisson_nb1_score_reference.csv"
```

The NB1 reference uses a tighter R GLM stopping tolerance (`epsilon=1e-14`)
because its score divides by the fitted mean and responds to tiny coefficient
stopping differences.
