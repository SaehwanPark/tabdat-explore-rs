# Native normal-RE whole-firm bootstrap — WIP contract and evidence

Current status: production ordinary-normal stable-increment/no-op refinements
pass affected checks; private warm/refined fitting passes native500-copy source
feasibility. All500 source fits and31 empirical quantities now match independent
R; control500 R references are generated. Public bootstrap stats/parser/result/
report/script/failure/auditor workflow remains pending.

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
production adoption. Proposed production precision/increment changes are not
implemented.

Candidate exact identity to validate, with old mu_i=exp(eta_i),v=exp(t), actual
rounded increments d=(d_beta,dt),d_i=X_i*d_beta, and old group posterior E:

```text
h(u) = sum(y_i*d_i) - exp(u)*sum(mu_i*expm1(d_i))
       - dt/2 - u²/(2*v)*expm1(-dt)
A = E[h] = score_group'd - E[exp(u)]*sum(mu_i*(expm1(d_i)-d_i))
           - E[u²]/(2*v)*(expm1(-dt)+dt)
R = E[expm1(h)-h]
log E[exp(h)] = A + R + (log1p(A+R)-(A+R))
```

The two parenthesized expm1-minus-linear/log1p-minus-linear functions require
stable series near zero, not cancellation-prone direct subtraction. Sum the
linear part via the existing global Stein score; do not replace an actual
likelihood increment with a quadratic prediction. A useful independent
closed-form control is h=a*Z+b*Z²+c under Z~N(0,1): the exact log ratio is
c-log1p(-2*b)/2+a²/(2*(1-2*b)),b<.5. Standard-library decimal arithmetic can
provide tiny positive/negative/near-cancelled controls without a new dependency.
The [isolated arithmetic controls](../../crates/tabdat-stats/tests/fixtures/normal_poisson/increment/README.md)
now implement these remainder/cumulant helpers **under cfg(test) only**, not in
the fitter.44 independent400-digit Decimal remainder cases at actual binary64
inputs and12 Gaussian quadratic closed-form cases (including positive/negative
~1e-24 changes and near cancellation) pass. Invalid inputs reject without
clipping/renormalization. Focused3/3 tests pass (74c0e82a-0cc1-4adf-8bf7-e8538a0dd729);
stats all-target Clippy passes. This first stage validates arithmetic, not the
Poisson posterior/Stein construction or source-copy optimizer. No full-crate test rerun
or production numerical-contract change is claimed at this checkpoint.

The next cfg(test)-only group helper now also matches18 independent R direct
improper-integral directional-score/log-ratio controls across singleton/3/5-row,
zero-total/100000-count/tiny-mean shapes and both shift signs down to1e-12.
R integrates complete-data scores, not the native Stein identity, and forms the
ratio inside the integral, not by subtracting log-likelihood levels. Fixed
relative1e-9 plus1e-24 tests preserve tiny signs. New test1/1 passes separately
(b0a27f39-1362-4429-a48d-92dfb52f0275), not a fresh combined4-test run; updated
stats all-target Clippy passes. Old96-node posterior moments and ratio points
are deliberately matched here. Any refined fitter path must parameterize both
together before using this helper. No production no-op/optimizer fix or
copied-source replay is yet claimed.

## Integration feasibility / required initializer revision

Stable small-increment comparison and rounded no-op rejection are now being
integrated into the ordinary optimizer; the statistical gates remain unchanged.
Matched128/160 profile/moments/ratio points are private/test-only. Focused17
normal tests pass including all42 independent scalar-control fields/full
covariance/predictions under both profiles, independent Gaussian moments for
all four rules, and18 direct-R group ratios under both profiles. This is not a
full-crate/workspace or bootstrap-command claim.

The source feasibility replay now fits copies1..6, including copy2 at its R
solution, but stops at copy7. Temporary tracing (removed) places this at
**iteration1**, not at the solution: pooled beta=1.9670169781410587/normalized
slope1.473252951855466,t=0 has indefinite joint observed information despite
positive diagonal entries. Independent direct-R copy7 MLE is1.2513903274375/
.379131218523013,t=.944140581212082,LL=-5249.55818325462,24 zero-total copies;
maximum raw score9.66e-11/Richardson covariance discrepancy1.49e-11 meet fixed
gates. Thus neither ridge/absolute eigenvalues nor accepting indefinite
covariance is appropriate.

The proposed copied-refit initializer is now explicitly **the original normal
joint estimate in raw coefficient units**, converted to each copy's RMS units.
Use that one deterministic warm start for every replicate, not retry-after-error,
original pooled coefficients, prior application state or R coefficients. Refit
the full-X same-copy pooled null independently as before. Ordinary normal
commands keep their existing pooled/t0 initializer. Fail if the original or any
warm-started copied fit is unsupported; do not alter the undamped-SPD, ascent,
iteration, step or score requirements. This is a demonstrated initialization
contract revision, not a validated all500 outcome or a new public option.

## Native feasibility achieved (subsequently independently validated below)

The ordinary optimizer now uses the exact posterior-ratio construction for
max(abs(delta_eta))<=1e-4 and abs(delta_t)<=1e-4. This is a calculation crossover,
not a new convergence tolerance: large steps retain marginal-level comparison;
small steps require computed change>=0 with no epsilon allowance. Actual rounded
parameter deltas are used; bit-identical trials cannot be accepted. Both full
frames still pass the same posterior/design/bounds checks; undamped SPD and
score1e-10/relative step1e-12 are unchanged. Tests cover both directions,
no-progress rejection and the adjacent binary64 values across the crossover.

