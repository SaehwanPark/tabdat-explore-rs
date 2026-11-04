# Chapter21 manual-program inventory

**Scope:** all29 analysis/preparation/reporting requirements of the March2005
`mma21p2panmanual.do`, not a claim of complete Chapter21/book inventory or execution.
The [first-program inventory](inventory.md) remains separate. Native implementation,
source equivalence and independently audited inference are distinct statuses here.
Shared analyses overlap the first program;29 source requirements do not mean29
new or independent textbook illustrations.

## Publisher catalog and provenance

The live [publisher program index](https://cameron.econ.ucdavis.edu/mmabook/mmaprograms.html)
(lines1014–1065 as retrieved2026-11-04) lists:

| Program | Actual source | Coverage inventory |
| --- | --- | --- |
| First: FE/RE | [mma21p1panfeandre.do](https://cameron.econ.ucdavis.edu/mmabook/mma21p1panfeandre.do) | [20 first-program requirements](inventory.md) |
| Second: transformed OLS | [mma21p2panmanual.do](https://cameron.econ.ucdavis.edu/mmabook/mma21p2panmanual.do) / [output](https://cameron.econ.ucdavis.edu/mmabook/mma21p2panmanual.txt) | This document's29 requirements |
| Third: residual analysis | [mma21p3panresiduals.do](https://cameron.econ.ucdavis.edu/mmabook/mma21p3panresiduals.do) | Detailed inventory still required |
| Fourth: pooled OLS/GLS | [mma21p4pangls.do](https://cameron.econ.ucdavis.edu/mmabook/mma21p4pangls.do) | Detailed inventory still required |

The overview inside both first/second programs instead spells the second
`mma21p2panfeandre.do` and fourth `mma21p4panpangls.do`. Those exact live URLs
returned HTTP404 on2026-11-04. They are stale overview names, **not** evidence of
two additional implemented/absent analyses. The actual filenames above are supported
by live index links, successful HTTP200 source retrieval and matching program headers.
An exact Wayback CDX query for `cameron.econ.ucdavis.edu/mmabook/mma21p4panpangls.do`
(status200 filter, collapsed digests) returned an empty list; no archived content or
capture timestamp was recovered. That does not prove the alias never existed.

SHA256 of retrieved, unmodified source bytes (line references below preserve the
360-line manual program; do not normalize CRLF before checking hashes):

| Source | SHA256 |
| --- | --- |
| Program index | `d9f01aa7013e38ecf715b59a63f70f6de0b2a3c8cacbb80c2e47f3343fb11728` |
| Manual .do,13291 bytes | `0d02574be8958fbb7c67f41e488da9eb5c5d221d48acb106ce926f2f6c4e97f7` |
| Manual .txt,50856 bytes | `4de5e4335efb939deacf30a52b533b0959b371e1e78b9d33e64652fce56f126d` |
| Residual .do | `6c4ccfa560eb8f818412e87895e4c205f51c1e00865346f3b0c532902b7a43e8` |
| Actual GLS .do | `dde40722e3de67cb2b447792e28ffc14e7b6144ad6dae89ecb23f0e0fa694b6f` |

Raw MOM.dat is the same5,320-row/532-person/1979–1988 eight-column source pinned
in [inventory.md](inventory.md). This program explicitly assumes one wage slope,
complete balanced T10 people and no extra covariates. It is not the higher-precision
Chapter22 data. Its header cites§21.3.1–3 pp709–714; block7 cites§21.4.3 pp717–719,
and the final tables identify Table21.2 p710. No publisher program/data is vendored.

## Semantics that prevent false coverage

- Baseline native Between/FE/RE-GLS/ML fits run first; GLS and ML each save their
  own variance components and `theta=1-sqrt(sigma_e²/(10*sigma_u²+sigma_e²))`.
  The later OLS transformations freeze those original theta values. They are not
  component-refitting bootstraps or marginal-ML observed-information inference.
- `N=5320`, `G=532`, two-parameter auxiliary OLS has residual DF5318; canonical
  absorbed one-slope FE uses4787. Equal-person Between OLS has N532/DF530.
  The augmented Hausman auxiliary regression has three parameters/DF5317.
  Reusing canonical FE covariance does **not** validate manual auxiliary covariance.
- The centered FE auxiliary intercept is numerically zero, not the normalized
  level intercept. The RE auxiliary intercept is `(1-theta)*population_intercept`.
  Intercept labels/scales and named coefficient ordering must remain explicit.
- Ordinary auxiliary OLS uses RSS/(N-K); row-HC1 uses N/(N-K) score correction and
  is **not panel robust**. Person-cluster auxiliary OLS uses grouped scores and
  G/(G-1)*(N-1)/(N-K), t/F denominator G-1. Canonical nested-unit FE corrections
  can differ; do not substitute its already-validated cluster SE for this row.
  Ordinary/row-HC1 auxiliary references are working comparisons, not automatically
  valid canonical FE inference: demeaning induces dependence and estimates unit
  means. The clustered interpretation still assumes independent people and
  adequate cluster asymptotics; not all rows are valid under the same error law.
- The pooled-OLS label says not relevant and has no active fit. Despite a broad
  heading mentioning FD, the active transformations here are mean/quasi
  differences, not an annual-lag FD regression; do not invent missing fit blocks.
- Global `nreps=500` does not override the Between block's literal B200. That
  block has **no preceding explicit seed reset**. FE/GLS/ML bootstrap blocks each
  reset10001 and use B500. Our B200 workflow chooses explicit native seed10001
  for reproducibility; it is not a replay of unspecified source-session RNG.
- Lines302–303, labelled/stored as `remlehet`, actually fit **GLS-transformed**
  variables with HC1. The output log repeats this at863–879, and its final ML
  comparison at1018ff contains that GLS row. Do not silently call it ML-HC1 or
  substitute a corrected ML design while claiming literal historical equivalence.
  A future workflow must label source-as-written versus any corrected extension.
- The nonrobust auxiliary Hausman statistic is distinct from the native classical
  covariance-difference test. The source prints squared auxiliary t statistics;
  our [robust workflow](robust_hausman.td) separately labels finite-sample F versus
  asymptotic chi-square calibration and assumes independence across people.

## Existing evidence keys

- **BE:** [between.td](between.td), [R inference](between_reference.R), and
  [README evidence](README.md). Classical/HC1 mean-record inference is audited.
- **BASE-FE:** [within.td](within.td), [within R](within_reference.R), and the
  distinct [classical normalized-intercept increment](classical_intercept.md).
- **BASE-GLS/ML:** [GLS](random_effects.td)/[R](random_effects_reference.R),
  [ML](random_effects_ml.td); independent ML kernel/reference evidence is linked
  in README. Historical Stata8 ML-SE differences remain unresolved.
- **FE-AUX-BOOT:** [within_bootstrap.td](within_bootstrap.td)/[R](within_bootstrap_reference.R):
  source500 raw-copy bootstrap summaries/predictions audited, not a dedicated
  audit of all classical/HC1/cluster auxiliary-OLS inference rows.
- **GLS-AUX-BOOT:** [fixed-theta script](random_effects_gls_bootstrap.td)/[R](random_effects_gls_bootstrap_reference.R):
  theta, quasi/population scaling,500 bootstrap summaries and original predictions
  audited; this is not ML-theta or a complete analytic covariance-row comparison.
- **HM-CL:** [robust_hausman.td](robust_hausman.td)/[R](robust_hausman_reference.R)
  and [README interpretation](README.md). Do not replace it with
  [classical covariance-difference Hausman](classical_hausman.md).
- **BE-BOOT:** [manual B200](between_bootstrap.td)/[R](between_bootstrap_reference.R),
  retaining B200 defaults; [first-program B500](between_bootstrap_b500.md) is separate.

## Exact manual-program requirements

`Validated bounded` means the linked declared native target has independent
original-sample evidence, **not** complete Stata state/statistic/output parity.
`Partial` names usable evidence and what is still unaudited. `Missing` means no
complete source-specific workflow/reference has been established by this inventory;
generic parser/OLS capability alone does not close that requirement.

| ID / .do lines | Exact outcome / inference or artifact | Current status and evidence |
| --- | --- | --- |
| P2-DATA / 80–92 | Load exact MOM source, describe/sample counts, declare global500 | Partial: prepare/source/key guards and EDA exist; source-wide manual preparation artifact remains below |
| P2-BASE-BE / 101–102 | Native Between baseline/store, balanced sample and panel metadata | Partial: BE validates equivalent mean-record fit; no native Between owner/panel metadata or full saved comparison |
| P2-BASE-FE / 104–105 | Canonical absorbed FE baseline/store | Partial: BASE-FE core/covariance/effects supported; complete saved comparison remains absent |
| P2-BASE-GLS / 107–112 | Native RE-GLS baseline/store and original component-derived theta | Partial: BASE-GLS core validated; no named RE snapshots/full saved comparison or whole historical table equivalence |
| P2-BASE-ML / 114–119 | Marginal ML baseline/store and separate ML-derived theta | Partial: BASE-ML point/component/information targets validated; old Stata SE and manual auxiliary targets remain distinct |
| P2-PREP / 125–144,159–235 | Ordered ten-year person means, FE deviations and both fixed quasi-transforms; long prepared-data artifact | Partial: individual long-form SQL workflows cover subsets; no complete all-transform/MOM2-equivalent reproduction artifact. Literal wide reshape syntax/binary Stata format is not the analytical target |
| P2-BE-IID / 146–147 | Equal-person mean OLS, classical covariance/DF530, stored fit | Validated bounded: BE; see separate comparison requirement |
| P2-BE-HC1 / 150–151 | Same mean-record design, independent-person HC1, stored fit | Validated bounded: BE |
| P2-BE-BOOT / 154–155 | B200 mean-record pairs, no prior seed reset, both coefficient SEs | Validated bounded native-stream: BE-BOOT; explicit native seed extension, not historical draw replay |
| P2-FE-IID / 240–241 | Centered auxiliary OLS with intercept, DF5318 | Partial: FE-AUX-BOOT supplies executable original fit/points; dedicated analytic covariance/DF/CI reference for this row not established |
| P2-FE-CL / 244–245 | Same centered design, person-cluster OLS correction/DF531 | Missing complete dedicated auxiliary comparison; canonical FE cluster evidence is not substituted |
| P2-FE-BOOT / 249–251 | Fixed centered inputs, whole-person B500/reset10001, both coefficients | Validated bounded native-stream: FE-AUX-BOOT; centered intercept not level-intercept uncertainty |
| P2-FE-HC1 / 254–255 | Centered design, row-HC1 comparison (not panel robust) | Missing dedicated source-specific workflow/audit |
| P2-GLS-IID / 264–265 | Original GLS-theta quasi OLS, transformed intercept/DF5318 | Partial: GLS-AUX-BOOT supplies theta/fit/scaling/points, not dedicated full analytic inference audit |
| P2-GLS-CL / 268–269 | Fixed GLS-theta quasi OLS, person-cluster covariance/DF531 | Partial: canonical RE-GLS cluster exists but differs in intercept representation; dedicated auxiliary comparison remains |
| P2-GLS-BOOT / 273–275 | Fixed GLS-theta whole-person B500/reset10001 | Validated bounded native-stream: GLS-AUX-BOOT; not component-refitting GLS bootstrap |
| P2-GLS-HC1 / 278–279 | Fixed GLS quasi design, row-HC1 comparison | Missing dedicated source-specific workflow/audit |
| P2-ML-IID / 288–289 | Fixed original ML-theta quasi OLS, transformed intercept/DF5318 | Missing dedicated workflow/audit; marginal ML information is not this covariance |
| P2-ML-CL / 292–293 | Same fixed ML quasi design, person-cluster OLS | Missing; native full-joint ML score sandwich does not close this row |
| P2-ML-BOOT / 297–299 | Fixed ML quasi design, whole-person B500/reset10001 | Missing; component-refitting marginal-ML bootstrap is a different target |
| P2-ML-HC1-SOURCE / 302–303 | Source-as-written GLS HC1 stored under ML label | Source anomaly verified in code/log; not a validated ML-HC1 target. Requires explicit labelling/decision in any future reproduction |
| P2-HM-IID / 314–316 | GLS quasi response/design plus within wage, ordinary auxiliary coefficient t² | Missing dedicated auxiliary-iid workflow/audit; not covariance-difference Hausman |
| P2-HM-CL / 320–322 | Same augmented design, clustered auxiliary coefficient t² | Validated bounded: HM-CL, with calibration/assumptions explicitly labelled |
| P2-REPORT-BE / 331–332 | Stored native/OLS/HC1 comparison, named coefficients and N/ll/R²/sums/RMSE/DF | Partial: BE named OLS reports exist, but native Between-panel row/whole source comparison not supplied |
| P2-REPORT-FE / 333–334 | Canonical/auxiliary-iid/HC1/cluster comparison with correct intercepts/DF | Missing complete mixed canonical/auxiliary table; no native FE snapshots |
| P2-REPORT-GLS / 335–336 | Native/quasi iid/HC1/cluster comparison and scale-aware coefficients | Missing complete saved comparison; do not equate Gaussian working OLS likelihood with panel likelihood |
| P2-REPORT-ML / 337–338 | Marginal-ML/quasi iid/source-labelled GLS-HC1/quasi cluster comparison | Missing complete comparison; preserve source anomaly and different likelihood/covariance meanings |
| P2-REPORT-BOOT / 341–346 | Four coefficient-SE matrices, named ordering, separate fixed-transform targets | Partial: BE/FE/GLS bootstrap summaries exist; fixed-ML target and complete combined report remain missing |
| P2-REPORT-DIAG / 349–354 | Both original theta values and robust/nonrobust auxiliary H scalars together | Partial: separate GLS/ML component and HM-CL outputs exist; combined source report/nonrobust auxiliary target missing |

## Delivery and verification limits

No estimator or numerical result changed in this inventory slice. Source/code/log
blocks, existing scripts/references and current capability documents were inspected;
all active baseline/auxiliary fits, bootstraps, Hausman and table/matrix outputs
are assigned above. Preparation/labels, statistical meaning and missing evidence
are recorded rather than hidden behind generic command availability. Local links,
source hashes/line mapping and diff are checked; unchanged Rust/numerical baseline
is not rerun for documentation-only changes. This is not new numerical validation.

A next useful implementation slice is the **manual FE auxiliary covariance
comparison**: exact centered N5320/G532/T10 design, original OLS versus row-HC1
versus person-cluster corrections/DF, independently checked on the original sample
and a small control. Explain the deterministic centered intercept and do not
advertise its floating-point t/p artifacts as inference. Keep it distinct from
canonical FE and its normalized common-intercept uncertainty. Remaining residual/
GLS program inventories and complete report/ML auxiliary work stay required;
do not make every remaining inventory a prerequisite to this bounded delivery.
