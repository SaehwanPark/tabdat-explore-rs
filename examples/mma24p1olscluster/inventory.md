# Chapter24 clustered household/individual companion inventory

## Scope and acceptance

This source-scoped inventory covers both May2005 §24.7 companion programs,
not every empirical illustration in Chapter24 or the book. It adds no runtime
capability. Every analytical source block must identify its data/preparation,
model, inference, post-estimation/report needs and either a linked executable
workflow with independent evidence or explicit required work. Administrative
Stata setup/log/clear commands are not empirical illustrations.

Acceptance for this documentation slice:

1. Pin both source programs and author logs by URL/SHA256 and cite source lines.
2. Map all descriptive, graphical, estimation, diagnostic, comparison and
   bootstrap blocks; distinguish source/log facts from native validation.
3. Link existing workflows and references without promoting partial coverage
   to whole-table parity; preserve unresolved precision/convention differences.
4. Reconcile SPEC, roadmap and coverage-ledger pointers and validate local links.

Important failure case: author comments, requested bootstrap counts or parser
acceptance must not be used as evidence of executable native coverage. No new
estimator, numerical result, historical-RNG parity or survey-design claim is
part of this slice.

## Pinned sources

Line numbers below refer to these exact publisher `.do` files (CRLF line
endings retained when hashing). Author logs are observations, not an independent
numerical oracle. Publisher data/programs/logs are not bundled.

