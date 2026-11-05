# WIP: individual log-hours lag-one correlation

## Bounded contract before implementation

Reproduce Chapter21 author residual program lines142–156: log-hours current/lag
Pearson correlation across nine consecutive annual pairs **within** people1 and2,
not correlation across532 people. The source reloads its own `mom3` state from the
same program before reshape; verify source provenance and report external-state
limitations honestly. Compose the calculation for all available people (including
1 and2) in one ordered table and export annual source/lag pairs and per-person
correlations, explicitly descriptive, not a formal test, causal inference or
panel covariance correction. The source program does not calculate wage
correlations here; do not claim it does.

Sample:3..532 complete people with unique exact id/year/source-row keys, ten
finite bounded lnhr/lnwg values for1979–1988, no missingness deletion or
bridging. Explicit id-partitioned annual lag yields9 within-person pairs; both
current and lag series need finite population SD>1e-12 for every person. Validate
all resulting coefficients/counts and annual source/lag rows independently against
R reshape/`cor` (abs1e-9), and compare the two source-log printed correlations at
four-decimal precision separately. Include interleaved control with meaningfully
different within-person vs across-person behavior, both CLI modes, invalid keys/
incomplete/missing and zero/resolution variance failures before export, and both
existing-target collisions/file-state invariants. No new native API, R/Python
production dependency or claim of full residual/chapter/book coverage.
