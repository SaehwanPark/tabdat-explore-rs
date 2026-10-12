# Classical NB2 feasibility contract

Bounded NB2 target validated on PR #171, base `ba24111`. Reuse the verified shared
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

## Results / deviation decision

[ADR 0013](../adr/0013-statistical-oracle-normalization.md) resolves the naming and
optimizer-target decision. Original executor numbers are retained in
[nb2-python-default.csv](../../spikes/gretl-prototype/fixtures/nb2-python-default.csv).
Its maximum score is 0.00135934 despite convergence; initial strict comparison
failed (native intercept 0.2756555943 versus default Python 0.2756589422).
No fixture or tolerance was silently changed. The separately labeled external
Newton refinement uses the same model/data, tol 1e-12, maxiter 100 and score <1e-7.

Native coefficients: [0.27565559432667025, 0.30596280923901015,
-0.23982767417488643, 0.7908779434233104]. Final alpha is positive raw dispersion;
log likelihood -506.3544420354368. All four coefficients/SEs, all 16 observed
covariance entries and likelihood pass unchanged abs 1e-7 + rel 1e-6 against
refined Python and independent R. Max differences (parameter / SE / VCV / LNL):
Python refinement 3.06e-9 / 5.71e-10 / 1.57e-10 / 0;
R 1.52e-8 / 2.83e-9 / 7.78e-10 / 4.55e-13.
Exact source rows 0..319 and intercept/x1/x2/alpha order pass.

Data SHA-256 `aab6a1564b281d345a972ccc013b94dbcb10541a4841f9349162fabe68e06a90`;
NumPy seed 1267, n=320, true alpha .65. Pinned oracle revision/tree/lock and
macOS/Rust/Python/NumPy/pandas/statsmodels versions are the same as the
[Poisson report](poisson-plugin.md). R 4.3.2, MASS 7.3.60.0.1. R `glm.nb` uses
epsilon 1e-12 / maxit 100; independent analytic observed joint information is
cross-checked against optimHess (max abs 1e-3). No conditional covariance used.

Two added tests cover the full nominal reference and invalid/rank/family recovery;
feature suite now 19 pass, one ordinary ignored manual loader probe. Shared
validation/result copying is factored by a closed family selector, preserving
Poisson API via a result alias with explicit family. Existing Poisson/binary/OLS,
thread and covariance tests pass. No new unsafe call, module, Cargo dependency,
source patch or native license is added; the renamed checked count FFI uses the
same serialized/process-owned boundary and source-matched module (ADR 0012).
Mac locked fmt/check/test/Clippy, audit/deny and default tests pass. Fresh Linux
x86_64 emulated source build and all 19 feature tests also pass;
[platform evidence](linux-revalidation.md). No pixel/performance or distribution
parity is inferred from Docker.
Three owner review passes cover parameterization/covariance, shared boundary/
error recovery and scope/policy. No independent-review claim.
