# MMA §14.7.5 maximum-score binary response

## Outcome

Add a bounded native maximum-score workflow for the fishing-mode charter-versus-
pier example: binary `charter` on an intercept and one continuous `lnrelp`
predictor, using the 630-row author sample. The estimator will enumerate all
one-dimensional threshold regions in both slope directions, report a unit-norm
coefficient representative and the exact empirical score, and retain the
classification for terminal, JSON and current-row prediction output.

This scope does not add a general multivariable maximum-score optimizer,
probability predictions, marginal effects, weights or a covariance estimator.
Maximum-score coefficients are identified only up to scale; predictions are
sign classifications. Tied optimal threshold regions must remain visible and
receive a deterministic representative without implying unique identification.

## Current behavior and evidence

- The coverage ledger marks MMA-MAXSCORE planned; no native maximum-score
  command exists. The existing binary workflow prepares `charter` and `lnrelp`
  from the 1,182-row fishing-mode source and retains the pier/charter subset.
- The official LIMDEP program specifies `MSCORE; Lhs=charter; Rhs=ONE,lnrelp;
  Nbt=50`. Its output reports the maximum-score coefficient and classification
  table. It labels its 50-replication standard errors and t-ratios approximate.
- An independent exact threshold enumeration of the official two-column data
  finds two maximizing threshold regions and 536 correct classifications out
  of 630, matching the author output's confusion counts. The printed coefficient
  vector is one representative of this tied finite-sample optimum.
- Maximum-score inference has cube-root, non-normal asymptotics. The ordinary
  nonparametric bootstrap does not consistently estimate its limiting
  distribution.

## Changes

1. Implement a pure statistics function for the bounded one-predictor problem.
   Enumerate open intervals between sorted distinct predictor values for each
   slope sign, include constant-classification candidates, maximize correct
   classifications, retain all tied maximizing regions and normalize a chosen
   representative to Euclidean norm one.
2. Add the typed command/result, parser, retained-fit state, execution dispatch,
   CLI availability/help, and terminal/JSON reporting. Fail before state changes
   for invalid outcomes, insufficient predictor variation and empty samples.
3. Add an example command script and an independent Base R threshold-enumeration
   reference/checker over source-derived data. Cover both slope signs, tied
   optima, constant predictions, exact boundary values and failure atomicity.
4. Update the binary coverage ledger, SPEC and the MMA §14.2 example README with
   the exact sample, normalization, score, limitations and reproducible command.

## Verification

- Recreate the 630-row charter/pier input from the checked source conversion and
  preserve source-row order.
- Compare Rust against an independent R enumeration of every classification
  region, including score, ties, representative threshold and confusion counts.
- Confirm the source fixture's 536/630 optimum and all four confusion cells.
- Exercise no-fit, invalid/nonbinary outcome, constant predictor, all-tied
  thresholds, failed execution state and retained results after current-data
  edits; verify text and JSON agree.
- Run focused stats, runtime and CLI contracts, then scoped Clippy, formatting
  and diff checks. Keep the 50-bootstrap LIMDEP standard errors out of formal
  inference unless their treatment is decided first.

## Reporting-contract decision

The author program requests 50 naive bootstrap replications, but both its output
warning and published theory make those standard errors unsuitable as formal
inference. The product choice is whether to:

1. report only the point estimator and classification (recommended),
2. reproduce the source's 50-replication standard-error diagnostics with an
   explicit approximation label and no formal significance claims, or
3. add a separately justified inference method, which would no longer reproduce
   the author's `Nbt=50` uncertainty output.

The pure finite-sample point-estimation routine and an independent exact
threshold reference are common to all three choices and can proceed. Resolve
this choice before adding CLI result fields or reports for uncertainty; those
outputs must not imply formal inference unless the selected method supports it.

## Sources

- [Author program](https://cameron.econ.ucdavis.edu/mmabook/mma14p2maxscore.lim)
  and [LIMDEP output](https://cameron.econ.ucdavis.edu/mmabook/mma14p2maxscore.out)
- Manski (1975), [“Maximum Score Estimation of the Stochastic Utility Model of
  Choice”](https://doi.org/10.1016/0304-4076(75)90032-9)
- Abrevaya and Huang (2005), [“On the Bootstrap of the Maximum Score
  Estimator”](https://doi.org/10.1111/j.1468-0262.2005.00613.x)
