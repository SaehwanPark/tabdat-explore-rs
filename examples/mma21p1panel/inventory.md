# Chapter 21 first author-program inventory

**Scope:** executable analysis blocks of the March 2005
[`mma21p1panfeandre.do`](https://cameron.econ.ucdavis.edu/mmabook/mma21p1panfeandre.do),
not a complete Chapter 21 or book inventory. The program identifies §21.3.1–3,
pp.709–714, Table21.2 and Figures21.1–21.4; its overview says Table21.1 but
its executable section labels and final display say Table21.2. Companion
programs and illustrations not represented here remain required to inventory.

## Slice contract

This documentation slice splits the family-level ledger into independently
checkable source-program requirements. Acceptance: account for every executable
analysis block; identify data/preparation, model, inference and post-estimation;
link an executable native workflow and independent evidence where available;
and preserve unresolved outputs as required work. Important failure case:
fixed-transform OLS resampling must not count as variance-component-refitting
RE resampling. No estimator, numerical result, tolerance or public interface is
changed. No existing validation is rerun or newly claimed by this inventory.

## Source identity and status rules

Sources inspected in full on 2026-11-04 (SHA-256 of downloaded bytes):

- Program: `775890ef25903477e5be24027c070705910247daf46f9ed7b13d0ed0b48cc44a`.
- [Author output](https://cameron.econ.ucdavis.edu/mmabook/mma21p1panfeandre.txt):
  `3c4e65bdfcd6fe8f77fe32361bc29e041ad7caf9268ba35ecf52e2c78d5b88f6`.

Line locators below refer to that 344-line program, including blank lines.
These hashes identify inspected references, not a license to redistribute them.
See the [workflow README](README.md#data-and-exact-samples) for source access,
conversion and data hash. All rows use original `MOM.dat`, not Chapter22's
higher-precision source: 532 men × 10 years (1979–1988), 5,320 observations,
`lnhr lnwg kids ageh agesq disab id year`. The executable globals include **no
additional regressors**; commented year-dummy/socioeconomic alternatives are not
active analyses. Native preparation preserves source decimal values, checks
person/year keys and retains source-row provenance. Historical Stata float32
arithmetic is a separately documented comparison, not the native data contract.

- **Validated bounded:** the linked native target has independent evidence;
  this does not imply literal historical output or whole-table completion.
- **Partial:** a linked workflow covers only stated components or a different,
  explicitly identified adaptation. The remainder is required.
- **Planned:** no linked validated executable for this exact target. An em dash
  in the workflow/evidence field means missing, not unnecessary.

The evidence column links existing independent-reference code and documentation,
not fresh executions performed by this documentation change. Reproduction
commands, audit tolerances, sample guards and output/failure semantics reside in
the linked workflow notes. All bootstrap targets require original estimates,
replicate parameter estimates, uncertainty and reproducible reporting; a shared
integer seed does not establish shared random draws across software.

## Executable-block inventory

| ID / program lines | Preparation and model | Inference / post-estimation / report required | Native workflow and independent evidence | Status / unresolved requirement |
| --- | --- | --- | --- | --- |
| C21-P1-DATA / 78–97 | Read eight source fields, generate year dummies; describe/summarize, save working data, panel structure and within/between summaries | Confirm 532 people/T10, variable summaries and within/between decomposition | [Preparation and sample contract](README.md#data-and-exact-samples), [converter](prepare.sh); model workflows check keys/source rows | Partial: key/sample preparation covered; no complete executable counterpart/audit for author `describe`, all-variable `summarize`, `xtdes` and `xtsum` output block |
| C21-P1-POLS / 137–151 | All 5,320 rows, lnhr on intercept and lnwg | Classical, HC1 and person-cluster covariance; coefficient/95% inference and named reports | [pooled.td](pooled.td), [R reference](reference.R), [evidence](README.md#polsfd-inference-and-interpretation) | Validated bounded; historical float32 facts separate from native float64 |
| C21-P1-POLS-BOOT / 153–157 | Whole-person pairs resampling, refit pooled OLS | B500, seed10001; slope/intercept bootstrap SEs and report | [pooled bootstrap](pooled_bootstrap.md), [script](pooled_bootstrap.td), [R reference](pooled_bootstrap_reference.R) | Validated bounded native RNG; historical draw/SE parity not established |
| C21-P1-FIG211 / 159–168 | Original pooled rows, OLS and LOWESS | Data marks, smoothed and linear fits, labelled figure | [Figure21.1](pooled_figure.md), [script](pooled_figure.td), [R reference](pooled_figure_reference.R) | Partial: independently audited native SVG adaptation; historical tied-curve geometry unresolved |
| C21-P1-BE / 170–181 | Equal-person means of lnhr/lnwg, 532 rows; intercept and wage | Classical Between uncertainty and report | [between.td](between.td), [R reference](between_reference.R), [evidence](README.md) | Validated bounded; native HC1 is an extension, not an executed option in this program |
| C21-P1-BE-BOOT / 183–186 | Resample whole people and refit Between slope/intercept | B500, seed10001; bootstrap SEs and report | [between_bootstrap.td](between_bootstrap.td), [R reference](between_bootstrap_reference.R), [evidence](README.md) | Partial: existing independent native target uses B200, not this program's B500; historical RNG differs |
| C21-P1-FIG212 / 188–199 | One mean row/person after Between transform, OLS/LOWESS | Mean-data marks, two fitted curves and labelled figure | [Figure21.2](between_figure.md), [script](between_figure.td), [R reference](between_figure_reference.R) | Partial: audited source-scale SVG adaptation; historical ties unresolved |
| C21-P1-FE / 201–210 | All rows, person-absorbed lnhr on lnwg | Classical FE slope/covariance/DF, normalized common intercept and panel fit statistics | [within.td](within.td), [R reference](within_reference.R), [evidence](README.md#within-fe-inference-retained-effects-and-predictions) | Partial: slope, normalized effects/current predictions and classical unit-effects F covered; individual/common intercept uncertainty and remaining historical statistics not covered |
| C21-P1-FE-BOOT / 212–215 | Refit FE on resampled people; report slope and normalized level intercept | B500, seed10001; full reported coefficient uncertainty | [Related transformed-OLS bootstrap](within_bootstrap.td), [R reference](within_bootstrap_reference.R), [limitations](README.md) | Partial: linked script resamples fixed centered data with an auxiliary OLS intercept, **not** this exact FE level-intercept bootstrap |
| C21-P1-FIG213 / 217–228 | Within deviations plus grand means, auxiliary OLS/LOWESS | Transformed-data marks/curves and normalization; separate FE inference | [Figure21.3](within_figure.md), [script](within_figure.td), [R reference](within_figure_reference.R) | Partial: audited source-scale SVG adaptation; historical tied curves unresolved; auxiliary OLS uncertainty is not FE uncertainty |
| C21-P1-FD / 230–261 | Annual adjacent-pair differences, 4,788 pairs; dlnhr on intercept and dlnwg | Classical, person-cluster and HC1 inference and named reports | [first_difference.td](first_difference.td), [R reference](reference.R), [evidence](README.md#data-and-exact-samples) | Validated bounded: native raw within-person lags avoid cross-person/gap differences; unused extra covariate differences not needed by active model |
| C21-P1-FD-BOOT / 263–266 | Resample whole-person annual-pair blocks, refit FD | B500, seed10001; slope/intercept uncertainty and report | [FD bootstrap](fd_bootstrap.md), [script](fd_bootstrap.td), [R reference](fd_bootstrap_reference.R) | Validated bounded native RNG; historical draws not established |
| C21-P1-FIG214 / 268–277 | Annual differences, OLS/LOWESS | Difference-data marks, curves and labelled figure | [Figure21.4](fd_figure.md), [script](fd_figure.td), [R reference](fd_figure_reference.R) | Partial: audited native SVG adaptation; source level-axis labels corrected to changes; historical tied curves unresolved |
| C21-P1-REGLS / 279–290 | All rows; Swamy–Arora RE-GLS lnhr on intercept/lnwg | Model-based covariance, components, population predictions and report | [random_effects.td](random_effects.td), [R reference](random_effects_reference.R), [evidence](README.md) | Validated bounded; native unit-score extension is separate from this source default |
| C21-P1-REGLS-BOOT / 292–295 | Resample people; **refit variance components/theta per draw**, then RE-GLS | B500, seed10001; population-scale intercept and wage bootstrap uncertainty | —; [related fixed-theta script](random_effects_gls_bootstrap.td), [R reference for that different target](random_effects_gls_bootstrap_reference.R) | Planned exact target: fixed-theta transformed OLS does not satisfy per-draw RE refitting or population-intercept uncertainty |
| C21-P1-REML / 297–306 | All rows, Gaussian random-intercept marginal ML | Coefficients, joint component uncertainty, likelihood and boundary test; reporting | [random_effects_ml.td](random_effects_ml.td), [independent R fixture](../../crates/tabdat-stats/tests/fixtures/random_effects_ml_reference.R), [evidence](README.md) | Partial: native interior information/component inference validated; small historical Stata8 SE differences remain unresolved |
| C21-P1-REML-BOOT / 308–311 | Resample people with distinct copied unit identities; refit marginal ML/components | B500, seed10001; slope/intercept bootstrap SEs and report; convergence/boundary failure policy | — | Planned: no RE-ML bootstrap workflow/evidence; analytic unit-score sandwich is not resampling |
| C21-P1-PA / 313–316 | Same original panel; Gaussian identity-link population-averaged exchangeable GEE | Working correlation/scale, coefficient covariance and report for author `xtreg ..., pa` | — | Planned: RE point-estimate similarity is not an independently validated PA/GEE workflow; Chapter23 Poisson GEE has a different response/link |
| C21-P1-HAUSMAN / 318–328 | Same-sample classical FE versus RE-GLS | Covariance-difference Hausman statistic; explicit efficiency assumptions | [Classical Hausman](classical_hausman.md), [script](classical_hausman.td), [R reference](classical_hausman_reference.R) | Validated bounded one-slope classical comparison; not robust to arbitrary within-person dependence; companion robust auxiliary workflow is separate |
| C21-P1-REPORT / 330–339 | Saved model estimates and six bootstrap SE matrices | Two multi-model tables plus PA table; N/ll/r2/tss/rss/mss/rmse/df_r where defined; six bootstrap SE reports | [Current reporting limits](README.md), [pooled](pooled.td)/[FD](first_difference.td)/[Between](between.td) named reports | Partial: no exact full-source comparison table or six-matrix report; FE/RE/PA snapshots, fit-statistic conventions and unresolved bootstrap rows must be settled before closure |

## Next implementation and remaining inventory

The missing RE-ML whole-person bootstrap is a bounded candidate following this
inventory: original 5,320 rows/532 people, B500, mean model lnhr~1+lnwg,
refit both variance components per draw, distinct identities for duplicate
people, explicit convergence/boundary handling, full coefficient bootstrap
covariance and reproducible exports. Choose a documented native RNG rather than
claim historical Stata draws. Independently refit every draw in R; verify copy
identities, no silent dropped/failed replicates and script/file failure behavior
before marking that row validated. This paragraph is a candidate contract,
**not implementation or a claim that a general bootstrap command supports ML**.
RE-GLS component-refitting bootstrap and Gaussian PA remain separate requirements.

The source overview also names `mma21p2panfeandre.do`,
`mma21p3panresiduals.do` and `mma21p4panpangls.do`. This inventory does not assert
those programs or all book illustrations have been exhausted. Existing manual
within/GLS bootstrap and robust auxiliary Hausman evidence must be attached to
those companion blocks when inspected, not used to erase the distinct targets
above. No required illustration has been excluded by this slice.
