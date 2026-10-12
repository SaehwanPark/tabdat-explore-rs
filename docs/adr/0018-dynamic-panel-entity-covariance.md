# ADR 0018: Collapsed one-step difference GMM and entity inference

- Status: Accepted for bounded feasibility, not product xtabond parity
- Date: 2026-10-12

## Decision

Validate one-step difference Arellano–Bond, one lagged dependent variable,
collapsed lag-2-only outcome instrument and differenced strictly-exogenous
predictors, individual effects removed and no time effects/intercept. Native
OPT_C without OPT_T/L/D/A gives this mode with entity-summed one-step sandwich,
no finite-N correction. Compare independent R plm pgmm/vcovHC and external Python
moment algebra at predeclared abs 1e-9 + rel 1e-8; exact native sample mask,
keys, groups and instrument count. This is just identified, not evidence about
other weighting matrices or multi-lag/overidentified/system/two-step models.

Pinned Python executor uses pooled IVGMM with one level-lag instrument and default
two-step estimation; robust covariance treats transformed rows as independent.
Its coefficients match here because just identification makes weighting irrelevant.
Its SEs differ by ~0.06924 from entity inference. Preserve originals, label the
covariance decision, do not widen tolerance or claim default command parity.
R fallback also has distinct defaults; do not silently describe routes as one
public estimator. Broader runtime normalization remains deferred.

Initial native attempt failed singular: a supplied GMM specification disables
implicit regular x instruments. Explicit checked second separator and x instrument
list are required. Do not solve this with data/tolerance changes. This correction
produces R/Python agreement ~4.4e-16 / ~1.1e-16 for all exposed numerical fields.

## Boundary

Independent dpanel-plugin selects only source-verified dpanel.so under existing
shared-core/lazy serialized ownership controls, excluding libR. Dataset pd/structure
set/restored within one call on success/failure; owned residual-mask copy checks
MODEL type/full_n/buffer before read. Native MODEL/arrays/names remain RAII owned.
Closed lag/spec/duplicate exogenous-instrument roles and indices checked before C;
identifier characters cannot inject GMM expressions. Bound prototype to 4..32
periods, 8..4096 entities, 1..16 predictors and low-level dataset <=32 variables.
These are explicit feasibility caps, not arbitrary full dynamic-panel support.

No runtime R/Python, dependency/waiver, AR/Hansen/Sargan/CI/post-estimation/model-state
or packaging/backend acceptance. Native initialization diagnostic, comprehensive
integer/allocation/sanitizer/leak/license/relocation and other architectures remain
open. [Report](../feasibility/dynamic-panel.md) records complete evidence and limits.
