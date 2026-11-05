# Chapter21 independent-working Gaussian population-averaged GEE

## Target and contract

Source P4-IND: [inventory](gls_inventory.md), March2005 author `xtgee lnhr
lnwg, corr(independent) i(id)` and `robust`. Original pinned MOM.dat has 5,320
person-years in 532 complete ten-year blocks 1979–1988, outcome lnhr, wage lnwg,
intercept and equal weights. No missing case deletion. The script accepts 3–532
complete people, unique finite exact id/year and source_row in MOM ranges, and
finite model fields with magnitude at most20. It rejects unidentified wage
variation, RSS<=1e-12, unrepresentable coefficients/covariance, unresolved
marginal sandwich variances (each must exceed 128 machine epsilons times the
corresponding working variance), or a failed estimating-score certificate. The
variance bound prevents rounding noise in mathematically zero unit scores from
being presented as highly significant inference. These numerical bounds are
conservative workflow guards, not universal GEE admission rules. SQL guard
failures can surface as generic `sql failed`; check the sample and guards above.
Ancillary fields are preserved but the script does not impose finite values on
them; the independent original/control audit checks all original source fields.

Independent-working Gaussian identity-link estimating equations give the OLS
coefficient and fitted mean, with working scale RSS/N and covariance
`(RSS/N)*(X'X)^-1`. The separately labelled robust covariance is the person-score
sandwich `G/(G-1) (X'X)^-1 sum_i (X_i' r_i)(X_i' r_i)' (X'X)^-1`,
without the additional `(N-1)/(N-2)` OLS CR1 factor. The report gives full2x2
covariances, normal z/two-sided p/normal95 and the slope Wald chi-square(1), not
OLS t/F. Only the slope gets a model Wald statistic; no joint covariance-rank
test is claimed. Original wage estimate0.08274354528 has working SE0.00912342108
and person-sandwich SE0.02926839471. Working inference assumes independent errors;
the sandwich requires independent people and a correctly specified marginal mean,
not independent person-years. This association is not causal, a formal serial
correlation test, survey inference or a few-cluster coverage guarantee.

[Script](independent_gee.td) uses existing OLS, named SQL tables, the validated
normal-CDF SQL function and no new parser/kernel/API. The owned retained model is
**pooled OLS**, not GEE; the OLS result's t/F/working covariance are not the GEE
report. Named GEE mean/residual columns are in-sample and a later `predict` still
uses the original pooled model. There is no general `xtgee`, retained GEE state,
working-correlation selector or prediction uncertainty.

## Reproduction and outputs

Prepare the pinned publisher data as in [shared preparation](README.md#reproduce),
in fresh separate JSON/text directories. From the checkout root:

```sh
root="$PWD"; work=/path/to/fresh/work
sh "$root/examples/mma21p1panel/prepare.sh" /path/to/MOM.dat "$work/MOM.csv"
(cd "$work" && "$root/target/debug/tabdat-explore-rs" --json -f "$root/examples/mma21p1panel/independent_gee.td" > output.jsonl)
Rscript "$root/examples/mma21p1panel/independent_gee_reference.R" "$work/MOM.csv" "$work/reference"
python3 "$root/examples/mma21p1panel/check_independent_gee.py" "$work" "$work/reference" /path/to/MOM.dat
```

Run `--text` instead of `--json` in another fresh directory for readable batch
results; default batch output is quiet. The four no-clobber exports, in order, are
`panel_independent_gee_fit.csv` (sample/scale/full matrices/score certificate),
`panel_independent_gee_coefficients.csv` (four normal-inference rows, also a
readable/typed table), `panel_independent_gee_predictions.csv` (all source and
pooled/GEE predictions) and `panel_independent_gee_unit_scores.csv` (one
ordered unit's centered score and influence per person). The final active table
is predictions. Exports are not a whole-script transaction: an existing later
target is preserved but earlier exports/session effects can remain.

## Independent evidence and limits

Base R4.6.2 `lm.fit` QR, QR-based bread and independently grouped score
crossproducts validate the original sample: 18 fit quantities,26 inference
quantities, all5,320x13 source/prediction fields and all532x6 unit-score fields.
The checker independently binds all42,560 raw source fields and ordering to the
pinned MOM.dat SHA256. Original JSON/text exports are byte-identical. Fixed
absolute1e-9 applies to numerical values; nonzero p-values use relative1e-8 and
underflow must be exact zero. Maximum absolute original R discrepancies are
1.05e-11 fit,1.09e-11 inference,2.88e-13 predictions and5.57e-12 scores.
Two independent32-person controls, with materially different score structures,
validate both matrices, all rows and relative p tails. Focused tests cover
key/missing/rank/perfect-fit and unresolved-zero-unit-score failures, retained
model, text/JSON and each export collision; six numeric forgeries of fit/inference/prediction/source/scores are
rejected by the checker.

An R calculation on separately float32-rounded lnhr/lnwg matches24 author-log
facts at their printed precision; this is a plausible input-storage hypothesis,
not proof of historical storage and not a replacement for raw-double R evidence.
For example raw-double slope0.08274354528 rounds to0.0827435 while the source
prints0.0827436; float32-input slope0.08274355016 rounds to the latter. The raw
and author normal interval endpoints also differ at a last printed digit. No
tolerance was widened. The related [pooled bootstrap](pooled_bootstrap.md)
independently checks 500 native-stream refits, but P4's log requests500 and
prints only72 coefficient replications; neither historical RNG nor cause of
that discrepancy is established. Exchangeable/AR1/unstructured working models,
other P4 bootstraps and the full mixed report remain required as itemized in the
[inventory](gls_inventory.md); source's unstructured bootstrap is commented out
and its final matrix listing fails `r(111)`.
