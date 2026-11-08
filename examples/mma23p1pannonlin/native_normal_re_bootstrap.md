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

WIP: no bootstrap parser/kernel/result or source-sized feasibility evidence yet.
Normal bootstrap/historical issues and the complete roadmap remain required.
