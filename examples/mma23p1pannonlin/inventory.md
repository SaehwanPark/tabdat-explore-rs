# Chapter23 patents companion source inventory

## Scope and acceptance

This source-scoped inventory covers one March2005 `mma23p1pannonlin` program
and its May2005 log, not every empirical illustration in Chapter23 or the book.
The program's declared book locators are §23.3 pp792–795, Table23.1 and
Figure23.1; this is not a new independently inspected printed-book audit.
Low-risk documentation
slice; no runtime/model/parser/reference/fixture change or new numerical pass.
Each preparation, descriptive, graphical, fit, bootstrap and reporting block must
identify data/design, inference, post-estimation/report needs and executable/
independent evidence or explicit remaining required work. Administrative logging,
version/more/scheme and a commented-out dummy-GLM alternative are not executed
empirical analyses; their exclusion from the executed-block count is not a
product-scope exclusion of any required illustration.

Four acceptance cases before implementation:

1. Pin exact publisher program/log/data bytes by URL/SHA256, retain CRLF when
   hashing, and use exact program line ranges/log locators.
2. Account for every executed preparation/model/graph/bootstrap/report block,
   including unused generated variables and repeated PA/GEE routes; do not infer
   an executed estimator from a comment or commented-out code.
3. Trace workflow/R/checker evidence without promoting partial coverage to
   historical/table/whole-program parity. Distinguish nominal bootstrap counts
   from usable statistics, missing SEs from zeros and differing likelihoods.
4. Reconcile SPEC/roadmap/ledger links, check local targets and exact source
   line/block coverage. No unchanged numerical baseline rerun for docs-only work.

Important failure: declaring the author PA bootstrap to contain500 usable fits
because its header requests500 (the log actually displays338 per statistic), or
copying summary-table normal-RE zeros as valid SEs despite missing primary SEs.
No historical RNG/ID/optimizer resolution, complete chapter/book inventory,
parser-alias support, new estimator or exclusion of required work is implied.

## Pinned publisher sources

Raw program11532 bytes/332 lines, all CRLF; log opened23May2005. Source lines
below refer to this exact program, not normalized/reformatted copies. Author log
is evidence of the historical run, not an independent numerical oracle. Publisher
program/data/log are not bundled.

