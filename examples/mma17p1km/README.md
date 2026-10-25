# Strike duration: bounded nonparametric survival workflow

## Contract (implementation in progress)

Cameron–Trivedi (2005), §17.2 pp.574–575 and §17.5.1 pp.581–583,
`mma17p1km.do`: original Kennan strike data `strkdur.asc`, 566 complete
single-spell durations in days, all failures, entry at zero, no censoring.
The exact source has 24,691 days at risk, times1..235 and141 distinct times.
GDP is preserved but is not a survival covariate.

Deliver a composed native SQL/table/export/SVG workflow, not new `stset`/`sts`
syntax. Risk sets include all events at the tied time, with one post-event
Kaplan–Meier ordinate per distinct time. Greenwood SE and pointwise95%
log-minus-log intervals are undefined at the final all-fail time and exported
as NULL, not invented zero uncertainty. Nelson–Aalen is sum(d/n), not -log(KM),
with independent-event variance sum(d/n²). No Student-t/DF correction.

Acceptance:
1. Preserve original566 source ordinals and both input columns; native output
   reproduces every141-time risk/event/survival/uncertainty/hazard record from
   development-only R `survival::survfit` and rounded author `sts list` values.
2. Produce an interpretable table/CSV and right-continuous step SVG with
   pointwise bands (Figure17.1 equivalent, not WMF/layout parity), plus a native
   Nelson–Aalen step plot.
3. Execute the actual script on an independent tied-time control; terminal/JSON
   exports agree, ties and endpoint undefined uncertainty remain explicit.
4. Reject empty/nonpositive/missing/nonfinite durations before exporting any
   result; existing output files are never replaced and stop subsequent effects.

Exclusions: no censoring/truncation/delayed entry, weights, groups, regression,
Cox baseline equivalence, smoothed instantaneous hazards, simultaneous bands or
whole-chapter claims. General survival declarations/estimators and the separate
artificial censoring example remain required. The program's comment labels the
printed table both17.2 and17.3; use author `sts list` as the exact source locator
rather than silently claiming an unverified book table number.