| Artifact | Publisher URL | SHA256 |
| --- | --- | --- |
| Household program | [mma24p1olscluster.do](https://cameron.econ.ucdavis.edu/mmabook/mma24p1olscluster.do) | `74672f14199ad5925594132e223887db61eb35752624a567818f433e4b38b2d3` |
| Household log | [mma24p1olscluster.txt](https://cameron.econ.ucdavis.edu/mmabook/mma24p1olscluster.txt) | `5e74c8de58eaf833b0070d68b289d61fad43831f3eb8b57c06818a6bf549837e` |
| Individual program | [mma24p2poiscluster.do](https://cameron.econ.ucdavis.edu/mmabook/mma24p2poiscluster.do) | `746da2eca6d5970a045c137f106cfe33514e2f6e9595f26cb5451b3eb4bd2376` |
| Individual log | [mma24p2poiscluster.txt](https://cameron.econ.ucdavis.edu/mmabook/mma24p2poiscluster.txt) | `b86983173c9c712d10df522a0d3ce566fd7c03b77d9f9739b9e776c44223d33e` |

## Shared sample/design and status rules

The [household preparation/evidence](README.md) pins `vietnam_ex1.asc` and
`.dta`, label decoding, source ordering and conversion. There are 5,999 source
households, 993 missing outcomes and 5,006 complete cases in 194 communes.
H-model rows below use `lnexp12m ~ 1 + lnhhexp + age + sex + hhsize + farm + educ`.
FARM/EDUC are numeric codes and SEX is 1/2, not automatic factors. The source
additionally generates `HHEXP=exp(LNHHEXP)` for descriptive output; its ASCII
export omits that derived field. Descriptions use variable-wise available
source cases, not the regression cohort by default.

The [individual preparation/evidence](../mma24p2poiscluster/README.md) pins
`vietnam_ex2.asc` and `.dta`. There are 27,766 source people and 27,765 complete
cases in 194 communes. I-model rows use a log mean with intercept and ten slopes
`lnhhexp insurance sex age married illdays actdays injury illness educ`, outcome
`pharvis`. Conditional FE removes commune64's 94 all-zero people, leaving
27,671 people/193 groups and no common intercept; shared-gamma RE retains that
commune and integrates a shared mean-one gamma effect, not rowwise NB2.

Commune is a cross-sectional group, not time. Native `source_row` is an ordering
key added during preparation. None of these workflows uses survey weights,
strata, finite-population corrections or a causal identification design.

**Validated bounded** means the linked native script and independent R audit
cover the stated method/sample, subject to their documented limits. It does
not mean printed-table or whole-program parity. **Partial** means relevant
pieces exist but the specified output is not fully evidenced. **Open** means
required work with no validated workflow for this exact block. All open/partial
entries remain required, not exclusions. Linked workflow documents contain the
executable `.td`, reference `.R`, checker, exact inference conventions,
tolerances and failure/state evidence; this audit did not rerun their numerical
tests or change their evidence.

## Household program: 17 analytical/preparation/report blocks

| ID / source lines | Analysis, inference and post-estimation/report need | Status / executable workflow and independent evidence |
| --- | --- | --- |
| H-LOAD / 55–67 | Read/describe original DTA, rename eight fields, derive HHEXP | Partial: [preparation](README.md) independently audits projected fields and numeric labels on ASCII/DTA; native scripts use CSV, not native DTA. Original full-schema `desc`/`sum` and derived HHEXP are not covered by that projection audit. |
| H-DESC / 69–72 | Table24.3 household source summaries, including level HHEXP and commune | Open: validate variable-wise N/mean/sample SD/min/max against R on the source, separately from selected-case prediction summaries. Source says FARM replaces printed URBAN and acknowledges differences. |
| H-EXPORT / 74–77 | Eight-field ASCII source export, including missing outcomes and value labels | Partial: [prepare.py](prepare.py) plus [reference.R](reference.R) audit all prepared source rows; [DTA conversion](dta_reference.R) is reproducible. Recreating historical Stata `outfile` bytes/labels is not claimed. |
| H-OLS-IID / 84–86 | OLS coefficients, classical t/F, R²/N, stored olsiid | Validated bounded [OLS workflow](README.md), [script](household_ols.td): base-R QR/full covariance and source/prediction audit. |
| H-OLS-HC1 / 88–90 | Same OLS mean, row-HC1 t/F, stored olshet | Validated bounded [OLS workflow](README.md): R row-score sandwich; N/(N−K), t(N−K), not cluster inference. |
| H-OLS-CR1 / 92–94 | Same OLS mean, commune-CR1 t/F, stored olsclust | Validated bounded [OLS workflow](README.md): R commune scores; G/(G−1)*(N−1)/(N−K), t(G−1). Final lnhhexp test/interval and population predictions are native extensions. |
| H-RE-GLS / 96–100 | Random-intercept feasible GLS, coefficient/SE/z, components/theta, stored refgls | Validated bounded [equal-group-between workflow](household_equal_group_re.md), [script](household_equal_group_re.td): direct R/full covariance and 14 printed DTA coefficient/SE facts. Default [trace-corrected Swamy–Arora](household_re_gls.md) is independently validated but uses a different unbalanced component convention. |
| H-FE / 105–107 | Within commune FE, normalized intercept, slope inference, stored fe | Validated bounded [FE workflow](household_fe.md), [script](household_fe.td): full-dummy R QR/common-intercept covariance, all source and population/unit-specific predictions. |
| H-RE-ML / 112–116 | Gaussian marginal ML, joint information, components, likelihood/LR, stored remle | Validated bounded [ML workflow](household_re_ml.md), [script](household_re_ml.td): R profile/analytic joint information/nlme, 210 quantities and all predictions. Historical five coefficient-SE and both component-SD-SE differences remain unresolved; do not mark printed inference parity. |
| H-BP / 118–121 | Refit RE then Breusch–Pagan random-effect LM diagnostic | Validated bounded [BP LM workflow](household_bp_lm.md), [script](household_bp_lm.td): 69 independent R QR/plm quantities and all source/prediction rows on ASCII/precise DTA; balanced, unbalanced/singleton and negative-score controls. Uses the legacy squared-score chi-square1 reference, not a one-sided variance test or ML boundary LR; DTA LM matches logged432.75. Native workflow retains pooled OLS and does not need an RE fit. |
| H-HAUSMAN / 123–127 | Classical FE-vs-RE slope specification test | Validated bounded [joint Hausman workflow](household_hausman.md), [script](household_hausman.td): all six FE/equal-group-between RE slopes, separate native covariance scales and full matrices, parent-scaled SPD-only difference,139 independent R QR quantities/all source and prediction rows on ASCII/precise DTA, plus20 printed DTA author facts. DTA17.8864248463/p.00652238245 matches17.89/.0065. Retained RE ownership, model/design mismatch, indefinite/singular/cancellation and current-data/state/export paths are tested. Classical IID assumptions only; not robust/survey, common-scale or generalized/rank-reduced inference. |
| H-GEE / 129–133 | Gaussian identity exchangeable population-averaged fit, working inference, stored pa | Validated bounded [GEE workflow](household_gee.md), [script](household_gee.td): dense R/geeM and 16 printed DTA facts; RSS/N/pair moments/normal inference. Native uncorrected commune sandwich is an additional separate uncertainty choice. |
| H-REPORT-OLS / 135–138 | Named olsiid/olshet/olsclust coefficient/t/R²/N comparison | Partial: [household_ols.td](household_ols.td) produces the independently audited three-fit named comparison, not literal Stata formatting. Full printed-table reconciliation remains distinct. |
| H-REPORT-MIXED / 139–140 | Named pa/fe/refgls/remle coefficient/t/R²/N comparison | Open: fits exist separately, but no validated unified four-fit source report. Preserve estimator-specific z/t, covariance, sample, unavailable R² and likelihood meanings; do not fabricate a common statistic or silent refit. |
| H-BOOT-OLS / 142–155 | Whole-commune OLS B500, bias/SE/95% intervals and displayed estimate/SE ratios | Validated bounded [OLS bootstrap](household_ols_bootstrap.md), [script](household_ols_bootstrap.td): independent R refits/full B−1 covariance/type7 intervals, native non-Stata stream. Historical draws/interval method and the separate source `tols` display are not audited native outputs. |
| H-BOOT-FE / 157–166 | Whole-commune FE B500 and tfe display | Validated bounded [FE bootstrap](household_fe_bootstrap.md), [script](household_fe_bootstrap.td): all500 R QR distinct-copy fits, re-normalized intercept/full covariance/ordered draws. Historical repeated-ID/draw/interval parity and separate tfe display remain unclaimed. |
| H-BOOT-RE / 168–171 | Whole-commune RE-GLS B500 and tre display | Validated bounded [RE bootstrap](household_re_bootstrap.md), [script](household_re_bootstrap.td): all500 R component/quasi-GLS refits with distinct copied IDs/full covariance/draws. Conservative numerical gates apply; not fixed-theta bootstrap, ML bootstrap or historical tre/display parity. |

## Individual program: 17 analytical/preparation/report blocks

| ID / source lines | Analysis, inference and post-estimation/report need | Status / executable workflow and independent evidence |
| --- | --- | --- |
| I-LOAD / 60–69 | Read/describe original DTA, rename six fields | Partial: [preparation](../mma24p2poiscluster/README.md) audits projected source fields and labels on ASCII/DTA; full original-schema desc/sum is not independently covered. |
| I-DESC / 71–74 | Table24.3 variable-wise source summaries | Open: independent N/mean/SD/min/max report required, separate from model cohort. Source acknowledges LNHHEXP differences and absence of printed MEDEXP. |
| I-DETAIL / 75 | Detailed LNHHEXP quantiles/tails | Open: native summarize detail exists but no source-specific audited workflow; historical quantile, skewness/kurtosis and extreme-value conventions must be explicit. |
| I-FREQ / 77–79 | Table24.5 PHARVIS frequency/percent/cumulative percent | Open: source-specific all-available-outcome frequencies and independent audit required, not model-fitted countprob or selected-case summaries. Source acknowledges printed frequency differences, especially zero. |
| I-GRAPH / 81–82 | Discrete PHARVIS histogram plus kernel-density overlay | Open: choose/document bins, density bandwidth/kernel/boundaries, native rendering/export and source-aware independent ordinate/geometry checks; no validated histogram/KDE overlay currently linked. |
| I-EXPORT / 84–86 | Twelve-field ASCII source export | Partial: [source conversion/audit](../mma24p2poiscluster/README.md) preserves projected source ordering, missingness and code semantics; historical outfile bytes are not claimed. |
| I-POIS-IID / 93–95 | Pooled Poisson model covariance/normal95/LR, stored poisiid | Validated bounded [pooled workflow](../mma24p2poiscluster/README.md), [script](../mma24p2poiscluster/individual_poisson.td): R GLM/full covariance/source/prediction audit; retained-X continuous effects are an extension. |
| I-POIS-ROBUST / 97–99 | Same mean, row-score robust normal/Wald, stored poishet | Validated bounded [pooled workflow](../mma24p2poiscluster/README.md): independent R N/(N−1) row-score covariance; not OLS HC1 or clustered uncertainty. |
| I-POIS-CLUSTER / 101–103 | Same mean, commune-score normal/Wald, stored poisclust | Validated bounded [pooled workflow](../mma24p2poiscluster/README.md): G/(G−1) covariance; one printed clustered ACTDAYS SE final digit unresolved, no tolerance widening. |
| I-RE / 105–109 | Shared-gamma RE Poisson joint coefficients/dispersion/information/boundary LR, stored poisre | Validated bounded [RE workflow](../mma24p2poiscluster/individual_gamma_re.md), [script](../mma24p2poiscluster/individual_gamma_re.td): direct R integrated likelihood/full covariance/236 quantities/all source and marginal predictions; 29 printed DTA facts. |
| I-RE-REPEAT / 111–112 | Repeat RE with explicit i(COMMUNE) | Source repetition, not a separate covariance method: executable command has no cluster option despite the preceding comment. Native [RE workflow](../mma24p2poiscluster/individual_gamma_re.md) specifies id(commune); no robust-RE coverage inferred. |
| I-FE / 117–119 | Conditional Poisson FE, slopes/information/normal95, stored poisfe | Validated bounded [FE workflow](../mma24p2poiscluster/individual_conditional_fe.md), [script](../mma24p2poiscluster/individual_conditional_fe.td): R dummy-GLM/direct conditional audit, 179 quantities, 20 printed DTA facts, retained-normalization predictions and zero-total exclusion. |
| I-REPORT-POOLED / 124–130 | poisiid/poishet/poisclust coefficient/statistic/R²/N comparison | Partial: [pooled script](../mma24p2poiscluster/individual_poisson.td) emits three sequential independently audited fits, but does not store/compare them in a named source table. A unified comparison and printed Table24.6 reconciliation remain required. |
| I-REPORT-MIXED / 131–132 | poisre/poisfe coefficient/statistic/R²/N comparison | Open: no unified independently audited source comparison. Distinguish the FE cohort/conditional likelihood, RE full cohort/integrated likelihood and unavailable statistics. |
| I-BOOT-POOLED / 134–147 | Whole-commune pooled-Poisson bootstrap and tpois ratios | Validated bounded [pooled bootstrap](../mma24p2poiscluster/individual_pooled_bootstrap.md), [script](../mma24p2poiscluster/individual_pooled_bootstrap.td): 500 independent R QR refits/full covariance/type7 intervals/97,000 native-stream draws. Author executable/log B50, not claimed same uncertainty/RNG; separate tpois display unaudited. |
| I-BOOT-FE / 149–155 | Whole-commune conditional-FE bootstrap and tpoisfe ratios | Validated bounded [FE bootstrap](../mma24p2poiscluster/individual_conditional_fe_bootstrap.md), [script](../mma24p2poiscluster/individual_conditional_fe_bootstrap.td): 500 R refits/96,500 ordered draws, distinct-copy IDs and193-group sample. Nine of ten author original points match; MARRIED final-digit discrepancy and historical display/draw ambiguity remain. |
| I-BOOT-RE / 157–160 | Whole-commune shared-gamma-RE bootstrap and tpoisre ratios | Validated bounded [RE bootstrap](../mma24p2poiscluster/individual_gamma_re_bootstrap.md), [script](../mma24p2poiscluster/individual_gamma_re_bootstrap.td): 500 R joint refits/97,000 draws/full covariance, all194 groups. Ten of twelve author original points match; two final digits and historical repeated-ID treatment remain unresolved. |

## Source conflicts and next delivery

- P1 line51 calls ex2 household data; P2 lines53–56 call its people household
  data, claim N5006 and reverse the ex1 description. Actual data/log/native
  audits establish the household/individual cohorts above. Do not copy these
  comments into sample contracts.
- P2 header says Table24.6, analysis heading says p851, and report heading p852.
  These are source locators, not an independently resolved printed-page audit.
  P2 explicitly says its results differ in the second significant digit from
  the book; existing author-log matches do not resolve printed-table parity.
- P2 line138 says B500, but executable line139 sets50 and all three author-log
  bootstrap tables show50. P1 executable sets500 and its three log tables show500.
  Native workflows use B500/seed10001 and a specified non-Stata stream; source
  programs do not set that seed. Native percentile95 and source displayed
  normal/percentile/bias-corrected intervals must not be conflated.
- H-BP is a pooled-residual LM diagnostic; H-RE-ML's one-sided variance LR is
  not a substitute. H-HAUSMAN must not be implemented as a generic inverse of a
  possibly indefinite covariance difference or advertised as cluster-robust.
- Bootstrap coefficient/SE ratios are descriptive source displays, not evidence
  of valid finite-sample t tests. Current B−1 covariance/percentile evidence does
  not automatically validate those separate displays or their historical RNG.

**Next recommended source workflow:** unified H-REPORT-MIXED, preserving each
fit's scale, covariance, sample and available statistics rather than inventing
common R²/likelihood semantics. H-BP and H-HAUSMAN now have independent evidence
for their exact legacy/classical conventions, not robust/survey alternatives.
Descriptive/frequency/graphical and individual comparison entries also remain
required. This inventory closes neither Chapter24 nor
the full-book inventory requirement; no required illustration is excluded.
