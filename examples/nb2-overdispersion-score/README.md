# Robust NB2 overdispersion score method

This synthetic workflow tests the Poisson mean/variance boundary in the NB2
direction after a retained Poisson fit. The null is
`Var(Y_i | X_i) = mu_i`; the one-sided alternative is
`Var(Y_i | X_i) = mu_i + alpha*mu_i^2`, with `alpha > 0`. It uses the retained
Poisson fitted means, including any fixed offset, and tests the NB2 score at
`alpha = 0`. It does not fit or recommend a negative-binomial model.

The per-row score is

```text
u_i = 0.5 * ((y_i - mu_i)^2 - y_i)
```

The total score is standardized by model information `0.5*sum(mu_i^2)`, an
independent-row score sandwich with `N/(N-1)`, or a one-way cluster score
sandwich with `G/(G-1)`. The reference is the upper tail of the asymptotic
standard normal. Cluster inference requires independent groups and enough
groups for this approximation; robust score variance does not validate the
Poisson mean or a complete NB2 distribution.

Run from the repository root after building the CLI and generating the fixture:

```sh
repo="$PWD"
Rscript examples/nb2-overdispersion-score/reference.R
work=$(mktemp -d)
cp examples/nb2-overdispersion-score/score_reference.csv "$work/"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json \
  -f "$repo/examples/nb2-overdispersion-score/model_based.td")
```

The reference independently fits Poisson GLMs and computes the score and each
variance convention in base R 4.6.2. It includes independent NB2-like rows for
model-based and robust checks plus shared-gamma groups for the clustered check.
The CSV is method evidence, not a Cameron–Trivedi textbook replication. The
independent score fixture regenerates byte-identically with the data:

```sh
work=$(mktemp -d)
Rscript examples/nb2-overdispersion-score/reference.R \
  "$work/score_reference.csv" "$work/poisson_nb2_score_reference.csv"
cmp examples/nb2-overdispersion-score/score_reference.csv \
  "$work/score_reference.csv"
cmp crates/tabdat-stats/tests/fixtures/poisson_nb2_score_reference.csv \
  "$work/poisson_nb2_score_reference.csv"
```
