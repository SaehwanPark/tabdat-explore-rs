# Chapter 23 log-patents linear panel — WIP contract

## Exact target

Reproduce the March 2005 [author program](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.do)
linear-panel sequence for the §23.3 patents/R&D illustration and its
[output](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.txt): the
same 346 source-order firms and 1,730 firm-years, 1975–79, including 22 firms
with no patents in those five years. Pinned source `patr7079.asc` SHA-256:
`87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8`.
Model `LPAT = ln(PAT)` for positive counts and `ln(0.5)` for zeros, on LOGR
and intercept only. The zero substitution is ad hoc; no causal or count-model
interpretation is implied. Missing nonmodel ARDSSIC does not drop observations.

In one ordered load/prepare → estimate → diagnose → report script, fit pooled
OLS with firm-cluster covariance, one-way person fixed effects with classical
within covariance, and balanced one-way Swamy–Arora random-effects GLS with
classical covariance. Compare slope inference and clearly label model-specific
references/degrees of freedom and normalization; export source fields and all
pooled, within-conditional and population RE predictions/residuals. Native FE
normalizes the intercept but does not report its individual or common-intercept
SE; do not claim full author FE-column parity. Scripts report separate typed
fit results, not a restorable multi-model estimation store. The author Figure
23.1 and all remaining chapter/book work are separate requirements.

## Acceptance and failure cases

1. Independently construct source and model sample in R, check all 1,730
   identities, transformations and predictions, and compare three fits,
   covariance matrices, declared tests, cluster corrections and RE variance
   components with explicit tolerances. Investigate historical printed results
   and any storage/precision differences rather than widening tolerance.
2. Exercise an inspectable synthetic panel with zeros, missing nonmodel data,
   text/JSON, input order and no-clobber. Preserve file/session state and test
   malformed/missing/duplicate/out-of-range inputs, singular model design and
   invalid or degenerate panel components.
3. Declare retained model state, earlier effects of nontransactional script
   failures and prediction semantics explicitly; update SPEC, roadmap and
   coverage ledger only after implementation/evidence. Figure23.1 and remaining
   Table23.1 material remain open. WIP is not a capability.
