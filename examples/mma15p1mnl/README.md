# Fishing-mode multinomial and conditional logit: bounded Chapter15 workflows

The first workflow reproduces the **MNL column of Table15.2, p.493**, §15.2.1–3 of Cameron
and Trivedi (2005), under float64/converged-information conventions. It adds
analytic continuous income AMEs with joint-covariance delta inference. It does
**not** reproduce the whole table/chapter or the historical p494 finite-difference
and at-means calculations. The separate [conditional-choice workflow below](#conditional-logit-table152-cl-column-and-section2b-point-contrasts)
adds the price/catch CL column and explicitly labeled finite point contrasts.

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

Combined/nested choice models, robust/cluster/survey covariance, automatic factors,
IIA tests, remaining Table15.1/15.2 columns and later Chapter15 illustrations remain
required work. Current MNL bounds/failure/recovery conventions are in `help mlogit`;
passing these local checks is not certification of every MNL problem or platform.

## Conditional logit: Table15.2 CL column and section2B point contrasts

`clogit.td` targets the same author program's section2/Table15.2 CL column: all
1,182 people × four alternatives =4,728 long rows, keyed by original person ID
and numeric alternative code. `chosen` is the original alternative indicator,
checked against original `mode`. Utility has common price/catch slopes `p`/`q`,
no common intercept, alternative constants or income interactions. Each person
selects exactly one alternative from the complete four-alternative set.

Native `clogit chosen p q, group(id) alternative(alternative)` uses single-choice
conditional maximum likelihood and the full joint inverse conditional Fisher
information, **one independent decision per person**, correction1 (no row N−K
or group G−K adjustment). Coefficient inference is asymptotic normal95. The null
sets both slopes to zero, yielding four equal probabilities; the LR reference
is chi-square(2), not a binary-row or common-intercept LR convention. Assumptions
include independent decisions, maintained utility specification and IIA; this
workflow supplies neither an IIA test nor identification of causal price effects.

In a fresh source/preparation directory created by the shell recipe above, run:

```sh
"$repo/target/debug/tabdat-explore-rs" --json \
  -c "run \"$repo/examples/mma15p1mnl/clogit.td\"" > clogit.jsonl
Rscript "$repo/examples/mma15p1mnl/reference_clogit.R" \
  Nldata.csv R_clogit_predictions.csv > R_clogit_quantities.csv
python3 "$repo/examples/mma15p1mnl/check_clogit.py" "$work"
```

Those variables exist inside the recipe's subshell: insert these commands before
its closing `)`, or explicitly restore `repo`/`work` and `cd "$work"` afterward.
Use `--text` instead of `--json` for a separate human-readable run in a fresh
output directory; exports refuse collisions. R4.6.1/survival3.8.6 and Python are
optional development-audit dependencies only, not needed by native analysis.

The script guards person/alternative identity, fits, predicts baseline
probabilities, shifts **one alternative at a time**, and reports all32 mean
substitution contrasts. Price+1 entries are `100*(Pnew−Pbase)` percentage points;
catch+.001 entries are `1000*(Pnew−Pbase)`, finite approximations per catch unit.
They are **point predictions**, not analytic AMEs, causal effects or delta-method
uncertainty. Both predictors are reset from saved baseline columns before every
scenario, and restored before export. The author's final catch restore at line273
adds .001 again after computing the last predictions; we deliberately restore
the original baseline instead. No extra post-scenario perturbation is retained.

`fishing_clogit_predictions.csv` contains original keyed design, saved baselines
and all nine probability scenarios. `fishing_clogit_contrasts.csv` contains four
rows of baseline mean probabilities and price/catch contrasts, suitable for
terminal inspection or downstream reporting. Full fit uncertainty is in the
terminal/JSON fit, not invented for those finite contrasts.

`check_clogit.py` binds **both** native/R keyed outcome/design exports directly to
the hashed original source tokens, verifies exact restored inputs and validates
all61 scalar quantities at1e-9 absolute (coefficients, full2×2 covariance,
normal inference, likelihood/null/LR/McFadden and36 mean/contrast values). Tiny
nonzero p-values additionally require relative agreement at1e-9, so an underflowed
zero cannot pass an absolute-only check. All
42,552 probabilities are checked at1e-10, with per-set sum-to-one and per-scenario
sum-to-zero mean contrasts. R's exact single-event `survival::clogit` supplies
independent coefficients/covariance; its Cox expected counts also equal the
original-set normalized probabilities. Identically altered native/R design
exports are rejected by the original-token binding.

Observed local maxima: coefficient1.45e-15, covariance3.30e-17, likelihood6.83e-13,
price contrasts1.09e-15, catch contrasts7.47e-15 and probability1.45e-15. Seven
native information evaluations give averaged scaled score2.75e-14 and full
undamped step3.57e-13. The separate historical comparison matches printed
coefficients/SEs within1e-6 and likelihood within four-decimal rounding; it does
not substitute a loose historical tolerance for native/R agreement. Float64
finite contrasts are not claimed identical to Stata8 float-storage digits;
individual historical storage/optimizer differences have not been isolated.

Tests additionally cover exact2/3/4-alternative conditional MLE/covariance,
binary reduction, common-within-set shifts, row/group permutation, covariance
halving under independent-decision replication, whole-set missingness, exact
DECIMAL IDs/codes/outcomes before rounding, current-versus-retained provenance,
rank/separation/convergence failures and prediction/error/export invariants.

Keys, duplicates and identical complete choice sets are checked **before**
missingness. A missing/nonfinite outcome or predictor drops the whole identified
set; other finite outcome-domain errors are checked only on retained sets.
Current prediction ignores outcomes and admits new person IDs, but requires the
retained alternative universe. One missing predictor makes the whole current
set NULL; structural/numerical errors publish no column and retain the fit.
Failed new fits clear old inference while preserving data. See `help clogit`
for bounded resource guards and unsupported options.

This is not general matched case-control conditional logistic regression,
multiple-event choice, ragged availability, weighted/robust/cluster/survey
inference, automatic categorical design, nested models, named reports/general
restrictions, IIA testing or effects uncertainty. The author's combined fixed-
coefficient design can be expressed with explicit numeric columns but is not
source-validated here; that is not a random-coefficients mixed-logit claim.
`estat margins`
explicitly rejects this family. Remaining Chapter15 illustrations remain required;
neither these two columns nor passing local tests establish whole-table/chapter
completion or platform certification.
