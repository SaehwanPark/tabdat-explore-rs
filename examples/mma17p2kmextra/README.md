# Artificial right-censoring: bounded survival workflow

## Contract (implementation in progress)

Cameron–Trivedi (2005), §17.5.1 pp.581–582, author March2005
`mma17p2kmextra.do`, Table17.2 KM/NA portions. Extract its exact explicit input:
80 single-record spells, entry zero,40 failures/40 right-censored exits,3170 days
at risk,9 distinct exit times10..50. `failed=1` means failure. Comments incorrectly
call the32 exits at45 failures and the26 exits at50 censored; the executable
input and printed results unambiguously specify the opposite. Follow the executed
sample, not those comments. No replacement random generation.

Acceptance:
1. Preserve all80 source ordinals/duration/event cells; match independent R
   survival and both author printed KM/NA tables, including uncertainty.
2. Native table/CSV and right-continuous step SVGs carry estimates across
   censor-only times; at tied event/censor times the risk set includes both before
   failures, then removes both. Censoring is not treated as a failure.
3. The actual script also matches an independent control with an initial censor,
   a failure/censor tie and a censored tail; KM can end above zero. A terminal
   all-failure risk set yields KM0/undefined KM SE/CI, not invented certainty.
4. Reject empty/missing/nonfinite/nonpositive times, nonbinary/missing event flags,
   duplicate/invalid source keys and zero observed failures before any export.
   Source conversion is atomic; output collisions preserve existing files and
   stop later script effects, without pretending all files form one transaction.

Estimator: KM product(1-d/n), Greenwood sum d/[n(n-d)], pointwise95% log-log
intervals; NA sum d/n with variance sum d/n² and positive log-Wald95% intervals.
Before any observed failure use S1/SE0/bounds1 and H0/SE0/bounds0. No DF correction.
Interpretation assumes independent noninformative right-censoring and independent
spells; it is not a causal guarantee or proof of survival beyond observation.

Exclusions: no delayed entry/left or interval censoring/truncation, weights,
groups, regression/Cox/parametric models, simultaneous bands, retained fit state,
new `stset`/`sts` syntax, or full survival-family/book coverage. Native SVG layout
is not Stata WMF parity. This is an author artificial-data illustration, not an
empirical dataset or a substitute for required empirical survival regressions.
