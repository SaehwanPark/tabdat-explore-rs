# Lower-truncated Gaussian regression — source/reference WIP

## Exact source and distinction

The March2005 `mma16p1tobit.do` §2A comments on an intended **truncated Gaussian
MLE**, but executes no correct truncated MLE. It explicitly warns that censoring
Tobit on `ytrunc` reproduces truncated OLS instead. This required-method slice
fills that documented gap on the author's original exported data. It is a
validated extension, not historical coefficient/stopping parity, a published
table replay, or completion of Chapter16. Source assets/checksums/preparation are
in [README](README.md); do not substitute a new random stream.

Use `ytrunc ~ lnwage` plus intercept, fixed lower limit0. Exactly130 original
positive observations contribute conditional densities;70 missing truncated
outcomes contribute **nothing**, not censoring/Bernoulli masses. Retain original
row identities and raw cells. The [ledger contract](../../docs/reference/microeconometrics-coverage.md#mma-trunc-bounded-lower-truncated-gaussian-contract-wip)
defines support/missingness, inference, effects, prediction, failure and resource
semantics. No native command or Rust kernel is implemented yet.

## Independent development reference

- R4.6.1, installed `truncreg`0.2-5 and `maxLik`1.5-2.2.
- CRAN archive `truncreg_0.2-5.tar.gz`, SHA256
  `3e8e6a09eb0a34f2af8b40eb368c2e47bbd6697e0874b595940cad300b948be9`,
  MD5 `6ddc73494a349da69b23f859ffe89a54`.
- CRAN Git mirror commit `8fcad24e3106b94baa093aa9928ca5ceef755916`.
  The generator checks the archive and compares the installed main/fit/likelihood
  function bodies to that pinned source, not just a package-version label.
- GPL>=2 code is a development reference only, not vendored or linked into
  production. maxLik was already available; truncreg is installed in a temporary
  development library, not a runtime/application dependency.

From a prepared working directory, with `repo` pointing at this checkout:

```sh
mkdir -p R-lib R-source
Rscript --vanilla -e 'install.packages("truncreg", repos="https://cloud.r-project.org", lib="R-lib", destdir="R-source", dependencies=FALSE, type="source")'
R_LIBS_USER="$PWD/R-lib" Rscript --vanilla \
  "$repo/examples/mma16p1tobit/reference_truncreg.R" \
  mma16p1tobit.asc R_truncreg_predictions.csv \
  R-source/truncreg_0.2-5.tar.gz > R_truncreg_quantities.csv
```

`maxLik`1.5-2.2 must be available in that development R environment. Future
package releases/source drift fail explicitly rather than silently change the
reference. Existing prediction output is refused. No original data is committed.

The package fit explicitly uses `scaled=FALSE`, natural beta/sigma,
Newton–Raphson and the package's analytic observed information.
The response is divided by1000 solely for numerical conditioning; coefficients,
covariance, log-scale and likelihood Jacobian are all transformed back. Independent
base-R `deriv3` differentiates the actual conditional log density and refines
both full/null observed-Newton solutions. Positive information, score and
unconstrained-step certificates are checked; no OPG or conditional-on-sigma
covariance substitution. Three finite-difference steps separately check the
full-joint conditional-mean AME gradient. A four-start base-R likelihood probe
also reaches the same finite regular source solution, not an existence/global
proof by multistart alone.

Current certified original-source values:

| Quantity | Value |
| --- | ---: |
| Intercept | -1682.93983040031 |
| lnwage slope | 789.454004335952 |
| sigma | 874.882624503239 |
| Conditional log likelihood (130 observations) | -1012.605957391 |
| Same-sample intercept+scale null LL | -1021.50696516938 |
| LR / df / p | 17.8020155567762 / 1 / 0.0000245123441161169 |
| Conditional-truncated-mean AME | 451.714502048142 |
| Full-joint delta AME SE | 104.559756594971 |

This conditional130-row likelihood cannot be compared to the200-row Tobit or
selection likelihood as a joint LR test. The AME is not the latent slope, Tobit
censored-mean derivative, Heckman mean or causal effect.

## Current validation and next implementation

The reference emits49 exact-inventory quantities (45 numeric values plus four
separately bounded full/null certificates), all9 beta/lnsigma covariance entries,
600 current-X projections on200 rows,1400 original cells/ordinals and exact130
membership. Parent independent standard-library checks reconstruct source score,
observed-information inverse, normal inference, AME/full-joint variance and
predictions. Six malformed-input/output controls reject existing output,
modified archive, truncated source, nonbinary source selection, modified selected
outcome and an in-memory modified installed likelihood body. They produce empty
stdout/no new prediction file and preserve all four input fingerprints.

The planned native optimization uses the **concave canonical precision** form:
eta=beta/sigma², v=1/sigma²>0, density proportional to
`exp(y*X*eta-v*y²/2)` on known y>lower. Its negative Hessian is the conditional
covariance of the sufficient statistics; this avoids assuming beta/lnsigma
Newton information is positive away from the optimum. Full beta/lnsigma observed
inference is evaluated/transformed at the certified solution. Scale response and
RMS designs using effective selected data, not a huge statistically inactive
lower bound. Finite interior existence still must be checked, not clipped or
inferred from parser acceptance.

Kernel/synthetic reference and owned runtime/parser/reporting/effects/prediction,
actual scripts/failure controls, affected/reverse checks and independent review
remain required before merge. Scope is fixed-left-truncated IID homoskedastic
Gaussian ML; no upper/variable truncation, robust/cluster/weights, factors, named
storage, generic tests or fabricated sigma0 test. No numerical tolerances have
been broadened to conceal a discrepancy.
