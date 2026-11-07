# Chapter 24 individual pooled Poisson comparison

This bounded workflow covers Cameron–Trivedi (2005) §24.7/Table 24.6 pooled
Poisson columns 1–4, from
[`mma24p2poiscluster.do`](https://cameron.econ.ucdavis.edu/mmabook/mma24p2poiscluster.do)
(SHA256 `746da2eca6d5970a045c137f106cfe33514e2f6e9595f26cb5451b3eb4bd2376`).
It fits an intercept and `LNHHEXP INSURANCE SEX AGE MARRIED ILLDAYS ACTDAYS
INJURY ILLNESS EDUC` to individual `PHARVIS`; model-based, independent-row
robust and commune-clustered covariance are reported separately. This is
unweighted pooled Poisson, not complex-survey design, causal, fixed/random-
effects or bootstrap inference, nor the complete table/chapter. Robust SEs do
not repair a misspecified conditional mean; commune clustering assumes
independent communes and does not guarantee reliable few-cluster inference.

A separate [native conditional Poisson FE workflow](individual_conditional_fe.md)
now covers the individual FE column, with 27,671 retained people/193 communes,
179 independent R fit quantities, all source/prediction/exclusion rows and a
20-fact native DTA author coefficient/SE audit. It does not alter the pooled
workflow below or imply clustered FE or survey inference. The separate
[shared-gamma RE workflow](individual_gamma_re.md) now retains all27,765 people/
194 communes, validates236 R quantities and every source/marginal-prediction row
on ASCII and DTA, and matches29 printed native DTA author facts. It is model-based
shared-group mixing, not row NB2, posterior-effect or complex-survey inference.
The separate [pooled Poisson whole-commune bootstrap](individual_pooled_bootstrap.md)
refits all11 coefficients for each of500 group resamples and independently
checks all97,000 ordered draws, full covariance and all source/prediction rows.
The separate [conditional-FE whole-commune bootstrap](individual_conditional_fe_bootstrap.md)
samples the193 originally retained positive-total communes, independently checks
all500 conditional-score refits,96,500 ordered draws, full covariance and all
source/prediction rows on ASCII and precise DTA, with raw-copy dummy-GLM synthetic
controls. Nine of ten printed author bootstrap points match to half a digit; the
remaining last-digit `MARRIED` discrepancy is explicit in the linked audit.
The separate [shared-gamma RE whole-commune bootstrap](individual_gamma_re_bootstrap.md)
retains all194 groups, refits all12 joint parameters for each distinct group copy,
and checks500 R likelihood refits,97,000 draws, full covariance and all source/
prediction rows on ASCII and precise DTA. Ten of twelve author bootstrap original
points match half a digit; two final-digit discrepancies and historical copy-ID
ambiguity are explicit in the linked audit. None supplies survey-design inference.

The [source-pinned companion inventory](../mma24p1olscluster/inventory.md)
maps both Chapter24 programs and retains open descriptive/frequency/graphical
and comparison requirements. Three sequential pooled fits are not the source's
named comparison, and author-log matches do not establish printed-table parity.

## Source, design and execution

Download publisher [`vietnam_ex2.asc`](https://cameron.econ.ucdavis.edu/mmabook/vietnam_ex2.asc)
(SHA256 `e876f8ee17791881c6378de2761436e1e72e1eda84347089a9dc365ed290c85d`).
It has 27,766 ordered twelve-field records across physical line breaks; the
only incomplete record is **entirely missing**, source row 27,734. Excluding
it leaves 27,765 individuals in 194 communes, 20,668 zero counts and a count
sum of 14,209. `source_row` records physical source order, not an individual
survey identifier. All original fields and zero counts are retained. The
original [`vietnam_ex2.dta`](https://cameron.econ.ucdavis.edu/mmabook/vietnam_ex2.dta)
SHA256 `284a5dd732926fe6ee02157012b17d8751523b89f1dd2f652a6f4dd021f3dac3`
contains more precise binary floats. The author renames source `lnhhinc` to
`LNHHEXP`; this workflow preserves that mapping rather than asserting which
underlying income/expenditure concept it measures. `AGE` is modeled as its
original numeric column (its values resemble logged ages); no transformation
or years-based interpretation is silently introduced. `ILLNESS` comes from
`ILLDUM` but its observed numeric codes range from 0 to 9, not just 0/1.
`EDUC` and other numeric codes enter as numeric columns, not factor expansion.

From an otherwise empty work directory, with `repo` set to the repository path:

```sh
python3 "$repo/examples/mma24p2poiscluster/prepare.py" /path/to/vietnam_ex2.asc vietnam_individual.csv
"$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma24p2poiscluster/individual_poisson.td" > result.jsonl
```

Build the binary first if necessary. The optional preparer uses Python's
standard library only; Rust runtime has no Python or R dependency. Alternatively
supply a numeric CSV with the same header and stable source-row keys. Input is
unweighted, must have unique positive BIGINT source-row and commune keys, exact
nonnegative BIGINT counts within the native Poisson bound and finite model
fields. A completely empty record apart from `source_row` is excluded; a
partially missing record is rejected, rather than silently changing the sample.
Exact integer key/count columns prevent binary-float rounding from accepting
near-integer text. Identification, convergence and two-cluster guards apply;
SQL guards can report only `sql failed`. Text mode runs the same script and
writes the same prediction CSV.

## Interpretation and retained state

All fits use the same log-link mean. With `B=(X' diag(mu) X)^−1` and individual
scores `s_i=x_i(y_i−mu_i)`, the independent-row sandwich is `N/(N−1)` times
`B (sum s_i s_i') B`; commune covariance uses `G/(G−1)` and sums scores within
each commune first. The model covariance is `B`. Unlike the separate linear
OLS workflow, these corrections do **not** include `(N−1)/(N−K)`, and coefficient
and margin references are asymptotic normal, not a `G−1` t reference. Robust
fits report a chi-square joint slope Wald test rather than an LR p-value.
Likelihood, Pearson and deviance are descriptive and do not establish a valid
conditional count law or a formal specification test.

`estat margins` reports continuous-column average derivatives with conditional-
design delta SE for **every** supplied predictor. These derivatives do not
become binary/ordinal discrete changes or causal policy effects merely because
a column has coded values. Predictions `pooled_mu` and `pooled_xb` refer to
current selected rows; `individual_poisson_predictions.csv` retains all source
fields in source order. The final retained model is commune-clustered Poisson,
which replaces any previous model. The export refuses replacement, but the
multi-command workflow is not transactional: prior table/model changes and
output events can survive a later failure. Use an empty work directory.

## Independent evidence and reproduction

Base R 4.6.2 `glm` with converged refreshed weights and independently formed
row/commune scores, information and delta gradients checks **778** fit, full
covariance, normal-inference, slope-Wald, fit-diagnostic and continuous-effect
quantities across the three covariance conventions. Absolute tolerance is
`1e-9`; nonzero p-values use relative `1e-8`, and an underflowed reference zero
requires exact zero. All 27,766 prepared rows are bound to the pinned source and
independently parsed R source; every source field, mean and index on the 27,765
retained rows is checked against R. Actual text/JSON CSV exports are byte-identical.
A separate synthetic 240-observation/21-commune R reference covers all 778
fields, one wholly missing source record, zero counts and source/prediction
identity. CLI integration tests cover duplicate, decimal-rounded, missing and
invalid keys/counts, partially missing/nonfinite rows, singular design, zero
counts, a single cluster, source preservation and output collision. Auditor
forgery checks reject altered source, mean, index, coefficient, covariance,
effect and missing rows.

```sh
Rscript "$repo/examples/mma24p2poiscluster/reference.R" /path/to/vietnam_ex2.asc reference
python3 "$repo/examples/mma24p2poiscluster/check_reference.py" . reference /path/to/vietnam_ex2.asc
```

The publisher [log](https://cameron.econ.ucdavis.edu/mmabook/mma24p2poiscluster.txt)
SHA256 `b86983173c9c712d10df522a0d3ce566fd7c03b77d9f9739b9e776c44223d33e`
and the DTA/R reference provide a separate historical precision investigation:
all 27,766 rows have identical missingness and noncontinuous values; the
largest DTA/ASCII continuous cell difference is `4.99985e-7`. Of 66 author-log
coefficient/SE facts, **65** match the half-unit rounding interval of their
printed final digit. The clustered ACTDAYS SE is the unresolved exception: R on
the DTA gives `0.0190493502655971`, just `2.655971e-10` above the rounding
boundary for the log's `.0190493`. This historical last-digit discrepancy is
not counted as a match and its cause is not established; no canonical ASCII/R
tolerance was relaxed. The author program itself says book Table 24.6 differs
in its second significant digit despite equal sample size; no book-table parity
is claimed. To repeat the DTA investigation in a fresh work directory:

```sh
Rscript "$repo/examples/mma24p2poiscluster/dta_reference.R" /path/to/vietnam_ex2.dta dta.csv
Rscript "$repo/examples/mma24p2poiscluster/reference.R" dta.csv dta-reference --csv
python3 "$repo/examples/mma24p2poiscluster/check_author.py" /path/to/vietnam_ex2.dta /path/to/mma24p2poiscluster.txt dta.csv dta-reference vietnam_individual.csv
```

R/Python are development preparation/reference tools, not runtime dependencies.
Reference directories and the DTA converter refuse to replace existing targets;
publisher data and source programs are not redistributed.
