# Fishing-mode choice models: bounded Chapter15 workflows

The first workflow reproduces the **MNL column of Table15.2, p.493**, §15.2.1–3 of Cameron
and Trivedi (2005), under float64/converged-information conventions. It adds
analytic continuous income AMEs with joint-covariance delta inference. It does
**not** reproduce the whole table/chapter or the historical p494 finite-difference
and at-means calculations. The separate [conditional-choice workflow below](#conditional-logit-table152-cl-column-and-section2b-point-contrasts)
adds the price/catch CL column and explicitly labeled finite point contrasts.
The [combined-utility workflow](#combined-fixed-coefficient-utility-table152-final-column)
adds the final coefficient column and a conditional encoding check of MNL.
The [explicitly non-normalized nested workflow](#non-normalized-nested-choice)
targets the later MMA §15.6.3 p511 illustration, not the preferred RUM model.

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

Preferred-normalized/richer nested choice models, robust/cluster/survey covariance,
automatic factors, IIA tests, remaining illustrated tables and later Chapter15 analyses remain
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
coefficient design is validated in the separate workflow below, not as a
random-coefficients mixed-logit claim.
`estat margins`
explicitly rejects this family. Remaining Chapter15 illustrations remain required;
neither these two columns nor passing local tests establish whole-table/chapter
completion or platform certification.

## Combined fixed-coefficient utility: Table15.2 final column

`combined.td` targets `mma15p1mnl.do` section4's **eight-column** final fit,
labelled the Mixed column of Table15.2 p.493. This is a fixed-coefficient
combination of alternative-specific price/catch and person-income interactions,
**not random-coefficients mixed logit**. The preceding five-column exploratory
fit is not substituted for it. All1,182 original people/four alternatives remain.

Utility is `p q dummy2 dummy3 dummy4 d2y d3y d4y`. Price/catch slopes are common;
nonbeach alternative constants and income/1000 slopes are explicit numeric
columns. Beach's alternative constant/income slope are normalized to zero, not
its price/catch utility. The conditional likelihood removes a common person
intercept. Covariance is the full8×8 inverse conditional Fisher sum over1,182
independent decisions, correction1 and asymptotic normal95. The zero-all-eight
null gives equal choice probabilities, LR chi-square(8), and McFadden R² relative
to that null. No weights, robust/cluster covariance or causal identification.

The script first fits ordinary person-level MNL and its **six-column conditional
encoding** from author section3. Parameter order changes from category-major
`(intercept,income)` to all three constants then all three income interactions;
the audit permutes the full covariance, not just standard errors. Full fitted
likelihood/probabilities and coefficients agree. **Nulls are not equivalent**:
ordinary MNL estimates category shares under zero income slopes (LR df3), while
the conditional encoding zeros all six utility columns (uniform null, LR df6).
Do not rank these models by pseudo R² with unlike nulls or equate their omnibus
LR tests merely because their full fits are identical. General nested-model
LR comparison/named fit tables are not introduced by this workflow.

Using the source/preparation directory from the earlier shell recipe, insert
these commands inside its subshell, or restore `repo`/`work` and `cd "$work"`:

```sh
"$repo/target/debug/tabdat-explore-rs" --json \
  -c "run \"$repo/examples/mma15p1mnl/combined.td\"" > combined.jsonl
Rscript "$repo/examples/mma15p1mnl/reference_combined.R" \
  Nldata.csv R_combined_predictions.csv > R_combined_quantities.csv
python3 "$repo/examples/mma15p1mnl/check_combined.py" "$work"
```

Only the optional audit needs R4.6.1/survival3.8.6 and standard-library Python.
For native tables use `--text` in a fresh output directory. The script guards
source identity and exact agreement of mode with all four choice flags **before
any fit**, guards long group/alternative pairs, fits the three models, predicts,
restores all baseline features, displays four contrast rows and exports
`fishing_combined_predictions.csv` / `fishing_combined_contrasts.csv`. No source
file is modified; an existing export is not overwritten.

Extra interpretation scenarios, **not additional claimed published effects**, are:

- `income1000`: mean probability change for income+$1,000, holding prices/catch
  fixed. All three income-interaction predictors change jointly. Changing only
  raw `ydiv1000` would leave predictions unchanged: that raw column is not one
  of the eight stored predictors. Every scenario resets the scalar and all
  three interactions from saved baselines.
- `charter_price1`: mean `100*(Pnew−Pbase)` percentage-point change when only
  charter price increases by1, other alternatives' prices/catch/income held fixed.
- `charter_catch001`: mean `1000*(Pnew−Pbase)` for charter catch+.001, a finite
  approximation per catch unit. It is not an analytic derivative or uncertainty
  interval. All scenarios are conditional model point associations, not causal
  effects or resampled-X inference; `estat margins` remains unsupported here.

`check_combined.py` binds both exports' exact IDs/order, flags versus mode,
price/catch, income division, all six indicator/interaction columns and restored
baselines to the hashed original tokens. It validates **218 R scalar quantities**
including all100 covariance entries across the six/eight-column fits at1e-9
absolute, and **28,368 probabilities** at1e-10. Positive p-values also require
relative1e-9 agreement; zero underflow cannot pass an absolute-only check. The
native MNL/full-covariance encoding permutation, distinct null metadata, sample,
normal reference, per-set sums and zero-sum mean contrasts are also checked.

Observed maxima: combined coefficient3.85e-14, covariance3.34e-15,
likelihood9.10e-13; income/price/catch contrasts5.87e-16/1.12e-15/1.03e-14.
The conditional MNL encoding gives coefficient3.34e-14, covariance5.60e-15 and
likelihood3.19e-12. Across all six probability columns the maximum is1.65e-14.
Historical combined coefficient/SE agreement is separately1e-6, likelihood
four-decimal rounding; individual Stata8 storage/optimizer effects remain
unisolated. No tolerance is widened to make those historical conventions equal.

Reference-export precision matters: R `write.csv` rounded computed income
values even with `options(digits=17)`, breaking exact source binding by one ULP.
The reference formats numeric exports with `sprintf("%.17g",x)` before writing;
regeneration then round-trips every design value exactly. This fixes audit
serialization, not statistical estimates or comparison tolerances. Preserve
that formatting when extending the reference and rerun exact design binding.

The inspectable CLI fixture uses138 artificial decisions at three income values,
four alternatives and exact positive cell counts. Its full-rank eight-column
MLE has known log-odds coefficients; baseline and all three joint/design-aware
scenarios are checked against closed form, including restored inputs and mean
contrasts. Duplicate source IDs or inconsistent choice flags fail before fitting/
export without changing inputs; export collisions retain earlier files.

All three principal Table15.2 coefficient columns now have source-audited
workflows with explicit inference conventions. This is not a single formatted
whole-table report, every exploratory fit in the author program, historical
float-digit parity or completed Chapter15 coverage. Preferred-normalized/other
illustrated choice analyses, IIA diagnostics, broader availability, effects
uncertainty, weights/robust/cluster/survey and automatic design/reporting remain
required. The distinct earlier non-normalized illustration follows.

## Non-normalized nested choice

`nested.td` targets **MMA §15.6.3 p511**, author `mma15p2gev.do` sections2A–C,
using all1,182 original people/four alternatives. Shore contains beach1/pier2;
boat contains private3/charter4. This deliberately fits the earlier **non-normalized**
parameterization, not the preferred common-utility RUM normalization. Source assets:

| Asset | SHA256 |
| --- | --- |
| `mma15p2gev.do` | `0dff4b93103da9d075967c1517f04fbe44a1b08543a34421e44f0cd717e50e27` |
| `mma15p2gev.txt` | `b1c2d7bf4008fae64ed659d046debd846ff59c6aa2115ea518ce8f0dc0037726` |

The native command is explicit:

```text
nlogit chosen p q, group(id) alternative(alternative) nest(nest) upper(dshore dshorey) nonnormalized
predict baseline, pr
```

Conditional lower probabilities normalize `exp(p*beta_p+q*beta_q)` inside each
nest. Upper utility is `dshore*gamma_a+dshorey*gamma_y` plus the nest-specific
positive scale times the lower log-sum-exp. Upper predictors must be constant
inside each person/nest. **Raw lower utility levels are retained**: case-common
lower shifts do not cancel when nest scales differ. There is no automatic
intercept, factor or income-interaction update.

Inference uses the full joint6×6 **observed** information inverse across1,182
independent decisions, correction1 and asymptotic normal95. Slope tests use zero;
IV diagnostics test **one**, not the excluded zero boundary. Natural-scale Wald
intervals may cross zero. The λ=1 conditional restriction re-estimates four
slopes, with a regular LR chi-square2 only after local scale identification is
checked there. A uniform zero-slope null has unidentified scales; no corresponding
omnibus chi-square or pseudo R² is fabricated. Optimization uses two deterministic
positive-scale starts, not a global-maximum proof. See `help nlogit` for bounds,
whole-set missingness/structural validation and atomic current-prediction rules.

In the earlier source/preparation recipe's subshell (or after restoring `repo`,
`work` and `cd "$work"`), with **fresh nested output paths**:

```sh
"$repo/target/debug/tabdat-explore-rs" --json \
  -c "run \"$repo/examples/mma15p1mnl/nested.td\"" > nested.jsonl
Rscript "$repo/examples/mma15p1mnl/reference_nested.R" \
  Nldata.csv R_nested_predictions.csv > R_nested_quantities.csv
python3 "$repo/examples/mma15p1mnl/check_nested.py" "$work"
```

Native analysis needs neither R nor Python. The optional reference was tested
with R4.6.1, `mlogit`2.0.0 and `survival`3.8.6; use an isolated development library
via `R_LIBS_USER` if desired. `mlogit`'s `unscaled=TRUE` places all features inside
conditional utility: multiplying its shore-constant/income coefficients by the
shore IV gives the reported top-level coefficients. A separate base-R `deriv3`
scalar likelihood independently checks fresh package scores/probabilities,
refines stationarity and supplies **observed** information, not default OPG.
The λ=1 `survival::clogit` point additionally passes the independent derivative/
information check; at this source point its refinement shift is zero.

The script guards source IDs/choice flags **before fitting**, fits the conditional
restriction and nested model, predicts both baselines, jointly raises shore-income
interactions for income+$1,000, and separately raises charter price by1 or catch
by.001. It restores all original design columns, displays four means/contrasts,
and exports `fishing_nested_predictions.csv` / `fishing_nested_contrasts.csv`.
Income changes are unscaled probability differences; price contrasts are100ΔP
percentage points; catch contrasts are1000ΔP. These are conditional point
associations, not causal effects, analytic AMEs or uncertainty intervals.
`estat margins`, general restrictions/named comparisons and robust/cluster/survey
inference are not implemented for this bounded model. For native text, rerun
with `--text` in a fresh output directory: exports refuse collisions.

The checker validates **149 independent quantities**, including all52 covariance
entries across the four/six-parameter fits, and **23,640 probabilities**. Both
exports' exact identities/order, choices, nest map, raw price/catch, income,
interactions and restored baselines bind directly to hashed original tokens.
Nested scalars use1e-9 absolute +1e-8 relative; restriction scalars retain1e-9
absolute. This new nested-workflow audit uses the predeclared1e-8 positive-tail
relative contract including its restriction; existing standalone CL/MNL gates
are unchanged. The restriction's1.73e-11 slope stopping difference amplifies to
1.82e-9 relative at p=4.42e-56; native tails agree with independent `math.erfc`
at native z, not a fabricated zero. Probabilities use1e-10 absolute. Native
four-row displays, per-set sums, zero-sum contrasts and exact restores are checked.
Identically corrupted native/R designs and a fake zero LR p-value are rejected.

Observed maxima: nested coefficient1.53e-10, covariance3.98e-8 on large weak-IV
variances, LL3.41e-12 and probability1.20e-11; income/price/catch contrasts
1.40e-15/1.12e-13/1.08e-13. The stationary fit has LL−1180.0125384828,
IVs32.1725778578/15.0747565917 and observed SEs29.9971412397/14.0098795481.
The λ=1 restriction has LL−1252.7110426869, LR145.3970084082, p≈2.67572e-32.
**Both IVs violate the descriptive (0,1] RUM bounds**; higher likelihood does
not establish random-utility consistency.

Historical Stata8 instead reports LL−1180.0156 and IVs29.98591/14.06438: this is
**not printed-digit parity**. A float32 preparation replay still gives stationary
IVs≈32.172579/15.074757, so storage alone does not explain the discrepancy.
No historical tolerance is widened to erase it. The preferred-normalized probe
on this restricted design diverged with singular shore information; that failed
probe is not a validated substitute or an exclusion of required work.

The actual-script CLI fixture has704 artificial decisions with six known natural
MLE parameters (slopes/upper coefficients ln2/ln3; IVs2/3), explicit full-rank
λ=1 identification, joint income/price/catch closed-form predictions, restores,
displayed means and input/export preservation. Bad IDs/flags fail before fitting.
Additional kernel/runtime tests cover full covariance, derivatives, permutation,
replication, exact DECIMAL keys/outcomes, whole-set exclusion, current new IDs/
absent outcomes, upper constancy/map errors and failure-state atomicity. These
bounded local checks do not complete Chapter15, preferred normalization, richer
trees/availability, effects uncertainty or platform certification.
