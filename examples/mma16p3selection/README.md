# MMA §16.6 RAND selection

This bounded workflow fits the March 2005 author-program RAND illustration in
`mma16p3selection.do`, comparing the positive-spending probit plus selected-case
OLS, explicit Heckman two-step, and the default joint IID Gaussian Heckman ML.
It preserves the author's year-2 restriction and ordered prepared ASCII export;
the original `.dta` file is neither needed nor committed.

## Pinned source and preparation

The three public author files are pinned by SHA-256 so a changed upstream file
cannot silently become the numerical reference:

| File | Author URL | SHA-256 |
| --- | --- | --- |
| `mma16p3selection.do` | <https://cameron.econ.ucdavis.edu/mmabook/mma16p3selection.do> | `f7a37a6f67c0a9e9ed812d92cad19750d57bf09928995fda0536308dd78af2be` |
| `mma16p3selection.txt` | <https://cameron.econ.ucdavis.edu/mmabook/mma16p3selection.txt> | `8de79b87b7667f68ca762e611bd5a7382aa20c9997a921e29ea81419dfb84be6` |
| `mma16p3selection.asc` | <https://cameron.econ.ucdavis.edu/mmabook/mma16p3selection.asc> | `2fde2032f8f6319731ee56b40e87f901abcca9f2072211b61cedd6dab00c37e8` |

The `.do` program keeps year 2 and drops the single observation without
education before exporting 5,574 rows in 20 fields. There are 4,281 rows with
`DMED=1` and observed `LNMED`, and 1,293 with `DMED=0` and missing `LNMED`;
`DMED` agrees with `MED>0`. The ASCII export wraps records across physical
lines, so the converter validates the complete 111,480-token stream and restores
source-row order. The printed `LNMED` values have six-decimal precision and can
differ from `log(MED)` by up to about 3.3e-6; estimation uses the supplied
`LNMED`, not a reconstructed logarithm.

## Reproduce and compare

From a clean temporary directory, fetch and fingerprint the source, prepare the
CSV, run both application outputs, and create the separate base-R reference:

```sh
repo=/path/to/tabdat-explore-rs
work=/tmp/mma16p3selection
mkdir -p "$work"
cd "$work"
base=https://cameron.econ.ucdavis.edu/mmabook
curl -fsSLO "$base/mma16p3selection.do"
curl -fsSLO "$base/mma16p3selection.txt"
curl -fsSLO "$base/mma16p3selection.asc"
printf '%s\n' \
  'f7a37a6f67c0a9e9ed812d92cad19750d57bf09928995fda0536308dd78af2be  mma16p3selection.do' \
  '8de79b87b7667f68ca762e611bd5a7382aa20c9997a921e29ea81419dfb84be6  mma16p3selection.txt' \
  '2fde2032f8f6319731ee56b40e87f901abcca9f2072211b61cedd6dab00c37e8  mma16p3selection.asc' \
  > source.sha256
shasum -a 256 -c source.sha256
awk -f "$repo/examples/mma16p3selection/mma16p3selection_to_csv.awk" \
  mma16p3selection.asc > mma16p3selection.csv
cp "$repo/examples/mma16p3selection/selection.td" .
cp "$repo/examples/mma16p3selection/selection_json.td" .
cargo build --locked --manifest-path "$repo/Cargo.toml" --bin tabdat-explore-rs
Rscript "$repo/examples/mma16p3selection/reference_selection.R" \
  mma16p3selection.asc reference_quantities.csv reference_predictions.csv
"$repo/target/debug/tabdat-explore-rs" --text -f selection.td > native.txt
"$repo/target/debug/tabdat-explore-rs" --json -f selection_json.td > native.jsonl
python3 "$repo/examples/mma16p3selection/check_reference.py" .
```

The R script uses only base R: `glm.fit`/observed probit information, selected
OLS, the explicit two-step correction, an independently coded joint likelihood
and analytic score optimized with `optim`, and a central finite difference of
that score for the full observed information. `check_reference.py` verifies all
source fingerprints and cells, exact sample positions, both JSON estimator
results, all 38×38 joint covariance entries, the author log-likelihood/scale/
correlation benchmark, and all ten source-row point projections. The R and Rust
optimizers stop at slightly different points: this run's largest ML coefficient
difference is 5.6e-6 and largest ML prediction difference is 7.4e-6; the checker
uses a 1e-5 point tolerance for those quantities and a 2e-8 tolerance for full
covariance entries. Probit, OLS, and explicit two-step quantities use tighter
1e-8 gates. These are explicit optimizer/reference tolerances, not author-data
rounding applied to the Rust result.

## Reading the estimates

The author program reports joint ML log likelihood −10,170.11, `sigma` about
1.570053, `rho` about 0.735598, and the same-sample `rho=0` likelihood-ratio
statistic about 27.93 on one degree of freedom. The model uses the same 17
regressors in both equations and has no excluded selection predictor. Its
identification therefore depends on Gaussian functional form and may be weak;
these are latent-mean associations, not causal effects. The IID model-based
covariance does not account for household clustering.

The program's `yexpected` is a zero-filled contribution for log spending, not an
observed log-spending mean. The example exports `xb`, selection probability,
inverse Mills, and conditional-selected outcome projections for ML and
two-step, but does not use `yexpected` to interpret expenditure. General
two-part estimation, robust/cluster inference, prediction uncertainty,
causal identification, and other Chapter 16 illustrations remain outside this
bounded workflow.
