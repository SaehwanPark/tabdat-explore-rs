# RE idiosyncratic residual correlations — WIP contract

**Exact source outcome:** Cameron–Trivedi (2005) §21.3.4 pp713–715, author
[residual program](residual_inventory.md) lines85–87,95–109 and historical output
lines344–362. Original MOM N5320/G532/T10 1979–1988. Native balanced model-based
Swamy–Arora RE-GLS fit and population prediction; compose original-sample person
random-effect predictions and idiosyncratic residuals, correlate the latter
across people for all ordered year pairs and export source/prediction rows and
100-cell matrix. `ure` is **not** the native population residual: current
[Stata xtreg postestimation](https://www.stata.com/manuals/xtxtregpostestimation.pdf)
describes `e` as e_it, `ue` as u_i+e_it and `u` as random effect. With balanced
RE components, uhat_i=[T*sigma_u²/(sigma_e²+T*sigma_u²)]*mean(y-xb)_i and
idiosyncratic ehat=y-xb-uhat. The shrink factor is 1-(1-theta)², not theta.
Validate historical Stata8 meaning separately against its logged matrix; no claim
of a new native RE `u` prediction or general retained conditional RE state.

**Sample/failures:** 3..532 complete ten-year people, exact unique bounded
source_row/id/year, finite model y/x in [-20,20], identified within/between
wage designs, positive finite within variance, finite theta in[0,1), positive
cross-person idiosyncratic residual variance each year and100 finite correlations.
No missing/unbalanced sample, weights, extra regressors or imputation. Negative
raw unit variance is truncated to zero and theta0/shrink0 are valid. Preflight
before export; two exports are not a whole-session transaction, never overwrite
source/existing destinations. Retain native RE state for `lincom` only if RE
supports it; otherwise do not pretend a SQL-created model owns RE inference.

**Acceptance before implementation (risk high):**
1. Independent R4.6.2/plm Swamy–Arora fit and `ranef` checks full RE coefficient/
covariance/inference/components, 5,320 exact-source/prediction rows including
population residual, random effects and idiosyncratic e, all100 year cells;
fixed abs1e-9, p-relative1e-8/exact underflow0 and exact raw/keys. Per-person
uhat and algebra identities checked. Compare all45 historical log values to
printed four decimals separately; investigate discrepancies, never widen R tolerance.
2. Independent interleaved16-person positive-component and theta0 controls check
sample/order/design, all correlations, both CLI modes and original RE prediction
ownership; explicitly distinguish shrink from theta and e from combined residual.
3. Duplicate/fractional/missing/incomplete input, rank/component or zero-year
variance failure before export; verify both export-owner collisions, preserved
source and partial first output when second destination exists.
4. Document reproducible references and limits; update inventory, roadmap, SPEC
and ledger only on evidence. No FD/level/individual or full chapter/book closure.

Use existing native RE/SQL/predict/export, no production API/dependencies.
Start focused script tests, then root suite, final source/diff review and CI.
If source prediction semantics or implementation contradict this contract, stop
and report rather than substituting a population residual or changing tolerances.
