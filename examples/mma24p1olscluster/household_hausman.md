# Household classical joint FE–RE Hausman comparison

## Sample and classical inference

This workflow covers H-HAUSMAN in the [source inventory](inventory.md), original May2005
household program lines123–127. Use the original seven-parameter numeric design
and5,006 complete cases/194 communes; no factors, weights, causal identification
or survey design. Source-row is an ordering key, not time. Compare all six
commune-FE slopes against the opt-in equal-group-between RE-GLS slopes, excluding
the common intercept. Retain each fit's **separate native covariance scale**:
FE within RSS/(N−G−6), RE quasi RSS/(N−7). No common-scale substitution.

For d=b_FE−b_RE and D=V_FE−V_RE report d'D^−1d and asymptotic chi-square6
upper-tail probability, both coefficient vectors, both full slope covariances
and their difference, counts/DF and assumptions. Require a safely positive-
definite covariance difference after scaling by parent covariance diagonals;
reject rank deficiency, indefiniteness, cancellation-dominated differences and
nonfinite statistics. No pseudoinverse, ridge, absolute value or silent dropped
restrictions. A failure leaves data and retained model unchanged. A non-rejection
does not establish exogeneity; validity requires RE efficiency/classical IID
idiosyncratic errors under the null, not cluster-robust inference.

`estat hausman` additionally supports model-based equal-group-between RE-GLS fits,
1..16 numeric slopes/intercept, N<=40,000 and N*K²<=20million with K slopes.
Balanced/unbalanced groups and singletons follow the installed estimator sample.
Retain the original owned design and refit/compare the stored RE result before
post-estimation, independent of current data. Existing balanced one-slope
Swamy–Arora scalar behavior/result wire type stays unchanged. ML, clustered RE,
general named-fit comparisons and unbalanced/multivariate default Swamy–Arora
remain unsupported. The new joint output gets a separate typed JSON result.

Acceptance cases:

1. ASCII/precise-DTA original cohort, both full slope covariances, statistic/p,
   all original source and RE prediction rows agree with independent R QR.
   DTA separate-scale probe17.8864248463 / p.00652238245 matches author17.89/.0065;
   common FE/RE scaling gives different results and is not the contract.
2. Balanced and unbalanced/singleton multi-slope controls match independent R;
   predictor permutation/rescaling leaves the joint statistic invariant.
3. Negative/indefinite, singular and cancellation-dominated covariance
   differences fail rather than producing a generalized or positive statistic.
4. Retained-sample semantics survive current-data edits; mutated/legacy missing
   owned designs and mismatched retained fits fail atomically. Predictions and
   reporting preserve the original RE model; export collisions preserve owners.
5. Existing scalar wire/numerical contracts and unsupported variants regress
   unchanged; new JSON/text/script diagnostics expose assumptions and limits.

## Reproduce, interpret and report

The [household README](README.md) pins source hashes and conversions. Build the
binary, set `repo` to this repository and prepare `vietnam.csv` in a fresh `work`
directory as there. Numeric FARM/EDUC and SEX1/2 codes are not factor-expanded.
The script rejects invalid/non-BIGINT source/commune keys, nonfinite nonmissing
outcomes, missing/nonfinite predictors even on excluded-outcome rows, and raw
source inputs over40,000 rows. It then selects nonmissing outcomes in source-row
order. The native comparison reports candidate N5006/dropped0 because selection
is explicit before fitting; the initial source audit reports5999/993 omissions.

```sh
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma24p1olscluster/household_hausman.td" > result.jsonl)
Rscript "$repo/examples/mma24p1olscluster/household_hausman_reference.R" /path/to/vietnam_ex1.asc "$work/reference"
python3 "$repo/examples/mma24p1olscluster/check_household_hausman.py" "$work" "$work/reference" /path/to/vietnam_ex1.asc
```

For precise DTA, in a separate fresh work directory:

