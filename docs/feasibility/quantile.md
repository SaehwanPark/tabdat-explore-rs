# Quantile regression feasibility contract (in progress)

Base 8e25c80. Finite complete full-rank intercept/two-predictor fixture, three
quantiles .25/.5/.75 and exact sample. Native quantreg plugin supplies mature LP
coefficients and separately labeled native iid covariance. Normalize canonical
iid covariance to pinned statsmodels defaults: Epanechnikov residual density,
Hall-Sheather alpha .05 bandwidth, population outcome SD and linearly interpolated
residual IQR/1.34 scale; q(1-q)/density² times inverse X'X. Preserve native covariance
as explicitly labeled feasibility diagnostic, not Python covariance parity.

Predeclare abs 1e-8 + rel 1e-7 across coefficients/SE/full covariance, density,
bandwidth and check-loss pseudo-R². Compare pinned executor, external SciPy HiGHS LP
with independently evaluated canonical covariance, and R quantreg coefficients /
independent R QR and density algebra. Preserve original IRLS/covariance output and
record any solver stopping mismatch rather than widen tolerances.

Independent quantile-plugin selects only same-source quantreg.so against verified
shared core, bounded private path/serialized RAII. The additional translated rq
code carries the University of Illinois/NCSA license; native binary/source notices
remain distribution obligations. No robust/cluster, alternate kernels, general
missing samples, post-estimation/model state or production command acceptance.
