# Fishing-mode multinomial logit: a bounded Chapter15 workflow

This reproduces the **MNL column of Table15.2, p.493**, §15.2.1–3 of Cameron
and Trivedi (2005), under float64/converged-information conventions. It adds
analytic continuous income AMEs with joint-covariance delta inference. It does
**not** reproduce the whole table/chapter or the historical p494 finite-difference
and at-means calculations.

The March2005 [author program](https://cameron.econ.ucdavis.edu/mmabook/mma15p1mnl.do)
labels this executable block Table15.2; its overview's Table15.1 is a typo.
The [author output](https://cameron.econ.ucdavis.edu/mmabook/mma15p1mnl.txt)
provides the historical comparison. Data originate in Herriges and Kling (1999),
*Review of Economics and Statistics*81:62–72. Author data/code are downloaded
locally, not redistributed here.

| Asset | SHA256 |
| --- | --- |
| `Nldata.asc` | `1fb6f7d5abaf790d4df2c6a13a499ca42c0db4f950d941ba92565c30b4ab6c22` |
| `mma15p1mnl.do` | `6c68bc3c8b969d6ee77781251044e9f5a659ea01b295fb92c1ea4cfd7f144579` |
| `mma15p1mnl.txt` | `f1a9f8edaf0cac9359cfedfd262bc640a79132abfb5194c4b799dc0c8e628247` |

## Sample/design/inference

- All1,182 original people, original1-based `source_row` identity/order; no
  stochastic preparation, filters or additional missing/nonfinite exclusions.
- Numeric `mode`: beach1 (134), pier2 (178), private boat3 (418), charter4 (452).
- Intercept and `ydiv1000=income/1000`, beach1 reference. This is a person-level
  shared design, **not** conditional logit on alternative-specific price/catch.
- Unweighted softmax maximum likelihood, full joint inverse Fisher covariance;
  no N−K correction, asymptotic normal95 coefficient/effect inference. Parameter
  order is category2/3/4, each intercept then income, unlike the author display's
  income-then-intercept order. Names/category IDs identify parameters explicitly.
- Null likelihood estimates the four shares; omnibus zero-income-slope LR has
  three degrees of freedom and an asymptotic chi-square reference. McFadden R²
  compares these likelihoods; it is not a linear-model variance explanation.
- Coefficients/effects are conditional model associations, not identified causal
  income effects. Softmax entails the usual IIA substitution structure; this
  workflow does not test that assumption.
- Category probabilities and base-relative indices use current finite predictor
  columns, ignoring current outcomes and original sample membership. The base
  index is zero. Scalar predictions require `outcome(code)`.
- `estat margins` uses retained estimation X for all four income derivatives,
  with analytic gradients and the **full joint** covariance. Uncertainty is
  conditional on observed X, not resampled-X uncertainty. These are continuous
  column derivatives, not at-means effects, discrete changes or chain rules.

## Run and independently audit

No R or Python is needed for the native analysis. The optional audit uses R4.6.1
with `nnet`7.3.20 and standard-library Python. The converter consumes **16-field
token-stream records**, not physical lines, and only publishes its CSV on success.
Start in the repository, use a fresh working directory, and keep that path:

```sh
(
set -eu
cargo build --locked --bin tabdat-explore-rs
repo="$PWD"
work="$(mktemp -d)"
printf 'Work directory: %s\n' "$work"
cd "$work"
curl -fsSLO https://cameron.econ.ucdavis.edu/mmabook/Nldata.asc
printf '%s  Nldata.asc\n' \
  1fb6f7d5abaf790d4df2c6a13a499ca42c0db4f950d941ba92565c30b4ab6c22 | shasum -a 256 -c -
awk -f "$repo/examples/mma14p1binary/nldata_to_csv.awk" Nldata.asc > Nldata.csv.tmp
mv Nldata.csv.tmp Nldata.csv
"$repo/target/debug/tabdat-explore-rs" --json \
  -c "run \"$repo/examples/mma15p1mnl/mlogit.td\"" > mlogit.jsonl
Rscript "$repo/examples/mma15p1mnl/reference_mlogit.R" \
  Nldata.csv R_predictions.csv > R_quantities.csv
python3 "$repo/examples/mma15p1mnl/check_mlogit.py" "$work"
)
```

For human-readable tables, run the same script with `--text` in a fresh output
directory; an existing export is not overwritten. `mlogit.td` performs load →
identity check → preparation → fit → effects → eight scalar predictions → CSV
export. JSON contains the joint covariance and all coefficient/effect uncertainty.
Named `estimates store/table`, fit restoration and general `lincom/test` are not
supported for this family in this slice.

## Evidence and interpretation limits

`check_mlogit.py` checks all107 independent-R scalar quantities (coefficients,
full6×6 covariance, normal inference, likelihood/null/LR and four AMEs with delta
inference) at **1e-9 absolute**. It checks all21,276 source/design cells at1e-10,
all4,728 probabilities and4,728 indices at2e-10, all18,912 original numeric
source tokens by exact parsed-f64 equality and the prepared income column by
exact division, exact source identities/order,
category codes/counts, sample and covariance/reference metadata, sum-to-one
probabilities, zero base indices and sum-to-zero category AMEs.

Observed local maxima against R were coefficient7.2e-15, covariance4.2e-15,
likelihood3.9e-12, AME3.5e-16, delta-SE7.7e-14, AME z6.1e-11,
probability1.1e-15 and index1.1e-14. The native fit needed six information
evaluations, with averaged RMS-scaled score2.03e-16 and full Newton step6.17e-15.
R/nnet likelihood optimization is polished to a score fixed point using nnet's
independent alternative-summed Fisher implementation; AME covariance uses
independently Richardson-differenced gradients, not native results.

Additional committed R/nnet fixtures check376 scalar quantities for exact
three-class cell odds and four-class/two-predictor designs with/without an
intercept. Tests also check nonlowest-base full-covariance reparameterization,
binary-logit reduction, missingness, exact DECIMAL categories before rounding,
retained-versus-current designs, and failure-state/data invariants. The high-z
no-intercept numerical derivative reference uses sixth-order Richardson with a
larger step to reduce cancellation; native comparison tolerance remains1e-9.

The historical check is deliberately separate: author coefficients/SEs agree
within1e-6 and likelihood within the printed four-decimal rounding, **not every
printed SE digit**. For example, native pier-intercept SE is0.228631953891812,
versus author0.2286316. Float-storage, optimizer termination and information
conventions differ; their individual contributions have not been isolated.
This does not replace the much tighter independent float64 R comparison.

Native analytic income AMEs are:

| Category | AME per additional $1,000 income |
| --- | ---: |
| beach1 | 0.000164667916071766 |
| pier2 | −0.0207689721452650 |
| private3 | 0.0317561578582475 |
| charter4 | −0.0111518536290541 |

The author program averages finite differences after a0.0001 shift using Stata8
float-generated columns/probabilities; for example its beach average is about
0.0001549, **not** the analytic value above. Its `mfx` also evaluates an effect
at mean income rather than an AME. Neither those computations nor their reported
uncertainty are claimed reproduced. No tolerance is expanded to equate them.

Conditional, combined and nested choice models, alternative-specific regressors,
robust/cluster/survey covariance, factors/discrete changes, IIA tests, remaining
Table15.1/15.2 columns and later Chapter15 illustrations remain required work.
Current bounds/failure/recovery conventions are in `help mlogit`; passing these
local checks is not certification of every MNL problem or platform.
