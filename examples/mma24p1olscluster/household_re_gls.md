# Chapter 24 household commune RE-GLS — in progress

Target a bounded §24.7/Table 24.4 unweighted household random-effects GLS
comparison, using the 5,006 complete cases in 194 communes and six numerical
predictors from the [pooled workflow](README.md). The groups are unbalanced,
including a singleton; `source_row` is an artificial unique key and **not**
time. The native Swamy–Arora unbalanced trace-corrected components and
unit-specific quasi-demeaning must be defined and independently checked; the
author's Stata-8 `xtreg ..., re` component/correction conventions might not
coincide. Treat historical differences as separate investigation, not grounds
to widen canonical tolerances or claim unexplained author/table parity.

Acceptance contract: reuse pinned source/preparation; report sample/group
counts, variance components/theta, model-based RE coefficient/full covariance
and normal inference, population xb/residual predictions and state, with an
independent R/plm or direct trace/QR reference on all source/cohort rows. Check
an unbalanced synthetic case with singleton and a zero-unit-variance control,
invalid/missing keys, rank/identification and output-owner collision; compare
actual text/JSON exports. Explain normalization and finite-sample/inference
conventions. This is not a unit-effect predictor, a causal claim, full survey
design, general GEE, ML or full Table 24.4/chapter completion.

**Status:** contract only; executable workflow and independent evidence remain.
