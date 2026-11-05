# Chapter21 independent-working Gaussian population-averaged GEE

## Preimplementation bounded contract (WIP)

Source P4-IND: [inventory](gls_inventory.md), March2005 author `xtgee lnhr
lnwg, corr(independent) i(id)` and `robust`. Original pinned MOM.dat has 5,320
person-years in 532 complete ten-year blocks 1979–1988, outcome lnhr, wage lnwg,
intercept and equal weights. No missing case deletion. Explicitly validate keys,
finite/bounded model fields, nondegenerate design, residual scale and covariance;
reject invalid samples rather than silently changing them. Preserve source order.

Independent-working Gaussian identity-link estimating equations give the OLS
coefficient and fitted mean, with working scale RSS/N and covariance
`(RSS/N)*(X'X)^-1`. The source's robust covariance is the person-score
sandwich `G/(G-1) (X'X)^-1 sum_i (X_i' r_i)(X_i' r_i)' (X'X)^-1`,
without the additional `(N-1)/(N-2)` OLS CR1 factor. Infer with normal z,
normal95 and one-slope Wald chi-square(1), not OLS t/F. A quick independent R QR
probe of original data reproduces author-printed working and robust SEs using
exactly these corrections; the full audited reference is still required.

Acceptance cases:

1. Actual CLI script in JSON and text prepares/guards the exact source sample,
   fits/predicts, reports both full covariance matrices and normal inference,
   independently audits source rows and predictions, and emits reproducible
   ordered, no-clobber output. Model state and earlier side effects are explicit.
2. Independent R QR/group-score calculations check all coefficient, covariance,
   normal-tail, Wald and source/prediction fields at fixed absolute1e-9 (relative
   1e-8 for nonzero small tail probabilities, exact zero for underflow); selected
   author-log facts are checked only at their printed precision.
3. A small synthetic control checks correction factors, order and both covariance
   modes, and a second control exercises materially different group scores.
4. Invalid/duplicate/missing keys, nonfinite or degenerate model fields, and
   existing export targets fail without clobbering existing data/files. Confirm
   retained model/prediction behavior and do not promise whole-script rollback.

The related [pooled bootstrap](pooled_bootstrap.md) is separately validated on
500 native-stream draws; this bounded slice does not equate those draws to the
P4 log, whose coefficient rows show only72 replications despite a500-request
header. No exchangeable/AR1/unstructured working model or mixed report is
claimed here; all remain open as described in the inventory. The result is a
marginal association under working/independent-person assumptions, not causal
or a formal serial-correlation test. Production must not require R/Python.
