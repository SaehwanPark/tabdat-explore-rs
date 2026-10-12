# Classical NB2 feasibility contract

In progress on the next Phase 1 loop, base `ba24111`. Reuse the verified shared
count plugin, no new native module or runtime dependency. Classical NB2 log-mean
model with integer counts, finite complete ordered sample, intercept + two
predictors; variance mu + alpha*mu^2, alpha > 0. Owned full coefficient/SE/VCV,
likelihood, exact sample and explicit family. Preserve existing Poisson API.

Predeclared optimizer comparison: abs 1e-7 + rel 1e-6 for all numerical fields;
exact names/order/sample. References: pinned Python TabDat, independent R MASS
NB2 fit plus observed joint information for beta/alpha (not conditional GLM
covariance). No robust/cluster/offset/exposure or product prediction state.

Contract conflict: pinned executor labels statsmodels' final raw alpha parameter
`lnalpha`. Inspect model parameterization and trusted R before accepting that
label. The spike will label variance-scale dispersion `alpha` explicitly, record
the Python naming mismatch and not silently transform values or expected fixtures.
No whole-command parity is inferred. Invalid counts/rank/all-zero output and
Poisson/binary/OLS recovery must remain covered. Existing platform, safety,
packaging and licensing limits remain explicit while remaining slices continue.
