# NSW/PSID treatment comparison

This bounded workflow reproduces the first two rows of Cameron and Trivedi
(2005), Table 25.4, using the authors' [program](https://cameron.econ.ucdavis.edu/mmabook/mma25p1treatment.do),
[output](https://cameron.econ.ucdavis.edu/mmabook/mma25p1treatment.txt) and
[original data](https://cameron.econ.ucdavis.edu/mmabook/nswpsid.da1). It reports
pre-treatment group means, the 1978 earnings difference, a covariate-adjusted
linear comparison, robust uncertainty and individual fitted values.

The 2,675 people comprise 185 NSW treated participants and 2,490 PSID controls;
the control sample is observational, not the NSW randomized control group.
The unadjusted coefficient is the difference in observed group means. The
adjusted coefficient is the constant treatment indicator coefficient in a
linear model with intercept, age, age squared, education, an indicator for less
than 12 years of education, Black and Hispanic indicators, and 1974 and 1975
real earnings. It is a model-dependent conditional comparison. A causal effect
interpretation additionally requires a well-defined treatment, appropriate
sampling, conditional exchangeability/no unmeasured confounding, positivity and
correct conditional mean specification. Group means show imbalance but do not
establish overlap or remove confounding. The output is not evidence that training
caused either estimate.

Both models use the complete, unweighted, ordered sample and HC1 independent-row
covariance `N/(N-K)` with Student-t `N-K` inference. Their treatment coefficient
estimates are -15,204.776 (SE 655.914) and 217.944 (SE 767.881); rounding of
source ASCII and historical storage can differ. `lincom treat` reports 95%
intervals. The author notes that source U74/U75 are reversed; this bounded
analysis uses neither and does not silently reinterpret them. The source output
labels RE75/RE78 as 1974 in one comment; the variables are 1975 and 1978 earnings.

The executable guards dense ordered IDs, finite analysis fields, binary treatment
and indicator columns, both groups and full-rank estimability. Exports do not
clobber existing files; a failed script is not a transaction. Pinned source
SHA256: `810f6dbfbe0ffbcc52676946c330f49d4d8bf5cafc06980e912ca945ac7f4e44`.
The ASCII data are fetched separately and are not redistributed here.

From a fresh directory:

```sh
python3 /path/to/repo/examples/mma25p1treatment/prepare.py /path/to/nswpsid.da1 nswpsid.csv
/path/to/tabdat-explore-rs --json -f /path/to/repo/examples/mma25p1treatment/comparison.td > result.jsonl
Rscript /path/to/repo/examples/mma25p1treatment/reference.R /path/to/nswpsid.da1 reference
python3 /path/to/repo/examples/mma25p1treatment/check.py /path/to/nswpsid.da1 result.jsonl treatment_predictions.csv reference
```

The development-only base-R reference uses `lm` QR and independently assembled
HC1 covariance. The audit checks all source rows and predictions, both full
covariance matrices, coefficient inference and treatment intervals, group means,
and stored sample/report contents. Numerical tolerance is `1e-8 +
1e-9*abs(reference)` with an additional `1e-7` relative check for tiny nonzero
p-values. R and Python are not runtime dependencies. This slice does not include
the before/after, difference-in-differences, propensity-score matching or other
Chapter 25 analyses, nor a general treatment-effects command.

## Paired before/after and difference-in-differences

[`paired.td`](paired.td) completes the two time-comparison point estimates in
Table 25.4 using two records per person, ordered by source ordinal and 1975
then 1978. The 185-person before/after coefficient is the treated group's mean
change. The 2,675-person difference-in-differences coefficient is the treated
mean change minus the PSID-control mean change; it is the interaction in a
saturated group-by-period regression. The respective estimates are 4,817.090
and 2,326.505. The author program's historical row-level HC1 standard errors
assume independent person-period observations and do not allow within-person
correlation; these are **not** the uncertainty convention used here.

Both fits cluster by person. For N stacked observations, K coefficients and G
persons, the score-sandwich correction is `G/(G-1)*(N-1)/(N-K)` and coefficient
inference uses Student-t `G-1`. The before/after fit uses 370 observations,
185 clusters and SE 609.246; the difference-in-differences fit uses 5,350
observations, 2,675 clusters and SE 644.752. Clustering assumes independent
people and enough clusters; the sample is still observational. Before/after
alone mixes treatment with other changes over time. A causal interpretation of
the difference-in-differences requires, among other assumptions, no anticipation,
comparable groups and an appropriate untreated parallel-trends assumption; the
two observed periods and descriptive group means cannot establish these.

The script checks ordered, dense, unique person keys, finite earnings, binary
treatment, at least two people in each group and finite group means/predictions;
it exports all ordered fitted person-period rows without overwriting an existing
file. A failed script is not a transaction. From a fresh directory with the
same prepared `nswpsid.csv` described above:

```sh
/path/to/tabdat-explore-rs --json -f /path/to/repo/examples/mma25p1treatment/paired.td > paired_result.jsonl
Rscript /path/to/repo/examples/mma25p1treatment/paired_reference.R /path/to/nswpsid.da1 paired-reference
python3 /path/to/repo/examples/mma25p1treatment/check_paired.py /path/to/nswpsid.da1 paired_result.jsonl paired_predictions.csv paired-reference
```

Independent base-R QR and grouped-score CR1 calculations check both full
covariance matrices, coefficient inference, intervals, every retained person and
all 5,350 ordered exported predictions, at `1e-8 + 1e-9*abs(reference)` with a
stricter `1e-7` relative check on tiny nonzero p-values. The historical points
are reproduced to displayed precision; person-clustered SEs are a deliberately
different inferential extension. This is not a general panel or treatment-effects
command, propensity matching, overlap assessment or causal identification proof.

## Propensity-score overlap diagnostics

[`overlap.td`](overlap.td) fits the author program's §25.8.1–4 treatment
assignment logit on all 2,675 original people, then reports empirical common
score-range membership and pre-treatment means by treatment group and fixed
score interval. Its design includes age and education and their squares,
marital status, less-than-high-school indicator, Black and Hispanic indicators,
1974/1975 earnings and their squares, and the product of Black and the corrected
1974 zero-earnings indicator. The source U74/U75 columns are miscoded and are
not used. Age and education are divided by 10 and earnings by 10,000 before
squaring; these are invertible changes of scale in the same logit design, not
changes to fitted probabilities. The fit uses model-based logit covariance and
normal coefficient inference, not inference for any treatment effect.

The common range is the intersection of the observed treated and control score
ranges, with endpoints included. On the original data it is approximately
[0.000652575, 0.973525517]: 179 treated and 1,146 controls remain, while 6 and
1,344 respectively are excluded. For retained observations the bins are
`(0,.1]`, `(.1,.2]`, …, `(.9,1]`; the report includes all 20 bin/group cells,
zero counts and null means for empty cells. In particular bin 9 has 13 treated
people and no controls. Balance means are descriptive, do not prove covariate
balance, and do not test or establish positivity, no unmeasured confounding,
or a causal effect. The sample selection is based on fitted scores, and its
uncertainty is not quantified. The workflow deliberately produces **no treatment
effect or confidence interval**. The author's stratified outcome-regression
point estimate and valid treatment-effect uncertainty remain separate work.

The script rejects missing/nonfinite inputs, invalid binary indicators,
negative pretreatment earnings, duplicate or unordered keys, nonconverged or
rank-deficient fits, scores outside the open unit interval, empty range
intersection, and nonfinite stratum means. It exports all ordered source rows,
including excluded rows and their bounds; existing exports are not overwritten.
Earlier script effects are not rolled back on a later error. With the same
prepared `nswpsid.csv`, from a fresh directory:

```sh
/path/to/tabdat-explore-rs --json -f /path/to/repo/examples/mma25p1treatment/overlap.td > overlap_result.jsonl
Rscript /path/to/repo/examples/mma25p1treatment/overlap_reference.R /path/to/nswpsid.da1 overlap-reference
python3 /path/to/repo/examples/mma25p1treatment/check_overlap.py /path/to/nswpsid.da1 overlap_result.jsonl propensity_scores.csv overlap-reference
```

The independent base-R 4.6.1 `glm` fit and direct information calculation check the
14 coefficients and complete model covariance, all 2,675 scores and source
rows, bounds, inclusion and stratum assignments, and all group counts/means.
The tolerance is `1e-10 + 1e-9*abs(reference)`, with a stricter `1e-7` relative
check for tiny nonzero probabilities and p-values. The empirical-range rule is a
sample diagnostic rather than a general support guarantee; matching, effect
estimation, score-estimation uncertainty and the historical plot remain open.

## Untrimmed inverse-probability ATT

[`ipw_att.td`](ipw_att.td) extends the same assignment model to a different,
bounded analysis: an untrimmed normalized inverse-probability weighted estimate
of the average treatment effect on the treated (ATT). It uses all 2,675 source
people, rather than the empirical-range subset or the authors' stratified
regressions. If `p_i` is the fitted logit probability and `w_i=p_i/(1-p_i)` for
controls, the point estimate is the treated mean of `re78` minus the weighted
control mean `sum(w_i*y_i)/sum(w_i)`. Normalization allows the two groups' total
weights to differ. This is not the authors' Table 25.4 stratified estimator.

The 95% normal interval uses an independent-person estimating-equation
sandwich that includes the estimated logit, without a finite-sample correction.
Let `X` include the intercept, `H=sum(p_i*(1-p_i)*X_i*X_i')`,
`g=sum_control(w_i*(y_i-mu0)*X_i)/sum_control(w_i)`, and `s_i=X_i*(T_i-p_i)`.
The individual contribution is
`T_i*(y_i-mu1)/N1 - (1-T_i)*w_i*(y_i-mu0)/sum_control(w_i) - g'*H^-1*s_i`;
the reported variance is the sum of squared contributions. A weighted regression
with weights `p_i*(1-p_i)` solves the information-matrix system. Its displayed
coefficients and standard errors are **internal computation**, not treatment
effect estimates or their inference. The script exports each person's score,
weight, nuisance multiplier and contribution to permit auditing.

On the original source, the ATT is 3,027.875, estimated-score sandwich SE
879.856 and normal interval [1,303.389, 4,752.362]. The control weight effective
sample size `(sum w)^2/sum(w^2)` is only 14.08 out of 2,490 controls and the
largest weight is 36.77. This concentration, sparse common support and other
violations of asymptotic regularity can make the normal interval unreliable. No
trimming, weight clipping, overlap correction or sensitivity analysis is applied.

A causal ATT interpretation requires, among other conditions, consistency,
independent people, conditional exchangeability given the included covariates,
positivity for the treated target population and a correctly specified assignment
model. Fitted probabilities and finite sample weights do not prove those
conditions. The PSID controls are observational and the estimates need not have
a causal interpretation. The reported sandwich accounts for estimation of the
logit under regularity; it does not correct model misspecification, unobserved
confounding or lack of overlap.

The script admits 15–10,000 dense ordered people, finite outcomes and covariates,
binary treatment/indicators, nonnegative pretreatment earnings, both groups and
positive within-group outcome variation. It fails on rank or convergence errors,
nonrepresentable probabilities, weights, internal solve, influence terms or
variance, and refuses to overwrite its export. An exactly degenerate internal
weighted solve may also fail. Prior script effects are not rolled back.

With the same separately prepared `nswpsid.csv`, from a fresh directory:

```sh
/path/to/tabdat-explore-rs --json -f /path/to/repo/examples/mma25p1treatment/ipw_att.td > ipw_att_result.jsonl
Rscript /path/to/repo/examples/mma25p1treatment/ipw_att_reference.R /path/to/nswpsid.da1 ipw-att-reference
python3 /path/to/repo/examples/mma25p1treatment/check_ipw_att.py /path/to/nswpsid.da1 ipw_att_result.jsonl ipw_att_rows.csv ipw-att-reference
```

Independent R 4.6.1 `glm`, direct information-matrix inversion and grouped
estimating-equation sums validate all 2,675 source scores and influence terms,
the point estimate, interval and weight diagnostics at `1e-8 +
1e-9*abs(reference)`; tiny scores also require `1e-7` relative agreement. An
independent central-difference check validates the weighted-control-mean
propensity gradient. A separate 500-person synthetic fixture checks the
estimated-score sandwich against R. This is a source-specific composition, not
a general treatment-effects or matching command; the author's stratified
outcome model, overlap alternatives and additional analyses remain open.

## Overlap-weighted effect

[`overlap_effect.td`](overlap_effect.td) uses the same 2,675-person assignment
logit, but weights treated observations by `1-p` and controls by `p`. Its
point estimate is the difference of the two separately normalized weighted 1978
earnings means. Under consistency, conditional exchangeability, positivity and a
correct score model, the target is the average effect in the covariate
population tilted by `p(x)*(1-p(x))`. This overlap-population estimand is not
the ATT above: changing the weights changes the population and potentially the
effect when effects differ across people. The method is an additional sensitivity
analysis, not an author Table 25.4 replication. See [Li, Morgan and Zaslavsky
(2018)](https://doi.org/10.1080/01621459.2016.1260466).

On this source sample the weighted contrast is -656.710 with an estimated-score
sandwich SE of 928.826 and normal 95% interval [-2,477.176, 1,163.756]. The
weighted treated and control effective sample sizes are 93.59 and 154.85,
respectively. These numbers and the untrimmed ATT describe different targets;
their difference alone is not evidence of treatment-effect heterogeneity.
Weights are bounded by one, but that does not prove overlap or remove model
misspecification, unmeasured confounding, uncertainty in sample selection or
finite-sample limitations. The PSID comparison remains observational and a
causal interpretation requires the assumptions above. No trimming or clipping
is used.

Write `a_i=T_i*(1-p_i)`, `b_i=(1-T_i)*p_i`, `W1=sum(a_i)`,
`W0=sum(b_i)` and `mu1,mu0` for the normalized group means. The script reports
independent-person, no-small-sample-correction sandwich contributions
`a_i*(y_i-mu1)/W1 - b_i*(y_i-mu0)/W0 + g'*H^-1*X_i*(T_i-p_i)`, with
`H=sum(p_i*(1-p_i)*X_i*X_i')` and
`g=-sum(p_i*(1-p_i)*X_i*(T_i*(y_i-mu1)/W1+(1-T_i)*(y_i-mu0)/W0))`.
The variance is the sum of squared contributions and the interval uses a normal
reference. The estimated-score term includes how the fitted score changes the
weights and target. The weighted regression printed during the script is an
internal information solve; its coefficients and inference are not treatment
effects. All person-level weights and contributions are exported for audit.

The report also shows weighted means of all fitted design columns by group.
Their near equality follows from the in-sample logit score equations, rather
than providing an independent test of the assignment model or unmeasured balance.
The script enforces the same ordered keys, valid inputs and nondegenerate outcome
as the ATT example, plus finite positive group weights, finite balance means,
positive finite variance and representable effective sample sizes. It rejects
rank/convergence and nonfinite failures and does not overwrite an export; earlier
script effects are not rolled back. From a fresh prepared directory:

```sh
/path/to/tabdat-explore-rs --json -f /path/to/repo/examples/mma25p1treatment/overlap_effect.td > overlap_effect_result.jsonl
Rscript /path/to/repo/examples/mma25p1treatment/overlap_effect_reference.R /path/to/nswpsid.da1 overlap-effect-reference
python3 /path/to/repo/examples/mma25p1treatment/check_overlap_effect.py /path/to/nswpsid.da1 overlap_effect_result.jsonl overlap_effect_rows.csv overlap-effect-reference
```

Independent R 4.6.1 `glm`, direct matrix inversion, and a central numerical
propensity-gradient check validate all 2,675 original scores, weights and
influence terms, the full design balance, estimate, interval and weight
summaries at `1e-8 + 1e-9*abs(reference)`, with `1e-7` relative agreement for
tiny scores. A separate 500-person fixture validates the estimate, sandwich
and both effective sample sizes against R. This is a bounded source composition,
not a general overlap-weighting or matching command; the author's stratified
analysis and additional treatment illustrations remain open.

## Historical propensity-stratified comparison

[`stratified.td`](stratified.td) reproduces the **observed printed calculation**
from the author program's last Table 25.4 row, not a general matching or
covariate-adjusted treatment estimator. It fits the same assignment model,
intersects the two observed score ranges, and divides retained people into ten
fixed score intervals. Within each bin containing both groups it takes the
unadjusted 1978 earnings mean of treated people minus the control mean. It then
averages those bin differences using each bin's treated count. The report and
CSV include counts, group means and explicit status for every bin or source row.
Bins missing either group are excluded, rather than filled with zero.

The original sample has 179 treated people in the common range. Of these, 13
are in bin 9, which has no control observations; the reported average therefore
uses 166 treated people in nine supported bins and equals 1,562.7274. It is a
coarsened, selected-sample descriptive comparison, not automatically an ATT for
all 185 treated people. Within-bin remaining imbalance and unmeasured
confounding can be substantial; the empirical range and binning depend on fitted
scores and do not establish positivity or identification. Several supported
bins have very few controls. The author output reports **no standard error**;
the script explicitly returns a null standard error and no interval. A naive
fixed-bin variance would not account for estimated scores, data-dependent range
selection or sparse cells. Valid uncertainty for this procedure remains open.

The author program text says it adds age, age squared, education, no-degree,
Black, Hispanic and earnings controls within bins, but its recorded log shows
the `XLIST` assignment as a continued comment and its nine printed coefficients
match unadjusted group differences. Independent R regressions on the source data
show, for example, that adding those controls changes bin 1 from -4,410.947 to
-4,102.242 and bin 5 from 1,899.984 to 14,596.769. The log establishes the
observed calculation, not an inference that either approach identifies a causal
effect. Historical zero coefficients were skipped by the author loop; this
workflow retains a supported zero difference, since zero is a valid result.
The original source results are unaffected by that deliberate divergence.

The script checks the same ordered finite assignment-model inputs and score
range as the overlap diagnostic, rejects nonfinite bin/aggregate means and a
lack of supported bins, and refuses to overwrite its export. A failed script is
not transactional. From a fresh directory prepared as above:

```sh
/path/to/tabdat-explore-rs --json -f /path/to/repo/examples/mma25p1treatment/stratified.td > stratified_result.jsonl
Rscript /path/to/repo/examples/mma25p1treatment/stratified_reference.R /path/to/nswpsid.da1 stratified-reference
python3 /path/to/repo/examples/mma25p1treatment/check_stratified.py /path/to/nswpsid.da1 /path/to/mma25p1treatment.txt stratified_result.jsonl stratified_records.csv stratified-reference
```

R 4.6.1 independently fits the score model and ten bin calculations, verifies
unadjusted differences using separate within-bin regressions, and checks two
nominal covariate-adjusted regressions. The audit verifies the hash-pinned source
and author output, all 2,675 scores and assignments, ten cells and the overall
contrast at `1e-8 + 1e-9*abs(reference)`, with `1e-7` relative agreement for
tiny scores. It compares all nine supported printed coefficients to half of a
thousandth, acknowledging historical float storage and display rather than
claiming exact digits. A separate 500-person R control, supported-zero case and
invalid-input and export checks cover the CLI. Propensity-stratified uncertainty,
matching and the remaining Chapter 25 analyses are still required.

## Fixed-bandwidth kernel matching

[`kernel_matching.td`](kernel_matching.td) compares each NSW treated person's
1978 earnings with a normalized Gaussian-kernel weighted mean of PSID control
outcomes, using the DW99 assignment logit described above. Its fixed score
bandwidth is `h=0.06`; all controls contribute with replacement, and no caliper,
empirical-range restriction or trimming is applied. Equal scores receive equal
kernel weights, so there is no arbitrary tie-breaking. The result is the mean
treated-minus-matched outcome across all 185 treated people. The procedure is a
bounded kernel-matching illustration related to Table 25.6, not literal
reproduction of the author's kernel, bandwidth, support rule or reported number.

On the source data the fixed-bandwidth contrast is 842.393, the estimated-score
sandwich SE is 881.392 and the normal 95% interval is [-885.104, 2,569.890].
The smallest per-treated effective number of controls `1/sum(q_ij^2)` is 6.99,
the mean is 183.71, the largest nearest-control score distance is 0.08696, and
the maximum summed reuse weight for a control is 13.31. These diagnostics show
that some treated people have weak support and controls may be reused heavily;
they do not establish positivity or remove unobserved confounding.

The inferential target is the **fixed-bandwidth smoothed observed-data
functional**, not automatically the causal ATT. With a nonzero fixed bandwidth,
the weighted control mean generally differs from the conditional counterfactual
mean at a treated score. A causal ATT interpretation requires the usual
consistency, independent-person, conditional exchangeability and positivity
assumptions, a suitable score model, and additional conditions making that
smoothing discrepancy negligible. The reported interval does not include
finite-bandwidth bias, specification uncertainty, or unobserved confounding.
No bandwidth selection or sensitivity analysis is claimed.

For treated person `i` and control `j`, let `q_ij` be the Gaussian weight
normalized over controls, `m_i=sum_j(q_ij*y_j)`, `r_ij=y_j-m_i`,
`d_ij=p_i-p_j`, and `theta=mean_i(y_i-m_i)`. The direct empirical-distribution
influence contribution is `(y_i-m_i-theta)/N1` for treated people and
`-sum_i(q_ij*r_ij)/N1` for controls; the latter accounts for reuse. The score
term adds `g'*H^-1*X_k*(T_k-p_k)`, where
`H=sum_k(p_k*(1-p_k)*X_k*X_k')` and
`g=sum_ij(q_ij*r_ij*d_ij*(p_i*(1-p_i)*X_i-p_j*(1-p_j)*X_j))/(N1*h^2)`.
The variance is the sum of squared contributions, without a small-sample
correction, and the interval uses a normal reference. This sandwich treats the
bandwidth as fixed and accounts for estimating the logit. The weighted
regression printed by the script is an internal information-matrix solve; its
coefficients and inference are not matching effects. Person-level matches,
control reuse, score derivatives and influence terms are exported for audit.

The script requires the same finite, ordered and nondegenerate inputs as the ATT
example, strictly interior finite scores, and at most 2,000,000 treated–control
pairs. It rejects nonfinite pair weights, matches, influence values or positive
variance failures, and refuses to overwrite its export; earlier session effects
are not rolled back. Pair and influence reductions are ordered for reproducible
exports. This is a bounded composition, not a general matching command. It does
not implement radius or author-program matching; the ordinary
bootstrap is not assumed valid for fixed-neighbor matching (see [Abadie and
Imbens (2008)](https://doi.org/10.3982/ECTA6474)).

From a fresh prepared directory:

```sh
/path/to/tabdat-explore-rs --json -f /path/to/repo/examples/mma25p1treatment/kernel_matching.td > kernel_matching_result.jsonl
Rscript /path/to/repo/examples/mma25p1treatment/kernel_matching_reference.R /path/to/nswpsid.da1 kernel-matching-reference
python3 /path/to/repo/examples/mma25p1treatment/check_kernel_matching.py /path/to/nswpsid.da1 kernel_matching_result.jsonl kernel_matching_rows.csv kernel-matching-reference
```

Independent R 4.6.1 pair matrices and direct information inversion validate all
2,675 source records, matched values, control reuse, score derivatives,
influence terms, effect, interval and diagnostics at `1e-8 +
1e-9*abs(reference)`, with `1e-7` relative agreement for tiny scores. A central
numerical-gradient check uses a `1e-6` step: the larger `1e-5` step showed
truncation error for the largest squared earnings regressor, which was resolved
by reducing the step rather than widening the comparison tolerance. A separate
500-person R control validates the contrast, SE and effective match counts;
CLI tests cover invalid inputs, degenerate outcomes, pair-work bounds and
JSON/text no-clobber exports.


## Descriptive nearest-score comparison

A separate [nearest-score workflow](nearest_matching.md) matches within the
empirical common score range, averages exact ties, and reports outcome and
covariate means with control reuse. It explicitly withholds uncertainty; it is
a selected-sample descriptive comparison, not general nearest-neighbor
inference or a reproduction of the author's matching program.
