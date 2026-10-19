# Native Poisson ML / robust QMLE and NB2: RAND doctor visits

This is a bounded Cameron and Trivedi, *Microeconometrics: Methods and
Applications* (2005), §20.3, pp. 671–674 replication: the **Poisson coefficients
and model-based/independent-row robust statistic columns of Table 20.5 (p. 673)**.
The book/author display calls these t-ratios; native inference is asymptotic
**normal z**, not Student t. It is not the complete table, count illustration,
chapter, or Poisson/NB graduate-method requirement. The separate
[NB2 extension below](#nb2-joint-mean-and-dispersion-workflow) validates the NB
coefficient/model-based/independent-row robust statistic portion of the same table.
The [original-person Poisson extension below](#poisson-with-original-person-clustered-inference)
adds independently validated clustered covariance, not a printed Table20.5 column.

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

For the optional original-source audit, install development R `foreign` and `MASS`:

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
  100 outer Newton iterations, up to50 backtracking candidates per iteration.
  The reported count excludes candidates. Both absolute scaled score and **full** Newton
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

Still required: offsets/exposure, NB2 clustering, household/site/multiway/survey
inference, weights, count-PMF/CDF UI and Table20.6, formal NB
boundary/specification tests, factors/discrete changes, general count test/lincom,
reusable named count reports and remaining Tables20.3–20.6/workflows. Existing
`estimates store` intentionally remains OLS/logit/probit-only. No restoration or
persistence of count fits is claimed.

## NB2: joint mean and dispersion workflow

`nb2.td` uses the **same 20,186 prepared records, source/order and 17-predictor
plus intercept design**, not a different cohort. It fits NB2 under both covariance
modes, reports dispersion/diagnostics/continuous effects, predicts means and
indices, and exports `nb2_predictions.csv`. This is the NB coefficient and
model-based/independent-row robust statistic portion of Table20.5, not the whole
illustration, Table20.6 or a dependable panel analysis.

NB2 defines mu=exp(X beta), Var[y|X]=mu+alpha*mu², alpha=exp(lnalpha)>0,
size=1/alpha. **Joint beta/lnalpha observed-information inverse** is the
model-based covariance. Independent-row robust uses that full bread and joint
scores with **N/(N-1)**, not HC1. This matters: MASS::glm.nb's conditional/Fisher
LC SE is0.0127027993483, versus joint observed0.0128694197000. The robust joint
LC SE0.0156237866368 and lnalpha SE0.0187562319316 match the rounded author values.
No fixed-alpha substitution is made for effects: their zero direct lnalpha
partial derivative still uses the beta block of FULL joint V.

### Reproduce and independently check NB2

After the checksum/conversion steps above, use a fresh session/output directory
containing the same `mma20p1count.csv` and checksum-verified `mma20p1count.asc`
(`repo` is the repository path):

```sh
"$repo/target/debug/tabdat-explore-rs" --json \
  -f "$repo/examples/mma20p1count/nb2.td" > nb2_native.jsonl
Rscript "$repo/crates/tabdat-stats/tests/fixtures/nb2_reference.R" \
  mma20p1count.asc R_nb2_predictions.csv > R_nb2_reference.csv
python3 "$repo/examples/mma20p1count/check_nb2_reference.py" \
  R_nb2_reference.csv R_nb2_predictions.csv nb2_native.jsonl nb2_predictions.csv
```

The R reference uses R4.6.1/**MASS7.3.65** for an independent ML start, then
joint score-fixed-point polishing with `dnbinom`, `digamma` and `trigamma` and
numerical score-differentiation checks of the observed Hessian. It validates
**1,187** full-covariance/beta/normal-CI/tail/dispersion/null/LR/Wald/diagnostic/
conditional-X effect quantities and every field/mean/index of **all20,186** rows.
Without arguments it regenerates742 synthetic quantities across multiple,
no-constant and intercept-only designs. The tolerances above remain unchanged,
including1e-10 ordinary/Wald and1e-7 accumulated totals; no relaxation for an
ill-conditioned quadratic or different DTA input is used.

The separate `nb2_scalar_reference.R`/CSV stresses20 count/dispersion points up
to y100000 and |lnalpha|11:120 PMF/score/observed-curvature checks. Scalar PMF
uses1e-10 absolute; derivative stress comparisons allow1e-8 absolute+1e-12
relative because the independent R digamma/trigamma differences amplify rounding.
This is not a relaxation of the full fitted-inference tolerance.

Native LL is-42777.61100954579, alpha1.1822516830333165, lnalpha0.16742082612579268;
restricted NB null alpha1.4709552441411557 is separately estimated. Source audit
finds original-DTA LL-42777.611007044703, maximum beta difference4.3703871e-8
and alpha difference1.93e-10 from ASCII quantization. These are different inputs,
not fitting tolerances. Historical Stata stopping/rounded values are also not the
converged float64 score fixed point (e.g. HLTHP approximately0.4256563 versus
0.42565748); the published four-decimal coefficient display agrees. Do not force
historical lagged/stopped estimates into the independently converged result.

### NB2 numerical, inference and state limits

- Exact loaded counts **0..100000**, to bound the integer gamma-ratio/derivative
  cache; the converter does not silently discard/clip larger counts. N>K+1,
  full-rank design, distinct names including reserved `lnalpha`. Missing/nonfinite
  model cells are excluded before rejecting invalid finite counts, as for Poisson.
- lnalpha strictly**(-12,12)**; unit-norm mean columns and lnalpha*sqrt(N)
  coordinates. At most100 outer Newton iterations per full/restricted fit,
  each with at most20 damping attempts/50 backtracking trials per attempt.
  Reported iterations count full-model outer steps, not candidate evaluations
  or restricted-fit work. Final UNDAMPED joint
  information must be positive definite and both absolute scaled score/full step
  <=1e-10. Poisson-limit/boundary/nonrepresentable fits fail, not clip alpha or
  silently fall back to Poisson. These are numerical limits, not an existence test.
- Beta has normal95 inference. lnalpha has joint SE/normal CI; alpha SE is delta
  and its CI exponentiates log-CI endpoints. **No normal alpha=0 or formal boundary
  LR test** is supplied. Zero variance does not imply exact significance/intervals.
- Model-based slope LR compares a restricted NB null with dispersion reestimated:
  intercept-only mean, or fixed mu1 without intercept. Robust LR p/reference are
  unavailable. Selected-V robust Wald excludes lnalpha; it tests beta slopes only.
- NB deviance at fitted alpha and Pearson/(N-K-1) are descriptive, not formal
  specification/panel tests or covariance multipliers. Their improvement over
  Poisson does not establish an adequate conditional distribution/dependence model.
- `predict ..., n`/xb and bare-xb use current predictors only, atomic publication,
  null overflow/nonfinite and finite negative exponential underflow0. Retained-X
  effects are continuous beta_j*mean(mu), full-joint-V conditional-X delta normal95,
  not factor/discrete changes, chain rules or random-X uncertainty.
- Owned `negative_binomial_fit` state preserves old `negative_binomial` placeholder
  reads without promising restoration. Failed new fits clear stale inference,
  not data/labels/order. Unsupported post-estimation/prediction and collisions
  preserve state. JSON adds `NbregResult`/`NbregMarginalEffectsResult`; other
  schemas remain unchanged. Fresh JSON/text/piped runs publish identical CSVs.

No original IDs occur in the ASCII: independent-row robust NB2 is **not
panel-robust**, and cluster requests are explicitly rejected. Offsets/exposure,
NB2 original-ID clustering, weights/survey, count probabilities/Table20.6, formal
alpha/specification tests, named count reports and other required count analyses
remain open. The Poisson extension below does not install NB2 clustering. Neither
addition completes the count family or book.

## Poisson with original-person clustered inference

`poisson_clustered.td` extends the **same Table20.5 Poisson conditional-mean
design** to one-way original-person covariance. This is a native analysis extension,
not a person-cluster column printed in the book/author log. It does not fit FE/RE,
resolve RAND household/site/randomization dependence, correct a misspecified mean
or establish causal insurance effects. Independence ACROSS chosen people is required.

The ASCII has no IDs. `prepare_person.R` independently checks all20,186 ASCII
outcomes/predictor rows against the original20,190-row DTA, excluding only the four
education-missing original positions listed above. Counts/order agree exactly;
<5e-6 predictor export differences are source quantization, not model tolerances.
Only AFTER that audit it adds `original_row`, actual `zper` as `person_id`, and
actual `year` as `study_year`; `prepared_row` remains a prepared export position,
NEVER a substitute person ID. Person IDs are finite exact integers; person-year
keys are unique. All five years have5638/5574/5545/1715/1714 records. G=5908,
with1/2/3/4/5 records per person for265/246/3743/70/1584 people respectively.

In a fresh output directory containing the checksum-verified ASC/DTA above:

```sh
Rscript "$repo/examples/mma20p1count/prepare_person.R" \
  mma20p1count.asc randdata.dta mma20p1count_person.csv
"$repo/target/debug/tabdat-explore-rs" --json \
  -f "$repo/examples/mma20p1count/poisson_clustered.td" > person_native.jsonl
Rscript "$repo/crates/tabdat-stats/tests/fixtures/poisson_cluster_reference.R" \
  mma20p1count.asc randdata.dta R_person_predictions.csv > R_person_reference.csv
python3 "$repo/examples/mma20p1count/check_poisson_cluster_reference.py" \
  R_person_reference.csv R_person_predictions.csv person_native.jsonl poisson_person_predictions.csv
```

Development R uses `foreign0.8.91`/`sandwich3.1.1` under R4.6.1. Preparation
stages a complete CSV and hard-links without replacing files/directories/symlinks;
failed audits leave no output. The independent reference reads ASC/DTA itself,
not the converted CSV. **551 quantities and every original/prepared/person/year/
source/design/mean/index field of all20,186 rows** match;467 synthetic multiple,
no-constant, intercept-only and singleton quantities are also validated. References
regenerate byte-identically; prior ordinary/total/design/relative-tail tolerances
are unchanged. These R programs are development data/reference tools, never
invoked by TabDat; ordinary user CSV cluster fits need no R. Native DTA access
remains an unimplemented required foundation, not a newly installed backend.

The reference warm-starts `glm` at its fixed point before `vcovCL(type='HC0',
cadjust=TRUE)`: ordinary likelihood convergence can leave stored QR/working
weights at the previous IRLS point (initial covariance difference2.95e-8).
Synchronized weights differ from means by1.95e-12; trusted vcovCL and an explicit
converged-information/group-score covariance agree within1.12e-14. No native
tolerance was widened to match lagged reference weights.

Covariance is B[sum_g S_g S_g']B*G/(G-1), S_g=sum_i_in_g X_i*(y_i-mu_i),
with converged Poisson inverse information B. Sum BEFORE outer products; no
row-score substitution, HC1, Pearson scaling or centering. Inference is normal95,
not G-1 t; LR p/reference unavailable; beta-slope Wald guards q>=G with no reduced
q/pseudoinverse. LC SE is0.022682392509641616,p0.059567525116940544; Wald(17)
827.0703031869317,p7.73648862864e-165. Point coefficients/means and descriptive
diagnostics remain those of the same Poisson mean model. Few clusters can still
give unreliable inference despite G/(G-1).

`cluster(var)` accepts numeric/text/boolean keys, excludes SQL NULL/nonfinite
numeric keys jointly BEFORE rejecting finite invalid counts, and preserves wide
integer/DECIMAL identities before f64 rounding. Loaded empty text is a group;
CSV loader NULL conventions still apply. G>=2/N>K/full rank and existing strict
Poisson numerical bounds remain. Failed new runtime fits clear only stale
inference; bad prediction/options preserve state. Retained-X effects carry
original groups/selected covariance despite edits/drop ID, while current n/xb
need only mean predictors, not outcomes/IDs. JSON adds optional cluster metadata
ONLY for clustered results; old noncluster shapes/owned-state reads are preserved.
JSON/text/piped native runs publish identical ordered CSVs without replacing targets.

NB2 clustering, household/site/multiway/survey/few-cluster methods, offsets/exposure,
count probabilities/Table20.6, formal tests, general restrictions/named count
reports and remaining required count/book analyses remain open. This is not
whole count-family/panel/chapter/book/goal completion. No author data/program
is redistributed.
