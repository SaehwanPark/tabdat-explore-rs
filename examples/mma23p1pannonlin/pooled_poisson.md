# Chapter 23 patents pooled Poisson — WIP contract

## Exact scope

Table 23.1's pooled Poisson column in §23.3 pp.792–795, from the March 2005
`mma23p1pannonlin.do` and `patr7079.asc`. Original source SHA-256:
`87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8`.
The source contains 346 firms by ten years, 25 values per firm; assign firm ID
by original input record, not CUSIP. Reshape to 1975–79 inclusive, 1,730
firm-years/346 firms. Outcome PAT is a nonnegative patent count; sole predictor
LOGR is log R&D, with intercept and no extra regressors, weights or offset.
Fit pooled Poisson with model-based, independent-row robust and firm-clustered
covariances, current means/linear indices, summaries, and non-overwriting export.
This is an unobserved-effects-ignoring pooled mean model, not FE, RE or GEE.

## Acceptance and independent evidence

1. Exact 346 original firms, 1,730 year-specific outcome/covariate row IDs,
   design, predictions and summaries are independently checked from source;
   text and CLI-v1 JSON script workflows report all three fits and predictions.
2. Independently fit the same float64 design with base-R Poisson and separately
   calculate model-based, row-score and grouped firm-score sandwich covariances
   with the native documented corrections; verify full coefficients/covariance,
   inference and prediction values at justified fixed tolerances. Compare the
   author log's rounded pooled slope .6929337 and model/row/cluster SEs .0022454,
   .0198092 and .0430441, investigating rather than hiding correction or historic
   float-storage differences.
3. Preparation rejects empty/malformed/incomplete/nonfinite source, invalid or
   duplicate derived keys and invalid counts before publication, without clobber;
   synthetic small data exercise ordering and negative/failure cases. Fit,
   export and rerun failures preserve promised dataset and file state.
4. Update SPEC, roadmap and the ledger accurately and run focused plus affected
   workflow/integration checks. R/Python are development-only audit tools; the
   original author data and programs are not bundled.

The author also reports firm FE/RE Poisson, GEE, exponential NLS, whole-firm
bootstraps and a figure. They are required separate work, not implied by this
pooled illustration. Firm clustering assumes independent firms and no
few-cluster guarantee; it does not make the Poisson law valid or identify a
causal R&D effect. No historical bootstrap RNG, entire Table 23.1 or chapter
completion claim.
