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
integer seed does not establish shared random draws across software. The author
log also prints normal, percentile and bias-corrected bootstrap intervals;
current native type7 percentile95 intervals do not establish all three historical
interval conventions. Those output differences remain explicit for full-program
closure even where a bounded SE/covariance workflow is validated below.

## Executable-block inventory

| ID / program lines | Preparation and model | Inference / post-estimation / report required | Native workflow and independent evidence | Status / unresolved requirement |
| --- | --- | --- | --- | --- |
| C21-P1-DATA / 80–96 | Read eight source fields, generate year dummies; describe/summarize, save working data, panel structure and within/between summaries | Confirm 532 people/T10, variable summaries and within/between decomposition | [Preparation and sample contract](README.md#data-and-exact-samples), [converter](prepare.sh); model workflows check keys/source rows | Partial: key/sample preparation covered; no complete executable counterpart/audit for author `describe`, all-variable `summarize`, `xtdes` and `xtsum` output block |
| C21-P1-POLS / 140–153 | All 5,320 rows, lnhr on intercept and lnwg | Classical, HC1 and person-cluster covariance; coefficient/95% inference and named reports | [pooled.td](pooled.td), [R reference](reference.R), [evidence](README.md#polsfd-inference-and-interpretation) | Validated bounded; historical float32 facts separate from native float64 |
| C21-P1-POLS-BOOT / 157–159 | Whole-person pairs resampling, refit pooled OLS | B500, seed10001; slope/intercept bootstrap SEs and report | [pooled bootstrap](pooled_bootstrap.md), [script](pooled_bootstrap.td), [R reference](pooled_bootstrap_reference.R) | Validated bounded native RNG; historical draw/SE parity not established |
| C21-P1-FIG211 / 162–169 | Original pooled rows, OLS and LOWESS | Data marks, smoothed and linear fits, labelled figure | [Figure21.1](pooled_figure.md), [script](pooled_figure.td), [R reference](pooled_figure_reference.R) | Partial: independently audited native SVG adaptation; historical tied-curve geometry unresolved |
| C21-P1-BE / 173–177 | Equal-person means of lnhr/lnwg, 532 rows; intercept and wage | Classical Between uncertainty and report | [between.td](between.td), [R reference](between_reference.R), [evidence](README.md) | Validated bounded; native HC1 is an extension, not an executed option in this program |
| C21-P1-BE-BOOT / 183–185 | Resample whole people and refit Between slope/intercept | B500, seed10001; bootstrap SEs and report | [B500 script](between_bootstrap_b500.td), [R reference](between_bootstrap_reference.R), [evidence](between_bootstrap_b500.md) | Validated bounded native B500 target:18 independent R summaries/all532 person means/predictions and exact raw input; manual B200 unchanged. Continuing native stream, not historical Stata RNG/repeated-ID/all-interval parity |
| C21-P1-FIG212 / 188–197 | One mean row/person after Between transform, OLS/LOWESS | Mean-data marks, two fitted curves and labelled figure | [Figure21.2](between_figure.md), [script](between_figure.td), [R reference](between_figure_reference.R) | Partial: audited source-scale SVG adaptation; historical ties unresolved |
| C21-P1-FE / 201–205 | All rows, person-absorbed lnhr on lnwg | Classical FE slope/covariance/DF, normalized common intercept and panel fit statistics | [within.td](within.td), [R reference](within_reference.R), [evidence](README.md#within-fe-inference-retained-effects-and-predictions) | Partial: slope/effects/predictions and classical unit-effects F covered; [classical normalized common-intercept inference](classical_intercept.md), full joint covariance/_cons lincom/test now independently validated by full dummy QR (39 original fields/all5,320 predictions plus controls). Remaining historical statistics/full reporting stay open; pairs-bootstrap uncertainty is separate |
| C21-P1-FE-BOOT / 211–213 | Refit FE on resampled people; report slope and normalized level intercept | B500, seed10001; full reported coefficient uncertainty | [Workflow/evidence](fe_level_bootstrap.md), [script](fe_level_bootstrap.td), [raw-copy R QR reference](fe_level_bootstrap_reference.R) | Validated bounded native-stream target: all500 refits/266,000 exact draws, per-draw sample means/normalization, full B-1 covariance/type7 intervals and original native FE predictions. Zero empirical slope variance retained; old centered auxiliary-OLS bootstrap unchanged. This bootstrap is not the separate classical analytic intercept target or a general resampling API; historical RNG/all-interval parity remains open |
| C21-P1-FIG213 / 216–225 | Within deviations plus grand means, auxiliary OLS/LOWESS | Transformed-data marks/curves and normalization; separate FE inference | [Figure21.3](within_figure.md), [script](within_figure.td), [R reference](within_figure_reference.R) | Partial: audited source-scale SVG adaptation; historical tied curves unresolved; auxiliary OLS uncertainty is not FE uncertainty |
| C21-P1-FD / 233–255 | Annual adjacent-pair differences, 4,788 pairs; dlnhr on intercept and dlnwg | Classical, person-cluster and HC1 inference and named reports | [first_difference.td](first_difference.td), [R reference](reference.R), [evidence](README.md#data-and-exact-samples) | Validated bounded: native raw within-person lags avoid cross-person/gap differences; unused extra covariate differences not needed by active model |
| C21-P1-FD-BOOT / 258–260 | Resample whole-person annual-pair blocks, refit FD | B500, seed10001; slope/intercept uncertainty and report | [FD bootstrap](fd_bootstrap.md), [script](fd_bootstrap.td), [R reference](fd_bootstrap_reference.R) | Validated bounded native RNG; historical draws not established |
| C21-P1-FIG214 / 263–270 | Annual differences, OLS/LOWESS | Difference-data marks, curves and labelled figure | [Figure21.4](fd_figure.md), [script](fd_figure.td), [R reference](fd_figure_reference.R) | Partial: audited native SVG adaptation; source level-axis labels corrected to changes; historical tied curves unresolved |
| C21-P1-REGLS / 274–278 | All rows; Swamy–Arora RE-GLS lnhr on intercept/lnwg | Model-based covariance, components, population predictions and report | [random_effects.td](random_effects.td), [R reference](random_effects_reference.R), [evidence](README.md) | Validated bounded; native unit-score extension is separate from this source default |
| C21-P1-REGLS-BOOT / 285–287 | Resample people; **refit variance components/theta per draw**, then RE-GLS | B500, seed10001; population-scale intercept and wage bootstrap uncertainty | [Component-refitting workflow/evidence](random_effects_gls_refit_bootstrap.md), [script](random_effects_gls_refit_bootstrap.td), [R/plm reference](random_effects_gls_refit_bootstrap_reference.R) | Validated bounded native-stream target: all500 refits/266,000 draws, signed/truncated components, theta, population-intercept/slope covariance/type7 intervals and original predictions audited. Fixed-theta workflow unchanged; historical Stata draw/duplicate-ID/all-interval parity unresolved; no general resampling API |
| C21-P1-REML / 291–295 | All rows, Gaussian random-intercept marginal ML | Coefficients, joint component uncertainty, likelihood and boundary test; reporting | [random_effects_ml.td](random_effects_ml.td), [independent R fixture](../../crates/tabdat-stats/tests/fixtures/random_effects_ml_reference.R), [evidence](README.md) | Partial: native interior information/component inference validated; small historical Stata8 SE differences remain unresolved |
| C21-P1-REML-BOOT / 301–303 | Resample people with distinct copied unit identities; refit marginal ML/components | B500, seed10001; slope/intercept bootstrap SEs and report; convergence/boundary failure policy | [Bounded workflow/evidence](random_effects_ml_bootstrap.md), [script](random_effects_ml_bootstrap.td), [R reference](random_effects_ml_bootstrap_reference.R) | Validated bounded native-stream coefficient target: all500 refits/266,000 draws, full B−1 covariance/type7 intervals and original predictions audited; boundary fits retained. Historical Stata RNG/duplicate-ID/all-interval parity unresolved; no general ML bootstrap API |
| C21-P1-PA / 307–308 | Same original panel; Gaussian identity-link population-averaged exchangeable GEE | Working correlation/scale, coefficient covariance and report for author `xtreg ..., pa` | [Workflow/evidence](gaussian_pa.md), [script](gaussian_pa.td), [R gee/dense reference](gaussian_pa_reference.R) | Validated bounded coefficient/covariance/normal95 target: legacy Pearson/RSS-N, full working and uncorrected unit-sandwich matrices, all5,320 predictions/532 scores and12 printed facts. Negative-correlation controls pass; not relabelled RE. No owned GEE state/general API/p-values or literal historical iteration/output replay |
| C21-P1-HAUSMAN / 310–322 | Same-sample classical FE versus RE-GLS | Covariance-difference Hausman statistic; explicit efficiency assumptions | [Classical Hausman](classical_hausman.md), [script](classical_hausman.td), [R reference](classical_hausman_reference.R) | Validated bounded one-slope classical comparison; not robust to arbitrary within-person dependence; companion robust auxiliary workflow is separate |
| C21-P1-REPORT / 327–339 | Saved model estimates and six bootstrap SE matrices | Two multi-model tables plus PA table; N/ll/r2/tss/rss/mss/rmse/df_r where defined; six bootstrap SE reports | [Current reporting limits](README.md), [pooled](pooled.td)/[FD](first_difference.td)/[Between](between.td) named reports | Partial: no exact full-source comparison table or six-matrix report; FE/RE/PA snapshots, fit-statistic conventions and unresolved bootstrap rows must be settled before closure |

## Next implementation and remaining inventory

For literal historical bootstrap parity, also inspect the old resampler's
handling of duplicated IDs; this inventory does not establish that convention.
Native refitting must give resampled copies distinct unit identities so an
original person drawn twice represents two independent cluster copies, not one
artificially doubled-length panel.

The [RE-ML whole-person bootstrap composition](random_effects_ml_bootstrap.md)
now validates the bounded coefficient target on original5,320-row MOM with
500 independent R refits/266,000 exact draws, distinct copies, per-draw component
refits, boundary/convergence guards, full coefficient covariance and reproducible
exports. The separate [component-refitting RE-GLS bootstrap](random_effects_gls_refit_bootstrap.md)
now validates all500 Swamy–Arora refits/266,000 draws, theta and population-scale
intercept uncertainty instead of fixed quasi-transforms. Neither composition
installs a general resampling API or establishes historical Stata draw/duplicate-ID/
all-interval parity. The [Gaussian PA composition](gaussian_pa.md) now validates
the independently specified legacy moment/scale convention, both full coefficient
covariances/normal95, every original prediction and unit score, with negative-rho
controls and explicit retained pooled-OLS state. No general/owned GEE API or
p-values are installed. The distinct [normalized FE level-intercept bootstrap](fe_level_bootstrap.md)
now validates all500 raw-copy QR refits and original native FE predictions,
including zero empirical variance and failed-draw guards. The old centered
auxiliary-OLS workflow is unchanged. The separate [classical analytic normalized
common-intercept workflow](classical_intercept.md) now adds full joint covariance,
Student-t95/_cons restrictions and independent full dummy QR evidence, without
manufacturing unit-cluster common inference. Next reassess companion-program inventory/source alignment and complete
comparison/reporting.
The separate [first-program B500 Between workflow](between_bootstrap_b500.md)
now validates its native replication-count target while retaining manual B200;
historical replay and full first-program coverage remain unclaimed.

The [manual-program inventory](manual_inventory.md) now maps29 separate source
requirements and attaches the manual within/fixed-GLS bootstrap and robust
auxiliary Hausman evidence without erasing the distinct targets above. The live
publisher filenames are `mma21p2panmanual.do`, `mma21p3panresiduals.do` and
`mma21p4pangls.do`; the overview's second/fourth names are stale HTTP404 aliases.
Detailed residual/GLS inventories and the full book inventory remain required.
No required illustration has been excluded.
