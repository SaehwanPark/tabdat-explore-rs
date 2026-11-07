# Household pooled-residual Breusch–Pagan random-effect LM

## Bounded contract

Deliver H-BP in the [Chapter24 companion inventory](inventory.md), source
`mma24p1olscluster.do` lines118–121: the original household seven-parameter
numeric design on 5,006 complete cases/194 communes, with pooled OLS residuals.
This is an unweighted cross-sectional commune diagnostic, not a time-series,
causal, cluster-robust or complex-survey test. It is not the ML variance LR.

Let N be the selected row count, n_g the group sizes, e the pooled OLS
residuals, R=sum(e²), Q=sum_g(sum_g e)² and M=sum_g n_g². Report signed
score z=N*(Q/R−1)/sqrt(2*(M−N)), LM=z² and the **legacy two-sided squared-score
chi-square(1)** upper-tail probability 2*Phi(−abs(z)). This is the source log's
chi2(1) convention, independently implemented by R/plm 2.6.7 `plmtest` with
`effect="individual", type="bp"`; it is not a one-sided boundary mixture.
A negative z is retained and squared, not clipped or relabelled evidence for
positive unit variance. Null calibration assumes the classical homoskedastic
independent Gaussian idiosyncratic-error model; asymptotic inference, not an
exact finite-sample guarantee. No estimated RE variance is needed for this test.

Acceptance cases:

1. Original ASCII and precise-DTA data: source cohort/order, all predictions,
   residual aggregates, signed score/LM/p against pooled R QR and plm; DTA LM
   matches the author's 432.75 to half a printed digit without widening tolerance.
2. Balanced and unbalanced/singleton controls agree with independently generated
   R QR/plm; balanced formula reduces to N/(2*(T−1)) times (Q/R−1)²,
   where N is the total observation count (not the number of groups).
3. Negative signed-score control preserves sign and uses the documented squared
   reference, not a silent positive-part boundary test.
4. JSON/text exports agree; predictions and reporting retain pooled OLS.
5. Invalid raw keys/missing predictors, no within-group pairs, singular or nearly
   perfect fit fail; export collisions preserve existing files/model ownership.

Scripts are ordered and nontransactional. Reject source rows beyond40,000,
selected groups beyond10,000, fewer than two selected groups/no pair, nonfinite
aggregates and residual RSS <=1e-20 times centered outcome sum of squares.
These are explicit bounded-work/numerical guards, not general estimator limits.
No Hausman test, native generic `estat` API, one-sided LM or whole-table parity
is part of this slice. Implementation and evidence are in progress.
