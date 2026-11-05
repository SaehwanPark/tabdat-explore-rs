# Chapter 24 household OLS cluster comparison — in progress

This bounded slice targets Cameron–Trivedi (2005) §24.7 Table 24.4,
OLS columns 1–4: intercept regression of `LNEXP12M` on `LNHHEXP AGE SEX
HHSIZE FARM EDUC`, using complete cases and comparing IID, HC1, and
commune-clustered CR1 covariance. It does not cover random/fixed effects,
GEE, bootstrap, the separate Poisson illustration, or complex-survey
weights/strata. Independence across communes and the assumptions of each
reported covariance are substantive assumptions, not established here.

The publisher's [`vietnam_ex1.asc`](https://cameron.econ.ucdavis.edu/mmabook/vietnam_ex1.asc)
contains 5,999 records, 993 missing log-medical outcomes and 194 communes.
SHA256 `74cb37e8c14609e382d334e368a54f63bccd4e952d02bcbf535baf19115d4061`.
It wraps records across physical lines, prints `.` for missing outcomes, and
prints FARM and EDUC value labels instead of numeric codes. Numeric label
mappings must be checked against the publisher's original
[`vietnam_ex1.dta`](https://cameron.econ.ucdavis.edu/mmabook/vietnam_ex1.dta),
SHA256 `0ea6639f93a5bc3f53bcbd26b6865ad6908a04f41ab04d90e7c9aa4b7e9787bf`;
its underlying binary floats retain more precision than the ASCII export. No
binary identity or exact printed-author replication is implied by ASCII input.

## Acceptance contract

- Prepare the publisher ASCII source without splitting quoted labels or wrapped
  records; preserve row order, missingness and actual numeric category codes.
  Reject malformed source, unknown labels, unexpected counts, and output
  collisions without overwriting inputs or an existing owner.
- Fit all three models on the same 5,006 complete observations, including
  valid zero outcomes; retain all 194 commune identifiers. Report sample,
  covariance correction, inference degrees of freedom and prediction/residual
  identity. Compare classical, HC1 and CR1 point estimates, covariance,
  standard errors and t inference to independent base-R QR and explicitly
  constructed sandwiches on the same ASCII values. Fixed numerical tolerances
  must be documented; separately investigate DTA/author-output precision.
- Exercise representative invalid/rank-deficient/single-cluster inputs,
  state and no-clobber failures, a small independently checked synthetic case,
  actual text and JSON runs and source/prediction row audit. State retained
  model and nontransactional file effects explicitly.

**Status:** contract only; no executable or validated workflow yet.
