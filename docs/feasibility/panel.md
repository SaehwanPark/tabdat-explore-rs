# Balanced classical panel FE/RE contract

In progress, base 62b0bd5. Balanced entity-major/time-minor finite complete fixture,
explicit canonical row keys and exact sample, two predictors. Classical FE
within slopes (entity effects absorb the intercept) and RE Swamy-Arora with an
explicit intercept. Gretl requires a constant; FE drops its native normalized
average-intercept result from the owned slope result. For RE, Python is probed
with an explicit constant column; do not claim Python's default no-constant RE
command parity. Full exposed covariance/SEs compare with pinned linearmodels and
independent R plm. Predeclared abs 1e-9 + rel 1e-8; exact keys/labels/sample.

No unbalanced panel, arbitrary gaps/keys, robust/cluster/time effects, Hausman,
prediction/session state or dynamic-panel acceptance. Native dataset metadata is
set/restored inside one serialized call; raw pointers and MODEL stay private.
Malformed keys/data, constant/collinear predictors, invalid periods and subsequent
successful models need coverage. No new native module or dependency.
