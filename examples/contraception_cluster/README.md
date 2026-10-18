# One-way clustered binary inference: Bangladesh contraceptive use

This is **external method evidence**, not a Cameron–Trivedi textbook replication.
It validates pooled logit/probit with district-clustered uncertainty. It does
not reproduce the random-effects models shown in the source package's examples.

## Exact sample and statistical contract

- Source: [Rdatasets `mlmRev/Contraception.csv`](https://vincentarelbundock.github.io/Rdatasets/csv/mlmRev/Contraception.csv),
  [documentation](https://vincentarelbundock.github.io/Rdatasets/doc/mlmRev/Contraception.html).
  Documentation identifies the 1988 Bangladesh Fertility Survey and cites Huq
  and Cleland's report with 1989 in its title. Original source is Bristol's
  multilevel-modeling data collection. Data remain external; this repository
  does not redistribute them or assert redistribution rights.
- CSV SHA-256: `dd76de5f4f1fb57081b01ef0f81581cd928ad545d13feb8bf7d337d71e690034`.
- All **1,934 women**, sorted by the original `woman` ID (1..1934); **759** report
  `use == "Y"`, 1,175 `"N"`. Outcome y is 1/0 respectively. No imputation,
  weights, missing/nonfinite exclusions or additional selection occur here.
- Predictors: intercept, then the provided `age` in years, already centered in
  the source. Do not recenter it or invent the omitted centering constant.
- Cluster by `district`: **60** distinct districts, codes spanning 1..61.
  The maximum ID is not the cluster count. Retained indices are 0..1933,
  with cluster keys aligned to those exact people.
- Both fits maximize the pooled Bernoulli likelihood. Covariance is
  `G/(G-1) * B * sum_g(S_g S_g') * B`, where `S_g=sum_{i in g}s_i` and B is
  inverse observed information at the converged fit (Fisher for logit).
  There is **no** linear-model `(N-1)/(N-K)` factor. This is the ML correction
  in [Stata User Guide §20.22.2](https://www.stata.com/manuals/u20.pdf).
- Coefficient/95% interval and continuous-age AME uncertainty use asymptotic
  **normal**, not G-1 Student t, inference. The AME is conditional on retained
  X, using the full delta gradient and clustered covariance. Its estimand is
  a descriptive predictor-column derivative, not a causal age effect.
- Clusters must be independent; this correction does not guarantee good
  coverage with few/imbalanced clusters. At least two retained clusters is
  only a computability guard. This is not survey/design-based inference.
- Ordinary LR p/reference are unavailable; likelihood differences and pseudo-R²
  remain descriptive. The omnibus Wald test of all non-intercept coefficients
  uses the selected clustered covariance and asymptotic chi-square(q), not F;
  rank-limited/singular restrictions are unavailable, without reduced-q tests.
  Multiway clustering, weights and random-effects estimation are not supplied.

## Acquire, run and independently validate

From the repository root (curl, Rust, development-only base R and Python's
standard library; neither R nor Python is needed by the production application):

```sh
(
set -eu
cargo build --locked --bin tabdat-explore-rs
repo="$PWD"
work=$(mktemp -d)
curl -fsSL https://vincentarelbundock.github.io/Rdatasets/csv/mlmRev/Contraception.csv \
  -o "$work/Contraception.csv"
printf '%s  %s\n' \
  dd76de5f4f1fb57081b01ef0f81581cd928ad545d13feb8bf7d337d71e690034 \
  "$work/Contraception.csv" | shasum -a 256 -c -
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json \
  -f "$repo/examples/contraception_cluster/clustered_binary.td" > native.jsonl)
Rscript crates/tabdat-stats/tests/fixtures/binary_robust_reference.R --cluster \
  "$work/Contraception.csv" "$work/R_predictions.csv" > "$work/R_reference.csv"
python3 examples/mma14p1binary/check_robust_reference.py --cluster \
  "$work/R_predictions.csv" "$work/clustered_predictions.csv" \
  "$work/R_reference.csv" "$work/native.jsonl"
printf 'Outputs: %s\n' "$work"
)
```

Use a fresh directory for each replay: export refuses to overwrite an existing
file. For readable output replace `--json` with `--text`; the same commands run
in the no-argument persistent terminal session. Prediction uses current predictor
rows and does not require a district/outcome column. `estat margins` keeps its
original fit sample, groups and covariance even after current-data edits.

R **4.6.1** `glm`, independently reevaluated scores/observed information and
matrix algebra validate all **50** coefficient/covariance/normal-inference/delta
and omnibus-Wald quantities. All **1,934** person/district/outcome/design/probability/index records
and retained group identities are checked, not just counts. Absolute tolerance
is 1e-10 for moderate values/predictions and 1e-12 for the prepared age design;
test tails use 1e-10 relative plus the smallest positive normal f64. No tolerance
is widened to conceal differences. Eight clustered synthetic family/design
fixtures additionally cover intercept/no-intercept, one/two predictors and
unbalanced/noncontiguous groups. Their grouped-score meats independently match
`sandwich 3.1.1::meatCL(type="HC0", cadjust=TRUE)` after refreshing glm's working
quantities at its returned coefficients, avoiding its cached-iterate lag.

| Quantity | Logit | Probit |
| --- | ---: | ---: |
| Intercept | -0.437405854538573 | -0.273182897739321 |
| Age coefficient | 0.00656596439888336 | 0.0041533502142907 |
| Clustered age SE | 0.00537072111660872 | 0.00339527154508367 |
| Continuous age AME | 0.00156423141047196 | 0.00159522357872927 |
| Conditional-X clustered AME SE | 0.00128419688482865 | 0.00130761824944336 |

Native tests protect singleton-cluster equivalence to unclustered robust fits,
cluster relabeling, missing IDs before binary-outcome validation, exact wide
Decimal IDs, retained margins after edits, data/labels and failed-refit clearing.
Cluster IDs may be numeric, text or boolean; NULL/nonfinite IDs are excluded.
Wide IDs are not rounded through f64, and floating -0/+0 identify the same group.
See `help logit`, `help probit` and the [structured contract](../../docs/reference/command-results.md).
