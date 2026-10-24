# Heckman two-step source/reference — WIP

This is the next **unimplemented native** slice, not a delivered command or
published coefficient-table replication. Exact author analysis:
`mma16p1tobit.do` (March2005) §2A, associated with MMA §16.2.1 pp530–531.
The same original export has200 selection observations and130 observed outcomes:
selection `dy ~ lnwage`, outcome `ytrunc ~ lnwage`, intercept in both.
Use the existing [source assets/converter](README.md); no replacement RNG/data.
The separate later RAND selection illustration remains required.

The author explicitly warns that manual second-stage OLS SEs are incorrect.
This reference uses probit observed ML covariance and the conventional model-based
Heckman/Greene correction for generated Mills regressors, not ordinary/HC OLS.
Let t=Z gamma, lambda=phi(t)/Phi(t), delta=lambda*(lambda+t),
W=[X,lambda], alpha=the Mills coefficient and A=W'W on selected rows. Then:

- sigma²=selected RSS/N_selected + alpha²*mean_selected(delta), rho=alpha/sigma;
- F=W'diag(delta)Z;
- V_outcome=A^-1[W'diag(sigma²-alpha²*delta)W
  +alpha²*F V_probit F']A^-1.

There is no residual-DF multiplier. Normal95 inference is reported separately for
selection coefficients and augmented-outcome coefficients (including alpha).
Sigma/rho are points only; no cross-equation restrictions, joint ML likelihood,
scale/correlation SEs or prediction uncertainty are invented. Native planning
requires regular interior abs(rho)<1, with an explicit failure rather than
Stata-style ad-hoc clipping/covariance adjustments.

## Independent development reference

R4.6.1 `glm`/`lm`, base-R `deriv3` observed-Hessian/Newton refinement, and the
**unmodified external** sampleSelection1.2-16 `heckitVcov` function provide the
reference. This is **not** a claim that the full sampleSelection package is
installed. Its unrelated imports are unnecessary for this standalone covariance
function. It is GPL>=2 development-only source, not copied into the repository,
linked into native production, or a new Rust/runtime dependency.

Pinned CRAN-mirror commit: `5115e01a03d09a46ac1b92813c785c5799b4d654`.
`heckitVcov.R` SHA256:
`7506c7a0d7151f909ecc41d4597bfb11f21dc21222f561729fea79fc7decc769`.
The script additionally checks its exact MD5 via base-R tools before sourcing.

After preparing the original source in a fresh directory:

```sh
repo=/absolute/path/to/tabdat-explore-rs
curl -fsSL https://raw.githubusercontent.com/cran/sampleSelection/5115e01a03d09a46ac1b92813c785c5799b4d654/R/heckitVcov.R -o heckitVcov.R
printf '%s\n' '7506c7a0d7151f909ecc41d4597bfb11f21dc21222f561729fea79fc7decc769  heckitVcov.R' | shasum -a256 -c
Rscript --vanilla "$repo/examples/mma16p1tobit/reference_heckman_two_step.R" \
  mma16p1tobit.asc R_heckman_predictions.csv heckitVcov.R > R_heckman_quantities.csv
```

The reference emits58 exact-inventory quantities (13 covariance entries),1200
point predictions and1400 original cells/ordinals. First-stage log likelihood
is−105.296713010228; outcome intercept745.306997010131, slope176.646780648963,
alpha−498.995959534882; corrected SEs1617.58343858516,425.002572278714,
760.500897353647; sigma740.143348254785 and rho−.674188264626690.
Native validation will target these declared float64/reference conventions, not
claim exact historical Stata storage/stopping/digit parity. Scalar gates planned
at1e-9 absolute+1e-8 relative, positive tails1e-8 relative and probabilities1e-10
absolute; investigate differences rather than widening them.

Current evidence: reference execution and exact schema/inventory checks pass;
independent algebra reconstructs the covariance correction; source cells and all
1200 prediction formulas check independently. Existing-output, modified external
reference, truncated-input, nonbinary-selection and selected-missing-outcome
cases reject with empty stdout and no new prediction artifact. Four existing
input/output/reference fingerprints remain unchanged. Initial top-level chol
printing and named-scalar field suffixes were caught and fixed; their failed
CSV/inventory diagnostics are retained, not counted as passing runs.

## Identification and remaining implementation

There is **no excluded selection predictor** in the author design. Identification
uses the normal functional form; corr(lnwage,Mills)=−.97446818460552 makes the
small-sample outcome estimates imprecise. An exclusion variable is not created or
implied. Negative fitted rho is an estimate, not evidence of a causal mechanism.

The [ledger contract](../../docs/reference/microeconometrics-coverage.md#mma-select-bounded-heckman-two-step-contract-wip)
sets the exact two-stage eligibility/missingness policy, explicit twostep mode,
current latent/selection/Mills/conditional-selected/observed-contribution point
predictions and state/failure requirements. Native kernel, parser/owned state,
reporting, actual scripts, synthetic references and independent review remain
required. Do not present the existing parser-only bare heckman form as ML or
silently reinterpret it as two-step. Conditional-selected means differ from
Tobit truncation, and the zero-filled observed-contribution mean is not an
imputation of unobserved outcomes or a causal effect.
