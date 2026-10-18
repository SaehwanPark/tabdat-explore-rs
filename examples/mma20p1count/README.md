# Native Poisson ML / robust QMLE: RAND doctor visits

This is a bounded Cameron and Trivedi, *Microeconometrics: Methods and
Applications* (2005), §20.3, pp. 671–674 replication: the **Poisson coefficients
and model-based/independent-row robust statistic columns of Table 20.5 (p. 673)**.
The book/author display calls these t-ratios; native inference is asymptotic
**normal z**, not Student t. It is not the complete table, count illustration,
chapter, or Poisson/NB graduate-method requirement.

`poisson.td` loads the prepared source, validates export-position uniqueness,
summarizes counts, fits both covariance modes, reports diagnostics and continuous
partial effects, predicts conditional means/indices, and exports every input and
prediction in order. Terminal, JSON, scripts and piped sessions use the same fit.
No production R/Python or new native dependency is required.

## Exact source, sample and design

Author files (November 2006 revision, which also adds Table 20.6):

- [mma20p1count.do](https://cameron.econ.ucdavis.edu/mmabook/mma20p1count.do)
- [mma20p1count.txt](https://cameron.econ.ucdavis.edu/mmabook/mma20p1count.txt)
- [mma20p1count.asc](https://cameron.econ.ucdavis.edu/mmabook/mma20p1count.asc)
- optional [randdata.dta](https://cameron.econ.ucdavis.edu/mmabook/randdata.dta)

The executable uses **all five years**, despite an earlier stale year-2-only
comment. Each observation is a person-year; the author explicitly warns that
clustering is needed and fits additional clustered models not reported in the
book's table. **Neither covariance mode below is panel-robust.** Do not interpret
this reproduction of the independent-row columns as a dependable panel analysis.

The original DTA has 20,190 rows. Removing missing `educdec` at original 1-based
row positions 9746, 10153, 10400 and 10859 gives the author's **20,186 × 18**
complete prepared ASCII. No additional exclusions, selection, weights, exposure
or offset are used. The token order is:

```text
MDU LC IDP LPI FMDE PHYSLIM NDISEASE HLTHG HLTHF HLTHP LINC LFAM EDUCDEC AGE FEMALE CHILD FEMCHILD BLACK
```

`MDU` is the outcome; the other 17 numeric columns and an intercept form the
design, in that order. Native parameter order is intercept first. Source tokens
span physical lines (7/7/4 fields); `count_to_csv.awk` assembles 18-field records,
preserves decimal tokens and assigns **prepared_row**, an export position, **not
a person ID, year, or original DTA row ID**. No author data/code is redistributed.

The sample has count total **57,746**, **6,308 zeros**, maximum **77**, mean
**2.8606955315566713** and sample variance **20.29290442579822**. These raw moments
motivate checking the fitted variance assumption; they do not themselves test
conditional overdispersion.

The optional `audit_source.R` verifies every prepared count and predictor against
DTA order after the education exclusion. Counts match exactly; predictors differ
by less than 5e-6 from the author's ASCII export precision. This is **input
quantization**, not an estimator tolerance. The ASCII and original-DTA R log
likelihoods are respectively -60087.622048947298 and -60087.622072360973; maximum
coefficient difference is 2.7274857983261569e-8. Native validation uses the **exact
ASCII design**, not a widened comparison against different DTA inputs.

## Reproduce in a fresh directory/session

From the repository root, with the pinned Rust toolchain:

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo build --locked --bin tabdat-explore-rs
repo="$PWD"
work="$(mktemp -d)"
cd "$work"
for file in mma20p1count.do mma20p1count.txt mma20p1count.asc; do
  curl -fsSL "https://cameron.econ.ucdavis.edu/mmabook/$file" -o "$file"
done
shasum -a 256 -c <<'SHA256'
e433604d496293bc58119f7984ca009266870adadeb0eaf580e48b3fb5b700c7  mma20p1count.do
3694a7d94359a23d51c7acb9f4ebfad3443974bb8c4cb575d0a73c764f72658c  mma20p1count.txt
c8b1eee8ae1f637f4d4e4c6aa6b2dcbf2b8aa98f276b9369adcb4f02a528c58a  mma20p1count.asc
SHA256
awk -f "$repo/examples/mma20p1count/count_to_csv.awk" mma20p1count.asc > sample.tmp &&
  mv sample.tmp mma20p1count.csv
"$repo/target/debug/tabdat-explore-rs" --json \
  -f "$repo/examples/mma20p1count/poisson.td" > native.jsonl
```

Run `--text -f ...` or pipe `run ".../poisson.td"` into a new no-argument session
in a **different fresh directory containing the same input CSV** to inspect
readable tables. JSON contains `PoissonResult`, `PoissonMarginalEffectsResult`
and ordinary prediction/export metadata. Full predictions are in
`poisson_predictions.csv`, with all 19 input fields followed by classical mean,
classical xb, robust mean and robust xb. Means are **not probabilities** or
integer draws; do not clip at one. The two covariance modes have identical point
estimates/predictions, not identical inference.

Outputs are no-replace. An existing file/directory/symlink at the export path is
preserved and stops the script at its originating line. Source bytes are never
changed. The script is **not transactional**: completed fits/column additions
precede a later export failure. Recover deliberately in a fresh output directory
and session, not by silently overwriting or automatically deleting old results.

## Independent development validation

With development-only R **4.6.1** and Python 3 (no third-party Python packages):

```sh
Rscript "$repo/crates/tabdat-stats/tests/fixtures/poisson_reference.R" \
  mma20p1count.asc R_predictions.csv > R_reference.csv
python3 "$repo/examples/mma20p1count/check_poisson_reference.py" \
  R_reference.csv R_predictions.csv native.jsonl poisson_predictions.csv
```

Base R `glm` followed by a score-fixed-point update reevaluates Poisson
information. The reference independently computes the full covariance, normal
inference, LR/Wald, diagnostics, predictions and conditional-design delta effects.
It checks **1,095 numerical quantities** across both covariance modes and every
field/identity/mean/xb of all **20,186** records. Metadata checks require the exact
sample positions, parameter order, covariance correction and reference law.
The R generator without arguments also regenerates the **469-quantity synthetic
fixture** covering multiple predictors, no constant, intercept only, positive
constant responses and an all-zero no-constant interior optimum.

Tolerances: coefficient/covariance/SE/z/CI/effects/predictions and ordinary
statistics **1e-10 absolute**; accumulated likelihood/LR/deviance/Pearson totals
**1e-7 absolute** over 20k rows (bounded well below 1e-11 relative at this scale);
input design **1e-12 absolute**; tails **1e-10 relative + f64 minimum positive
normal**. No tolerance is widened for the DTA/ASCII difference. True float64
underflow may be zero; representable small tails may not be clipped to zero.

For the optional original-source audit, install development R `foreign`:

```sh
curl -fsSL https://cameron.econ.ucdavis.edu/mmabook/randdata.dta -o randdata.dta
printf '%s\n' '8364cf9e53e32c794bcf0313a1590adcedf4e0526ba792884c7978cde169a628  randdata.dta' | shasum -a 256 -c
Rscript "$repo/examples/mma20p1count/audit_source.R" mma20p1count.asc randdata.dta
```

## Statistical and state contract

- Exact nonnegative integer **loaded values** ≤2^53−1, joint finite complete
  sample, N>K and full-rank design. DECIMAL/wide integers are checked before
  conversion; this does not recover decimal digits already rounded by a loader
  into DOUBLE. Invalid finite counts on complete rows fail; a missing/nonfinite
  model cell excludes the row. No weights or cluster groups.
- Unit-column-norm Newton/Fisher QR, deterministic bounded backtracking, at most
  100 information evaluations. Both absolute scaled score and **full** Newton
  step must be ≤1e-10. Numerical/domain admission does not guarantee a fit: strict
  convergence/rank/finite-representability bounds can reject extreme finite
  inputs. This is not an analytic existence/boundary classifier.
- Model-based inverse Poisson information has no N−K scaling. `robust` is the
  independent-row score sandwich **N/(N−1)**, not OLS HC1, Pearson-dispersion
  scaling or panel/cluster covariance. Coefficients have fixed normal 95%
  intervals. Zero robust variance retains estimates but z/p/CI are unavailable,
  not exact significance/intervals.
- Intercept null uses the estimated overall mean; `noconstant` null fixes all
  coefficients at zero, μ=1, not a fitted intercept. Robust LR p/reference are
  unavailable; likelihood/McFadden remain descriptive and selected-V zero-slope
  Wald is asymptotic chi-square, with explicit rank/unavailability reasons.
  Intercept-only models do not invent a slope test.
- Residual deviance and Pearson χ²/(N−K) are descriptive diagnostics, not panel
  or general specification tests. They do not rescale the selected covariance.
- `predict newvar, n` uses current predictors and exp(xb); `xb` and **bare predict**
  return the index. Outcome values are irrelevant to prediction. Missing/nonfinite
  current indices and overflowing means yield NULL; finite negative-index
  exponential underflow is represented by zero. Publication is atomic, preserves
  order/labels/fit and never replaces an existing column.
- `estat margins` uses the owned retained design: continuous column derivative
  βj*mean(μ), with full selected-covariance delta gradient
  βj*mean(μX)+ej*mean(μ). It is conditional on observed X, not discrete changes,
  original-variable interaction/log chain rules or random-X uncertainty. These
  effect/diagnostic reports are native validated extensions, not extra claimed
  book columns. Current edits do not change the retained effects. A failed new
  runtime fit clears stale inference without changing data/labels/order.

Still required: NB with explicit parameterization; offsets/exposure, clustered
Poisson (original IDs must be recovered from DTA), survey/weights, count-PMF/CDF
prediction UI and Table 20.6, factors/discrete changes, general count test/lincom,
reusable named Poisson reports and remaining Tables 20.3–20.6/workflows. Existing
`estimates store` intentionally remains OLS/logit/probit-only. No restoration or
persistence of count fits is claimed.