With matched128/160 nodes and the original normal raw joint point as the sole
copied initializer, a removed development probe now fits **all500** source
copies/173000 reused independently verified native-stream indices in58.404s
(debug). No copy retry/redraw/skipping. t ranges.1801618898518454..1.9326944936544626;
max final per-group score7.253e-13, relative undamped step9.477e-13 and final
paired discrepancy8.596e-11. Copies2/7 match their independent direct-R points
within4.93e-14/1.47e-13. This is native feasibility plus two independently checked
copies, **not independent validation of all500 fits/uncertainty**. The public
bootstrap command/result/state/report workflow is not implemented yet.

After implementation, fresh stats all-target327/58 binaries/one pre-existing
skip pass (805675d3-0a2e-42ca-a7bc-7074711b8aef,13.048s); reverse normal runtime3
(444b5084-9164-4cb6-bd1e-1845eef141a1) and root source workflow3
(98c82deb-7b77-464d-abe3-e813af27e823) pass. Updated focused normal18 tests include
raw-initializer RMS conversion/opposite1e3/1e-3 units/full covariance, malformed
initializer rejection and both-profile independent controls. Subsequent test-only
expansion checks all eight direct-R posterior moments in all six stress cases
under both profiles (modified1/1,128b9985-a1f1-4f24-98aa-b44812ad71f0), not a
second327-test run; production code was unchanged. Stats all-target
and scoped root bin/new-normal-test Clippy pass; initial root Clippy used the
wrong bin name `tabdat`, corrected to `tabdat-explore-rs`, not a code defect.
No new workspace/CI/doctest/platform/release claim. No public bootstrap or
merge readiness. Next independently refit all500 from raw copies/draws, then
complete stats/application/script/report/failure/auditor contracts.

## Independent reference driver (development only)

`native_normal_re_bootstrap_reference.py RAW_ASC DRAWS_CSV NEW_OUTPUT_DIR`
verifies the complete500-replicate seed10001 stream using the existing separate
Python RNG oracle, freezes the R model/summary sources with hashes, and runs
independent R point fits from raw distinct-copy records (not native coefficients
or warm starts). Default4 workers use independent R processes; a failed fit
aborts/cancels queued work and never publishes a partial fits.csv/reference.csv
or complete provenance. Existing output directories are never overwritten.
`--control` explicitly marks an unhashed artificial source; `--reps 2` is only
a prefix smoke, not source500 evidence. All raw inputs/logs remain for diagnosis.

The R `bootstrap-point` mode uses the existing improper-integration/BFGS/Newton
model. BFGS omits unused information integrals; full observed information is
still required positive at every Newton/final point and raw score<1e-8. Final
per-copy covariance/Richardson exports are skipped because bootstrap uncertainty
uses empirical joint refits, not those Wald matrices. The legacy two-argument
full42-field control output is byte-identical; copy2 point mode before/after
skipping unused BFGS integrals is also byte-identical. Independent base-R
`cov`/type7 `quantile` summaries export31 quantities, original marginal means
and every fitted triple/likelihood/raw score.

Reference-driver4/4 RNG-tampering/no-clobber/failed-refit/failed-summary controls
pass. Actual two-copy source smoke passes with692 verified draws and independent
R summaries (6.443s); means/covariance/type7 identities pass. These mocks/smoke
are not themselves all500 statistical validation. Subsequent full runs follow.

## Full independent reference series achieved

Source500 direct-R refits/173000 verified draws complete in425.550s; artificial
40-firm control500/20000 draws complete in60.361s. Each R copy starts from its
own full-X pooled GLM/t0 (not native/original normal warm coefficients), uses
improper integration/complete-data scores/information, and requires final SPD
and raw score<1e-8. Source max raw R score2.147e-10. Frozen scripts/hashes/logs,
point triples/likelihoods/scores and independent base-R31-field summaries are
retained in `reference-source500`/`reference-control500`.

All500 prototype native source triples match R at fixed1e-9+1e-10*abs(ref):
max deltas4.899e-12/1.191e-12/2.306e-12, max loglikelihood delta7.276e-11.
All31 original point/marginal-mean and empirical mean/bias/full covariance/SE/
type7 quantities match the same fixed scalar gates. Native joint means are
1.42736691757247/.2840890158976975/.9916756742094646; bootstrap SEs
.18739135487060413/.1339801535962461/.3055342450484391. R type7 intervals are
[1.0827199678624204,1.8259472687076947],
[.0028317537800471621,.53613928159713375] and
[.3549663380036438,1.5789470374470538]. They concern intercept/slope/log variance,
not a no-heterogeneity test or invented natural-sigma percentile interval.

This validates the prototype source numerical series/empirical construction,
not an implemented public bootstrap report/workflow. Native control500 replay,
permanent kernel/API tests, raw-equation/forgery auditing, read-only state and
file/CLI/script/report contracts remain. No new Cargo/workspace/CI claim from
these development-oracle runs; all native/R counts above are separate evidence.

Artifacts `/tmp/tabdat-ch23-native-normal-bootstrap/`: exact replicate002.asc,
independent replicate002-reference/reference.csv/R log, original feasibility
and temporary128/160 feasibility JSON, both optimizer traces and R node/probe
generators. No raw source or transient example/trace is committed. The original diagnostic
edits were restored before implementation. Subsequent numerical integration now
uses the validated Rust increment helper in production ordinary fits; the
128/160 profile/raw warm-start machinery remains private/test-only. The public
normal API, model/inference/sample/prediction semantics,80/96 rules,176 work
factor and all statistical convergence/SPD/quadrature gates remain unchanged.

WIP: ordinary stable numerical comparisons, private refined/warm copied-fit
feasibility and independent controls; no production bootstrap parser/kernel/
result or independent all500 uncertainty/workflow validation yet. Normal bootstrap/historical issues and the
complete roadmap remain required.
