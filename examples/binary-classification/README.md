# Binary classification after logit and probit

## Planned contract

Add `estat classification[, cutoff(#)]` after a successful logit or probit fit.
Use the outcome and predicted probabilities retained for the exact estimation
sample. A row is predicted positive when its probability is greater than or
equal to the cutpoint; the default is 0.5. The event is `y == 1`, matching the
installed binary fit's exact 0/1 outcome domain.

Report the four confusion-table counts (true positive, false positive, false
negative and true negative), sensitivity, specificity, positive and negative
predictive values, the four false-classification rates, and the overall correct
classification percentage. Percent metrics use 0–100 units. When a metric's
denominator is zero, return it as unavailable while preserving all defined
counts and metrics. Classification is descriptive and has no standard error or
hypothesis test.

This slice uses the retained estimation sample only. Current-data edits do not
change its sample or predictions. `all`, row qualifiers, weights, ROC curves
and cutpoint selection are outside scope; unsupported forms must fail clearly
without changing the fit or data. Cutpoints must be finite and lie in [0, 1].

The classification rule and reported statistics follow Stata's Base Reference
Manual entry for
[`estat classification`](https://www.stata.com/manuals/restatclassification.pdf),
adapted to TabDat's exact binary outcome and retained-sample contracts. A
Base R reference over the existing 240-row mixed-factor fixture will validate
both links, cutpoints at 0.5 and a nondefault value, confusion counts and all
defined ratios. Runtime and JSON/text CLI contracts will cover edits, boundary
cutpoints, zero denominators, invalid options and no-mutation failures. This is
synthetic method evidence, not a new Cameron–Trivedi illustration.
