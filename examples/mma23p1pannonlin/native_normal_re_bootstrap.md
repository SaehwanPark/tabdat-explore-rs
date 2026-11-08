# Native normal-RE whole-firm bootstrap — preimplementation contract

## Bounded outcome / source status

Extend the required normal random-intercept Poisson workflow with a read-only
whole-firm pairs bootstrap report and reproducible text/CLI-v1/export workflow.
The target is the native §23.3 patents adaptation:1730 rows/346 firms/all22
zero-total firms. **The publisher companion program does not request a normal-RE
bootstrap.** This is a required native resampling extension, not a sixth source
bootstrap block or historical RNG/interval/SE parity. The five actual source
bootstrap blocks remain as inventoried.

Proposed selector: `bootstrap pat logr, model(normal_poisson_re) cluster(id) reps(500) seed(10001)`. Reuse existing bootstrap summaries/RNG/draw helper and
existing validated normal fitter. No dependency, FFI, production R/Python,
bootstrap framework or change to other estimator/default/wire contracts.

## Sample, refits and inference

Use the exact normal joint-finite sample after validating every raw typed key,
including omitted y/X rows. Retain all selected zero-total/singleton groups.
Each replicate draws G whole groups with replacement in first-seen selected
group order, using one continuous native xoshiro256++ stream. Every repeated copy receives a new
independent shared Gaussian log effect: never merge repeated original IDs.
Refit intercept/slopes/t=ln(variance) jointly using the same normal fitting
contract; pooled full-X initialization/null remain independently refitted.

Report joint coefficient vectors in original units/order `_cons`,slopes,
`lnsigma2`; all B fits/draws/row counts, empirical means/bias, B-1 covariance/
SE and type7 marginal percentile95 intervals. Original normal inference is not
replaced by bootstrap covariance; do not invent bootstrap p-values, simultaneous
coverage or sigma-scale intervals from log-scale SEs. Successful or failed
resampling preserves prior estimation/data/labels/stored reports; original
normal fit in the example remains the source of subsequent n/xb/residuals.
Any failed original/copied fit aborts with its replicate number, no redraw,
dropped failures or partial-success report.

Bounds: intercept+1..14 slopes,2..500 replicates,<=512 selected groups, existing
normal row/group/count/quad/design limits. Worst copied N=G*largest selected
group size must be<=40000 and satisfy normal176*N*(slopes+2)^2<=20m; conservative
aggregate176*worstN*B*(slopes+2)^2<=3b. No quadrature threshold change or fit
fallback is planned. A feasibility probe must check all500 original-source
replicates before public integration. If existing strict80/96 GH fails on a
valid copy, preserve the failure and revise the numerical contract explicitly
with independent evidence, never widen tolerances to force completion.

## Acceptance cases / independent reference

1. Original1730/346/22-zero source: B500/seed10001,173000 independently regenerated
 draws, all500 joint intercept/slope/t R refits, full3x3 empirical covariance/
 SE/bias/type7 intervals. Exact source/sample and original n/xb/residuals remain;
 JSON/text exports match. Raw empirical identities and integrated scores must
 reject coherent wrong native/R artifacts, not agreement alone.
2. Frozen40-firm/200-record source control plus bounded multivariate/unbalanced/
 singleton copy control: independent direct-R improper integration, actual copy
 identities, exact first-seen draw order, no implicit zero-group filtering or
 data/weight mutation. Independent expanded-copy versus merged-ID sensitivity
 demonstrates repeated copies are distinct effects.
3. Additive typed parser/result/report metadata with native RNG/interval/failure
 policy; previous owned normal fit/data/labels/stored reports survive success,
 export collision and unsupported commands. No report silently becomes a fit.
4. Replicate singular/boundary/numerical failure and work/group/rep-count errors
 preserve state/files and identify fail-abort behavior. Regression coverage for
 gamma/FE/pooled resampling remains; no parser-only/kernel-only merge.

R4.6.2 direct adaptive improper integration (not GH), raw data and independently
verified draws only; never native coefficients as inputs. Retain established
scalar1e-9+1e-10*abs(reference), positive-p relative1e-9 where applicable. Record
scores/information, numerical versions/tolerances and all unsupported behavior.

## Feasibility finding (before bootstrap implementation)

Original native seed10001 draw indices were reused from the independently
verified gamma-bootstrap draw artifact, not its coefficients. Expanding each
copy under a separate zero-padded key gives1730 rows/346 distinct effects.
The strict ordinary80/96 fit succeeds on replicate1 and aborts on replicate2.
Temporary optimizer tracing (removed, no production change) shows repeated
quadrature rejection, not a near-stationary roundoff issue: at the supported
edge t~1.486949, per-group score~.006865 and relative step~.012 remain far from
convergence; the32-trial error is the terminal symptom.

Independent direct-R improper integration on that exact copied sample (220
unique original firms,26 zero-total copies) converges to beta=1.61053501230409/
.0921644210366025,t=1.5069345894251,LL=-5755.5749503683. Raw maximum score
3.16e-11 and Richardson covariance discrepancy1.19e-11 pass the unchanged
reference gates. Therefore accepting the coarse numerical edge, loosening the
1e-9 discrepancy, convergence gates, dropping/redrawing the copy or relabeling
it as successful is not allowed.

Next evaluate higher-order paired rules with the same1e-9 moment-unit check.
A candidate explicit128/160-node **copied-refit** profile would cost288 nodes
and fit the source aggregate bound (288*1730*9*500=2242080000<3b), while ordinary
normal80/96 commands/outputs/bounds remain unchanged. This is a proposed
numerical-contract revision for a demonstrated need, not an implemented or
validated precision mode. It must pass all500 source fits and independent R/
posterior controls before adopting; no public option/framework is proposed.
A temporary128/160 prototype then gets past the coarse quadrature barrier on
replicate2 and reaches beta~1.610535012309/t~1.506934589423, matching the R point
under the existing fixed oracle. It still **fails**, now at the100-iteration
limit: global likelihood levels near-5755.574950368265 cannot resolve the last
Newton improvement. Per-group score~9.58e-13 passes, but relative undamped
step~1.7691e-12 does not pass the unchanged1e-12 gate. Line search accepts a
1.52588e-5 fraction whose rounded parameter vector is bit-identical, repeating
until the limit. Neither a higher iteration cap nor widening the step gate is
an acceptable fix.

Next numerical investigation: reject no-progress rounded trials and evaluate
small likelihood increments by an independently checked posterior-ratio
identity (stable expm1/log1p with centered cumulants/Stein first moment), instead
of subtracting large rounded likelihood levels. This must preserve genuinely
nondecreasing likelihood, positive undamped information, all score/step/quad
gates and explicit failure on unsupported inputs; it is not permission to
accept declines within an epsilon or substitute a damped-step convergence test.
Both ordinary and refined numerical paths need focused positive/negative/tiny
increment controls and unchanged direct-R regression comparisons before any
production adoption. Proposed precision/increment changes are not implemented.

Artifacts `/tmp/tabdat-ch23-native-normal-bootstrap/`: exact replicate002.asc,
independent replicate002-reference/reference.csv/R log, original feasibility
and temporary128/160 feasibility JSON, both optimizer traces and R node/probe
generators. No raw source or transient example/trace is committed. Production
normal_poisson.rs is restored byte-identically to merged7fbeb1e.

WIP: only contract and two diagnosed numerical barriers; no bootstrap
parser/kernel/result or all500 feasibility claim. Normal bootstrap/historical issues and the
complete roadmap remain required.
