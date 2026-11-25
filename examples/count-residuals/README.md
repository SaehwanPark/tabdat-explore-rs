# Count-model response, Pearson and deviance residuals

This synthetic workflow demonstrates current-row `predict` diagnostics after
Poisson and NB2 fits. It is independent method evidence, not a Cameron and
Trivedi textbook illustration or a formal goodness-of-fit test.

`predict target, residuals` returns the raw response residual `y - mu`.
`predict target, pearson` returns an unscaled Pearson residual, using
`(y-mu)/sqrt(mu)` for Poisson and `(y-mu)/sqrt(mu+alpha*mu^2)` for NB2, where
`alpha` is the retained fitted dispersion. It applies no estimated scale
correction. `predict target, deviance` returns the signed square root of the
family unit deviance:

- Poisson: `D = 2 * (y * log(y/mu) - (y-mu))`.
- NB2: `D = 2 * (y * log(y/mu) - (y+1/alpha) * log((1+alpha*y)/(1+alpha*mu)))`.

Both deviance formulas use the zero-count limit for `y * log(y/mu)`. Values are
scored at current outcomes, predictors and fixed offset/exposure with the
retained coefficients and NB2 alpha. The estimation sample does not filter
current rows. Missing/nonfinite model inputs and undefined/nonfinite residuals
produce NULL. A present outcome must remain an exact supported count or the
prediction fails atomically. At a zero fitted mean Pearson is undefined;
deviance is zero when `y=0` and undefined when `y>0`. Covariance choice does not
change residual values. These modes do not provide scale-adjusted,
studentized/standardized or leverage diagnostics.

## Independent reference

[`reference.R`](reference.R) creates a deterministic 36-row data set and uses
Base R `glm` plus `MASS::glm.nb` to write
[`count_residual_reference.csv`](count_residual_reference.csv). It covers both
an offset and equivalent exposure specification. After fitting, it changes
current outcomes, predictors and fixed terms, then computes current means and
all three residuals. Missing current outcomes and predictors are retained as
rows and receive `NA` predictions. The runtime contract compares every output
against this reference for model-based, row-robust and one-way clustered fits.

Regenerate the fixture and reference from this directory with:

```sh
Rscript reference.R
```

This requires R and the CRAN `MASS` package. The generated fixture and reference
are committed so the Rust contract test does not depend on R at test time.

## Run the scripts

Build the CLI, then run the examples in a temporary working directory so their
CSV exports stay outside the repository:

```sh
cargo build
repo=$(git rev-parse --show-toplevel)
work=$(mktemp -d)
cp "$repo/examples/count-residuals/count_residuals.csv" "$work/"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --text -f "$repo/examples/count-residuals/poisson.td")
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --text -f "$repo/examples/count-residuals/nb2.td")
```

The Poisson script uses a one-way cluster fit and offset; the NB2 script uses
row-robust inference and exposure. Both print and export fitted means, response
residuals, Pearson residuals and deviance residuals. Runtime tests also verify
the model-based covariance case, offset/exposure edits, missing inputs, empty
current data, zero-mean boundaries, invalid outcomes and atomic failures.
