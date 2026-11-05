# Chapter 27 missing-data baseline comparison — WIP contract

## Outcome

Implement the author-program **complete-case and single mean-imputation** linear
comparison for the four published generated data sets from §27.8.1 pp.936–937,
`mma27p1milinear.do`, columns2–3 of the illustrated regressions. These are
synthetic simulations, not empirical subjects. The author source says its seed
and generated data differ from the book's tables; its output is not literal book
reproduction. Its case4 overview says35% missing, but the executed code uses25%;
check the actual pinned records instead of silently choosing the prose.

The published input provides y and two possibly missing x values, not the true
complete x or simulation errors; column1 cannot be reconstructed from these
files. The author SAS companion separately runs a normal MCMC multiple
imputation (10 imputations,500 burn-in,1000 between imputations) and
`MIANALYZE`; it is **not** delivered by mean imputation. Multiple-imputation
models, pooling, diagnostics, valid MI uncertainty, logit and full chapter/book
coverage remain required. The naïve classical uncertainty following single mean
imputation treats filled means as fixed and does not reflect missing-value
uncertainty; do not claim it is valid MI, causal inference or generally unbiased.

## Sample, design and reference

Download the authors' data files from
<https://cameron.econ.ucdavis.edu/mmabook/mmaprograms.html>, preserving physical
row order and `.` missing values. Case1 `mma27linear1.asc` SHA256
`a2a58a2548096fff062d9c7ff3cb820785a39c70f55a3e34abe1eb5c256d7283` is the
primary target:1000 rows,100 missing in each x,812 complete cases. Cases2–4
are additional independent original-data controls. No source files are
redistributed. Preserve source rows/values and missingness; y must be present,
x may be missing. Model `y ~ 1 + x1 + x2` by nonrobust OLS in each sample. For
mean imputation calculate **each** predictor's mean from all its observed rows,
then fit all rows with fixed filled values; do not compute means from only the
joint-complete subset. Report both classical residual degrees of freedom and
Student-t inference, explain assumptions, give per-row missing indicators,
filled predictors, fit/residuals and no-silent-sample-change diagnostics.
Retained model/state and ordered no-clobber export effects must be explicit.

Independent base-R QR `lm`/`vcov`/`predict` on the exact rounded published input
is the numerical reference. Author-log comparisons must account for the program
having used unrounded internal data before ASCII export; investigate differences,
do not widen fixed native/R gates. Use absolute1e-9, relative1e-8 for nonzero
p-values and exact zero for underflow unless a more stringent method-specific
bound is justified. Python/R are development-only.

## Acceptance and failure contract

1. Prepare original1 in a fresh directory; both actual CLI fits, samples, all
   reported coefficient/inference fields, each missingness/mean diagnostic and
   every exported source/prediction field agree with independent R. Validate
   original2–4 independently as additional cases and compare the printed
   source log at the precision actually supported by the published input.
2. JSON and text execution expose both fits and the caveat; exported data are
   byte-identical, ordered and preserve NULL/source values. Verify retained fit
   predictions after the full script and enumerate file effects.
3. Small inspectable controls exercise unequal missing patterns, a fully observed
   case, predictor-specific mean calculation and an independent reference.
4. Malformed or nonfinite source, missing y, an all-missing predictor,
   insufficient complete cases or rank-deficient design fail explicitly without
   corrupting original data; converter and export collisions never clobber an
   existing owner. Test later effects and retained state rather than assume a
   whole-script transaction.
5. Source-bound checker rejects forged fit, source, filling and prediction fields;
   identify any full-source or historical-output limitations explicitly.

This is an implementation contract, not a claim of finished code or validation.
