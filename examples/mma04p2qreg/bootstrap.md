# Joint quantile pairs bootstrap (work in progress)

Target: the joint-quantile bootstrap underlying MMA §4.6.4 Figure4.1,
author mma04p2qreg.do block3, on the audited 5,006 complete ASCII records.
Requested quantiles share every IID row resample; the full joint covariance
uses B-1, with marginal percentile95 intervals and the existing explicit
xoshiro256++/SplitMix64 seed contract. This is a read-only report, preserving
data and retained fits on success/failure. No production R/Python dependency.

Acceptance: common draws/full joint covariance against R quantreg; repeatable
seed/sample mapping; native script/text/JSON workflow; explicit failed-replicate
index without redrawing; preserved data/model state. Original and resampled
point fits use the existing certified LP without IID sparsity estimation.
Weights, clusters, simultaneous coverage bands, contrasts/retained bootstrap
fits and rendering Figure4.1 are outside this slice. Further work is required
before claiming the figure or whole illustration complete.
