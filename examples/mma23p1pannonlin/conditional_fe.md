# Chapter 23 conditional fixed-effects Poisson — WIP contract

## Exact illustration and scope

Implement the model-based `xtpoisson PAT LOGR, fe` column of Table23.1 / §23.3
pp.792–795 using a transparent **composed bounded SQL workflow**, not a new
general `xtpoisson` command. Reuse the audited `patr7079.asc` converter and its
346 firms/five 1975–79 years. Log-link `E[PAT_it|x_i,a_i]=exp(a_i+b*LOGR_it)`;
condition on each firm's observed total patent count. Exclude all 22 zero-total
firms (110 rows), leaving 324 firms/1,620 rows; report both counts. One slope,
no time dummies, weights or offset. The original author log prints slope
−0.0377642, model-based SE 0.0325518 and p≈0.246.

Use the scalar conditional log-likelihood and score, bracket b in [−10,10],
and deterministic bisection; stable within-firm centering must bound exponent
arguments and enforce a finite interior score crossing and positive information.
Compute the observed/model-based conditional-information variance and normal
95% interval; normal p-value may be provided through a verified fixed-order
series on an explicitly bounded z domain and reported unavailable outside it.
No covariance substitution, ridge or fabricated fit for separation/no crossing.
Provide normalized fitted patent means and conditional residuals on retained
firms, export all original rows with explicit inclusion flags and NULL for the
22 unidentified zero-total firms. Fitted unit intercepts are nuisance estimates,
not individually inferred effects, and fitted totals equal observed unit totals.
Use a report table plus stable predictions/exports through text, script and
CLI-v1 JSON; no claimed restorable estimation state.

## Acceptance, evidence and failure cases

1. Inspect the original source and show exact 1,730 candidate/1,620 retained
   row identities, group membership, exclusion counts, sample/design, model
   coefficient/SE/z/conditional likelihood, and all predictions. Independently
   fit R `glm(pat ~ logr + factor(id), poisson)` on the exact positive-total
   sample; independently calculate conditional likelihood/score/information,
   normal tail and normalized predictions. Verify precision at fixed justified
   tolerances and selected author-log rounding; investigate differences.
2. Test a small inspectable synthetic balanced panel with positive and zero
   groups against the same independent R dummy-variable fit and conditional
   calculation. Verify row ordering, excluded-group NULLs, and unchanged input.
3. Reject all-zero panels, rank-zero within-group predictors, missing/nonfinite
   or duplicate keys, unsupported numeric magnitudes, nonpositive/nonfinite
   information, unbracketed/boundary/separated fits and unwritable/existing
   exports without silently publishing invalid output. Test recovery and
   failure paths. Validate the numerical tail against R over its supported range.
4. Update SPEC, roadmap/ledger, run focused/affected tests and review the diff.
   R/Python are development-only, original author data/program unbundled.

This model-based conditional Poisson inference assumes the specified
conditional distribution and is not cluster robust; within-firm dependence or
misspecified conditional means can invalidate it. It is not automatic panel
selection, random-effects/GEE, unconditional FE inference, unit-effect
uncertainty, out-of-sample forecasting, whole-firm bootstrap, or full Table23.1.
Do not claim general native conditional-Poisson support. If SQL recursion,
stability or reproducibility cannot meet this contract, stop and reassess rather
than widening tolerances or merging scaffolding.