```sh
Rscript "$repo/examples/mma24p1olscluster/dta_reference.R" /path/to/vietnam_ex1.dta "$work/vietnam.csv" --precise
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma24p1olscluster/household_hausman.td" > result.jsonl)
Rscript "$repo/examples/mma24p1olscluster/household_hausman_reference.R" /path/to/vietnam_ex1.dta "$work/reference" --dta
python3 "$repo/examples/mma24p1olscluster/check_household_hausman.py" "$work" "$work/reference" /path/to/vietnam_ex1.dta --dta /path/to/mma24p1olscluster.txt
```

Readable `--text` replaces `--json`; it reports the full three6x6 matrices,
ordered slopes, scales/DF and test/assumptions. CLI-v1 returns the additive
`JointPanelHausmanResult` with the same contents, counts, component/convention
labels and minimum parent-scaled Cholesky pivot. The legacy scalar Swamy–Arora
`ClassicalPanelHausmanResult` is unchanged. Normalized D is symmetrized only for
roundoff-level parent covariance asymmetry; pivots must exceed128 machine
epsilons. Negative eigenvalues/rank deficiency are never repaired. Guard failure
means the classical comparison is unavailable, not that RE was accepted.

The export `household_hausman_predictions.csv` contains all selected source
fields, population `re_xb` and `re_residual=y−xb`, not posterior unit effects.
The retained model remains the original equal-group-between RE fit after the
diagnostic, summaries and export. Existing output owners/source files are not
overwritten. Scripts are ordered and nontransactional: an export failure does
not undo earlier model/table effects. A diagnostic failure itself is read-only
and preserves data/model state, including when the covariance difference is
indefinite. New fit failures retain the installed fit-attempt clearing policy.

## Independent evidence and limits

R4.6.2 within QR, separate full-dummy QR and equal-group-between/quasi-GLS QR
validate139 quantities: counts/DF/scales/pivot/statistic/p, both six-slope
vectors/differences, all36 entries of each of the FE, RE and difference matrices.
All5999 source and5006 prediction rows are independently checked on ASCII and
precise DTA; fixed absolute1e-9 and nonzero-p relative1e-8 tolerances, exact
source fields/keys/order/membership. Maximum scalar absolute differences are
2.27e-11 (ASCII) and3.37e-11 (DTA). The checker also recomputes the full quadratic
from the reported matrices and verifies native RE/slopes/covariance consistency.

ASCII statistic **17.88642125897**, p **0.006522391817**; DTA statistic
**17.88642484628**, p **0.006522382447**. All20 DTA/author printed facts pass half
a last digit: six FE coefficients, six RE coefficients, six differences,
statistic17.89 and p.0065. The author log is source-pinned in the inventory;
these matches do not establish whole-table parity or resolve the separate ML
coefficient/component-SE discrepancies. Forcing a common FE variance gives
18.00865 and forcing a common RE variance18.01706 on the DTA: neither is this
source's separate-scale method, and no tolerance was widened to hide that.

Frozen [balanced/unbalanced controls](../../crates/tabdat-stats/tests/fixtures/panel_joint_equal_group_hausman/README.md)
check all139 quantities and every prediction row, including a singleton/missing
source outcome. Closed-form and predictor permutation/rescaling tests, forged
retained-design/result rejection, old-state serde defaults, current-data and
no-active-data independence, actual indefinite-fit rejection and export/state
failure paths are covered. The optional
`python3 examples/mma24p1olscluster/test_check_household_hausman.py` rejects17
forgeries, including a shared native/reference wrong statistic.

Independent R/Python are development tools, not runtime dependencies. Default
unbalanced/multi-slope Swamy–Arora, ML/robust/clustered Hausman, common-scale
variants, rank-reduced generalized tests, named arbitrary-fit comparison and
complete Chapter24/table parity remain unclaimed. Unified comparisons and
remaining descriptive/graphical/source requirements in the inventory stay open.