| Artifact | Publisher URL | SHA256 |
| --- | --- | --- |
| Program | [mma23p1pannonlin.do](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.do) | `e4b90cc60b52585fba6485d0cc9ed79ce0b9c9fc6fd0f390a435b15ef7dd98aa` |
| Log | [mma23p1pannonlin.txt](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.txt) | `92f68c76063cf083217724f1b1b45cfaf8ae7a8c7ce0b4a1d5f62accd86dc208` |
| Data | [patr7079.asc](https://cameron.econ.ucdavis.edu/mmabook/patr7079.asc) | `87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8` |

## Shared data/design and status rules

The raw source has346 firm records/25 free-format fields: five static variables,
ten LOGR values and ten PAT counts for1970–79. The program reshapes to3460 rows
with two-digit year70..79 **before** generating LPAT, DPAT, RANDD, five lags and
four year indicators. The log records336 nonmissing wide ARDSSIC values/3360
long values; ten firms have missing nonmodel metadata. Current
[preparation](pooled_poisson.md)/[script](prepare.sh) projects all nine fields
needed for1975–79, using source-record ordinal ID, not CUSIP, and full years
1975..1979. It does not validate every full-decade derived value or reproduce
the historical DTA/labels/describe/xtsum output.

All fit/graph rows below use346 firms/1730 observations in1975–79, with LOGR as
the sole slope (`xextra` is empty), no weights/offset/year effects. Missing
ARDSSIC never selects the model. There are337 zero counts, including22 firms
with all five counts zero. Linear LPAT uses ln(0.5) at zero; count and NLS/GEE
models use untransformed PAT. Conditional Poisson FE instead retains324
positive-total firms/1620 rows and no common intercept. Whole-firm resampling
samples346 firms except FE's324 selected positive firms, as observed in the log.
Native bootstrap copies have explicitly distinct IDs; this is not evidence of
historical duplicate-ID behavior. Source `nreps=500`, repeated `set seed 10001`
and all historical interval types are not a native RNG/interval equivalence.

**Validated bounded** means the linked script/R/checker supports the specified
sample and analytical method within its published limits, not literal source
syntax, all stored output or historical parity. **Partial** identifies evidenced
pieces plus missing exact output/inference/reconciliation. **Open** means no
source-specific validated output is linked. Required analytical/data/inference
portions of open/partial entries remain required, not exclusions. Recording
source DTA/WMF/table artifacts does not create a new exact-format/byte-parity
gate: a documented, independently validated semantic adaptation can satisfy
its bounded outcome without cloning Stata internals. Workflow guides name the executable `.td`,
independent `.R` reference, checker, numerical gates and failure/state evidence;
this inventory does not rerun or upgrade that evidence. Sequential fit reports
are not automatically named immutable model comparisons.

## Preparation and specification:11 blocks

| ID / program lines | Data/preparation, inference or output need | Status / executable and independent evidence |
| --- | --- | --- |
| P-READ / 93–104 | Free-format25-field source read; ordinal ID; wide describe/summarize | Partial: [preparation/evidence](pooled_poisson.md), [prepare.sh](prepare.sh) and raw-token R references preserve the1975–79 nine-field projection. Full wide-schema summaries/labels are not audited outputs. |
| P-RESHAPE / 108–110 | Ten-year PAT/LOGR long reshape, describe/summarize | Partial: each current workflow/R oracle validates1730 projected records, not all3460 long records/ten-year summaries. |
| P-LPAT / 113–117 | Substitute0.5 only for zero PAT, then natural log and labels | Validated bounded [linear script](linear_panel.td)/[guide](linear_panel.md): all1730 source/design rows, including337 substitutions, independently checked; full-decade derivation/labels remain part of P-SAVE. |
| P-DPAT / 119–121 | PAT>0 indicator and label, generated but not modeled here | Open: no source-specific all3460-row DPAT audit/export. This is not an executed logit/probit fit. |
| P-RANDD / 123–125 | EXP(LOGR) levels and labels, generated but not modeled here | Open: no source-specific full-decade RANDD audit/export. Numeric overflow/missing semantics must be explicit in any workflow. |
| P-LAGS / 127–137 | Declare ID/year; five LOGR lags, before dropping1970–74 | Open: no all3460-row source-aware lag/key/missing-boundary audit. Computing lags only after the1975 restriction would lose valid earlier-source histories. No lag enters the actual empty-xextra fits. |
| P-YEAR / 139–146 | Indicators for source years76..79, generated but unused in fits | Open: no source-specific all-row indicator audit/export; do not insert them into the single-LOGR fitted design. |
| P-SAVE / 149–153 | Describe/summarize full derived long data, drop NEWPAT, save DTA, resummarize | Partial: canonical CSV projection/ownership is validated by [native FE](native_conditional_fe.md), [RE](native_gamma_re.md), [GEE](native_poisson_gee.md) and other guides; no complete derived historical DTA/label/full-schema output. |
| P-XTSUM / 154 | Overall/between/within summaries of all long-source variables | Open: regression variance components/within R² are not a source `xtsum` audit; requires explicit differing nonmissing counts and panel summary conventions. |
| P-SPEC / 160–165 | B500 and empty additional-regressor global | Validated bounded designs/B500 in linked fit/bootstrap guides below. No omitted year/lag/static covariate is silently added; historical seed/usable-count ambiguity remains. |
| P-SAMPLE / 178–179, 211–212 | Reload saved data and select years75..79 before linear/count models | Validated bounded: all linked raw-source/R audits enforce1730 canonical rows/346 firms; FE's additional zero-total exclusion is explicit. Native full-year keys differ representationally, not in selected periods. |

## Graph and fits:16 blocks

| ID / program lines | Model, inference and post-estimation/report need | Status / executable and independent evidence |
| --- | --- | --- |
| P-FIG / 187–194 | Figure23.1 scatter LPAT/LOGR, LOWESS, pooled linear fit and WMF export | Partial: [SVG adaptation](figure.md)/[script](figure.td), independent R data/smoother and actual SVG geometry checks. Historical Stata tied-curve/precision/style/WMF parity remains open; corrected axis labels are deliberate. |
| P-LIN-POOL / 197–198 | LPAT pooled OLS with firm-cluster covariance; store linolspan | Validated bounded [linear workflow](linear_panel.md)/[script](linear_panel.td):106 R/plm/full-dummy quantities across three fits/all source/predictions; OLS CR1 and t/F(345). Source named-table coverage is separate P-REPORT-LIN. |
| P-LIN-FE / 201–202 | LPAT within FE, normalized intercept/classical inference; store linfe | Validated bounded: same [linear audit](linear_panel.md) now includes full-dummy OLS/normalized common-intercept SE/t/nonzero p/95CI/full2x2 covariance, classical F and1383 residual df.106 R quantities/all1730 rows and24 printed facts pass; independent raw covariance/tail checks and26 forgeries cover the repaired stale checker. No production change, cluster-common/unit-effect uncertainty or named-table parity. |
| P-LIN-RE / 205–206 | LPAT balanced RE-GLS, components/theta/z; store linre | Validated bounded [linear workflow](linear_panel.md): independent plm/quasi-GLS/full covariance. Not ML, a shared Gaussian likelihood or posterior-effect uncertainty. |
| P-POIS-IID / 217–218 | PAT pooled Poisson model covariance/normal inference; store poisiid | Validated bounded [pooled workflow](pooled_poisson.md)/[script](pooled_poisson.td):103 R quantities/19030 source/report/prediction cells across three covariance fits. |
| P-POIS-ROB / 221–222 | Same mean, independent-row score sandwich; store poishet | Validated bounded [pooled workflow](pooled_poisson.md): N/(N−1) row correction, not OLS HC1 or firm clustering. |
| P-POIS-CLUSTER / 225–226 | Same mean, firm-score sandwich; store poispan | Validated bounded [pooled workflow](pooled_poisson.md): G/(G−1), normal/Wald, all346 firms including22 zero totals. |
| P-POIS-FE / 232–233 | Conditional Poisson FE; store poisfe | Validated bounded [native FE](native_conditional_fe.md)/[script](native_conditional_fe.td):14 unchanged independent R dummy-GLM/conditional quantities, all1730 source/prediction records/110 NULL exclusions, full scalar information, owned retained-group normalization. [SQL counterpart](conditional_fe.md) is separate; normal model-based covariance, not firm-robust or unit-effect uncertainty. |
| P-POIS-RE / 245–246 | Shared-gamma RE Poisson/joint dispersion/covariance/boundary LR; store poisre | Validated bounded [native RE](native_gamma_re.md)/[script](native_gamma_re.td):37 unchanged R quantities/full joint covariance/all1730 source/predictions, six printed facts; historical lnalpha precision remains unresolved. [SQL counterpart](gamma_re_poisson.md) retains pooled start, not RE. |
| P-POIS-NORMAL / 249–250 | Shared Gaussian log-intercept Poisson/joint information; store poisrenormal | Partial: [bounded SQL adaptive-quadrature workflow](normal_re_poisson.md)/[script](normal_re_poisson.td),42 independent direct-R-integral fields/all1730 source/predictions; retains pooled state, no native normal-RE estimator. A [native workflow contract](native_normal_re.md) is WIP, not parser/fit/state completion; independent direct-R integration and owned population-prediction acceptance are declared. Historical fit materially differs and has missing SEs; not validated historical convergence/inference. |
| P-PA / 253–254 | xtpoisson PA route, exchangeable Poisson/log working covariance; store poispa | Validated bounded analytical route via [native GEE](native_poisson_gee.md)/[script](native_poisson_gee.td):27 unchanged legacy-R quantities/both full matrices/all1730 rows/346 scores. It is population-averaged GEE, not gamma/normal integrated RE; no native xtpoisson-pa syntax claim. Historical slope final digit remains unresolved. |
| P-PA-ROB / 257–258 | Same PA mean, firm-score covariance; store poispapan | Validated bounded [native GEE](native_poisson_gee.md), fixed working scale1 and G/(G−1) sandwich/normal95, independently audited; distinct from P-PA working covariance and P-BOOT-PA resampling. No alias/snapshot parity claim. |
| P-GEE / 263–264 | Explicit exchangeable Poisson/log GEE, working covariance; store poisgee | Validated bounded [native GEE](native_poisson_gee.md): source repeats PA equations through xtgee. Native fit owns actual GEE; [SQL composition](exchangeable_gee.md) instead retains pooled start. General/unbalanced moments remain required. |
| P-GEE-ROB / 267–268 | Explicit GEE with firm sandwich; store poisgeepan | Validated bounded [native GEE](native_poisson_gee.md): reports both covariance choices, not a separate fitted mean. Paired source routes do not constitute four different mean models. Named comparison/printed precision remain separate. |
| P-NLS / 271–272 | Independent Gaussian/log GEE exponential NLS, working covariance; store nls | Validated bounded [NLS workflow](exponential_nls.md)/[script](exponential_nls.td): native nl fit plus independently audited SQL RSS/N Gauss–Newton working covariance/normal intervals,27 R fields/all1730 rows/346 scores. Native nl's RSS/(N−2)/t report differs and is separately labeled; no Gaussian-log xtgee alias claim. |
| P-NLS-ROB / 275–276 | Same exponential mean, firm sandwich; store nlspan | Validated bounded [NLS workflow](exponential_nls.md): G/(G−1) full firm-score Gauss–Newton sandwich with normal intervals, not native clustered-nl fit state. Native SQL auxiliary p-values are not supplied; no invented Gaussian likelihood/global-optimality claim. |

## Bootstrap and comparison reports:10 blocks

| ID / program lines | Resampling, inference and report need | Status / executable and independent evidence |
| --- | --- | --- |
| P-BOOT-POOL / 285–287 | Seed10001; whole-firm Poisson B500, slope/intercept, save poisbootse | Validated bounded [bootstrap](pooled_poisson_bootstrap.md)/[script](pooled_poisson_bootstrap.td):500 R refits/full B−1 covariance/type7 percentile95,173000 exact ordered non-Stata draws. Historical normal/percentile/bias-corrected intervals/seed stream not reproduced by that evidence. |
| P-BOOT-FE / 290–292 | Whole-positive-firm conditional FE B500, slope only, save poisfebootse | Validated bounded [SQL bootstrap](conditional_fe_bootstrap.md)/[script](conditional_fe_bootstrap.td):500 R conditional fits/162000 draws/distinct copied IDs,324-group sample. The separate [native bootstrap](native_conditional_fe_bootstrap.md)/[script](native_conditional_fe_bootstrap.td) now admits324 firms under the conditional-specific512 cap:500 independent R refits/three dummy-GLMs,162000 Python-oracle draws/full scalar covariance/type7 limits and retained original predictions pass, with26 rejected forgeries. Pooled stays256; gamma now separately permits512. Native/SQL streams differ. No historical draws/interval parity. |
| P-BOOT-RE / 295–297 | Whole-firm gamma RE B500, slope/intercept, save poisrebootse | Validated bounded [SQL bootstrap](gamma_re_bootstrap.md)/[script](gamma_re_bootstrap.td):500 independent R joint coefficient/lnalpha refits/full covariance/173000 draws, distinct effects per copy. Nuisance lnalpha is refit even though source requests only two coefficient statistics. The separate [native bootstrap](native_gamma_re_bootstrap.md)/[script](native_gamma_re_bootstrap.td) now admits all346 firms/22 zero-total firms at cap512. All500 joint R refits/31 fields/full3x3 covariance,173000 independent draws and unchanged original predictions pass; raw-score/empirical identities reject30 forgeries including shared nuisance/covariance corruption. Historical repeated-ID/material SE/interval differences remain unresolved; native/SQL streams differ, and pooled remains256. |
| P-BOOT-PA / 300–302 | Whole-firm PA GEE, requested B500; save poispabootse | Partial historical: [distinct-copy bootstrap](exchangeable_gee_bootstrap.md)/[script](exchangeable_gee_bootstrap.td) validates500 legacy-R/base-R refits/173000 draws with explicit fail-abort policy. Author log1135–1157 requests500 but reports338 usable values for each statistic, slope SE.1072904 versus distinct-copy.0418896. Failure/omission/duplicate-ID/RNG mechanism is unverified; no equivalence or all500-author-success claim. |
| P-BOOT-NLS / 303–309 | Reseed; whole-firm Gaussian-log/independent NLS B500; save nlsbootse | Validated bounded [bootstrap](exponential_nls_bootstrap.md)/[script](exponential_nls_bootstrap.td):500 R two-parameter refits/full covariance/type7 intervals/173000 draws; native nl cross-checks and bounded local-root policy, not a general bootstrap or historical interval/RNG claim. |
| P-BOOT-DISPLAY / 310–313 | List four saved bootstrap-SE matrices | Partial: linked workflows independently report corresponding native-stream SEs, but no unified audited source-order matrix display/historical uncertainty. Source lists pooled/FE/RE/PA, **not** nlsbootse; do not invent a fifth executed list. |
| P-REPORT-LIN / 319–320 | linolspan/linfe/linre b/SE/statistic/N/ll/R²/TSS/RSS/MSS/RMSE/df comparison | Partial: [linear script](linear_panel.td) emits three independently audited fits, not this named immutable source table. Original normalized-intercept-SE/full-covariance audit now passes; common/within/conditional R², transformed GLS scales/unavailable likelihood provenance and complete named-table evidence still must be explicit. |
| P-REPORT-POOLED / 321–322 | poisiid/poishet/poispan named coefficient/SE/statistic/comparable-statistics table | Partial: [pooled script](pooled_poisson.td) emits three sequential independently audited covariance fits; no unified source-specific named comparison/printed Table23.1 audit. No count R²/TSS/RSS is invented for blank source cells. |
| P-REPORT-MIXED / 323–324 | poisfe/poisre/poisrenormal/poispa/poispapan named comparison | Open exact source report: individual analytical pieces above do not validate a unified table. Different FE/full cohorts and conditional/integrated/no likelihood, nuisance parameters and missing source normal-RE SEs require typed provenance. The author summary's0.000 does not turn missing primary SEs into zero uncertainty. |
| P-REPORT-GEE / 325–326 | poisgee/poisgeepan/nls/nlspan named mean/covariance comparison | Open exact source report: linked GEE/NLS outputs are independently audited, but no unified named working/sandwich/auxiliary-NLS report. Normal references, fixed Poisson scale1 versus Gaussian RSS/N, unavailable likelihood/R² and native nl's different default inference must not be conflated. |

## Source conflicts, historical evidence and remaining work

- Program26–29 omits the exponential in a schematic Poisson-mean comment;
  executed Poisson/log commands and positive exponential means define the
  models. The comments calling PA a random-effects fit do not establish an
  integrated gamma/normal likelihood. Program186 says Figure21.1; its overview
  and graph section locate Figure23.1/p792, with explicitly corrected axes.
  Program209 says Table32.1; surrounding text/overview identifies Table23.1.
  These are source comments, not distinct illustrations or new models.
- Program235–239 encloses dummy-variable Poisson in a block comment: it is not
  an executed alternate source fit. Independent R dummy GLM is real validation
  evidence for conditional FE, not evidence that Stata ran this commented code.
- Log196 gives346→3460 reshape;425–508 reports full-decade xtsum, not1730-row
  model summaries. LOGRL5 has1730 available rows (log493), but this does not
  mean all summary variables share that sample. Native estimation projection
  cannot substitute for source-wide variable-specific summaries/lag histories.
- Log799–849 normal RE stops after the displayed third full-model likelihood
  value−6261.9825 with coefficient/lnsig2u/scale SEs missing. Its later mixed
  table (1272–1312) shows0.000 SE placeholders. Do not validate those as finite
  zero SEs, infer successful historical convergence from table presence, or
  change native/direct-integral inference to reproduce them.
- PA and explicit GEE source logs868–983 print the same working versus robust
  pair, but the bounded converged float64 slope differs from their.5595302.
  Existing float32/nine-step probes support possible contributors, not proof
  of historical internals. Log1149's PA-bootstrap observed slope is.5595301;
  its338 usable statistics and500 nominal header must both be retained. Other
  four bootstrap tables show500 usable values. The cause of the PA shortfall
  is not established; no silent omission/redraw is licensed in native workflows.
- Source31–36 explicitly distinguishes default panel SEs from panel-robust SEs;
  Table23.1-style “panel” labels are not a blanket firm-robust claim. Model-based
  FE/gamma RE, GEE working, group sandwich and bootstrap remain different.
- Historical gamma lnalpha precision, GEE slope/stopping, repeated-ID/bootstrap
  SE/interval/RNG behavior, normal-RE missing inference/unverified convergence,
  Figure23.1 tied-curve/style and named-report/statistic
  reconciliation remain required. Normal-RE bootstrap is not executed in this
  program; its existing roadmap requirement is not erased by this inventory.

**Completed follow-up:** the original linear FE normalized-intercept covariance/
SE/t/CI audit (P-LIN-FE) now passes independent full-dummy R and raw-design checks;
P-REPORT-LIN remains partial rather than being closed by those scalar/matrix
facts. Reassess missing graduate methods and source priorities before another
slice; no inventory ordering is a mandatory phase gate. Full-decade
preparation/summaries and named pooled/mixed/GEE reports also remain required.
Unavailable historical core/RNG evidence is a blocker to literal parity, not a
reason to widen tolerances or declare those requirements complete. This37-block
companion inventory closes neither every Chapter23 nor every book illustration.

## Validation performed

All three fresh publisher downloads match pinned hashes. A source-specific
comment/line-coverage check verifies37 unique nonoverlapping block ranges cover
all117 non-administrative executable/continuation lines in the332-line CRLF
program; commented dummy GLM is excluded. Independent log extraction confirms
usable bootstrap counts500/500/500/338/500 and primary normal-RE missing SEs.
All433 local links across the five changed Markdown files resolve; no NBSP or
diff whitespace errors. This is documentation/source-evidence validation, not
Stata execution or a general Stata parser. Artifacts/check log are under
`/tmp/tabdat-ch23-inventory/`. No runtime/reference/fixture/Rust examples changed;
numerical tests, doctests and CI were not rerun or claimed. Existing bounded
workflow evidence is linked rather than relabeled as a new regression pass.
