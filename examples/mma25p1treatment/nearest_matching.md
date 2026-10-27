# Descriptive nearest-score comparison

[`nearest_matching.td`](nearest_matching.td) fits the DW99 assignment logit
used in the [NSW/PSID treatment examples](README.md), selects the inclusive
empirical intersection of the treated and control score ranges, and matches
each retained treated person to their closest retained control score. Controls
can be reused; exactly equidistant nearest controls are averaged. This is a
bounded, descriptive illustration related to Table 25.6, not a reproduction
of the author's random tie handling, historical support rule or reported
result. The author's logged DW02 and DW99 variants use different specifications.

The output reports the mean treated-minus-matched 1978 earnings contrast,
nearest-score gaps, exact ties, distinct matched controls and maximum reuse. It
also reports treated and matched-control means of age, education, Black
indicator, and earnings in 1974 and 1975. A pair contributes `1/tied_controls`
to each side, so each treated person has total weight one and reused controls
accumulate weight. These unstandardized means describe five variables in the
matched sample; they do not measure all aspects of balance or show changes from
before matching. Pair-level outcomes, scores, covariates and tie counts are
exported for audit.

On the original 2,675-person sample, 179 treated people remain in the empirical
range; they use 57 distinct controls, and three treated people have tied nearest
controls. The descriptive contrast is 515.906, the largest score gap is about
0.087, and the maximum fractional control reuse is 27. The treated and matched
control Black shares are 0.838 and 0.922 respectively, illustrating remaining
imbalance. These diagnostics do not establish positivity, exchangeability or
causality; the target is the selected sample's observed contrast, not an
identified population ATT.

**No standard error or confidence interval is estimated.** Fixed-neighbor
matching has nonstandard uncertainty: the ordinary bootstrap is not generally
valid ([Abadie and Imbens, 2008](https://doi.org/10.3982/ECTA6474)), and the
estimated-score adjustment can raise or lower ATT variance ([Abadie and Imbens,
2016](https://doi.org/10.3982/ECTA11293)). The empirical min/max selection and
exact-score ties require additional justification beyond that paper's
continuous-score conditions. The separate [fixed-bandwidth kernel
comparison](README.md#fixed-bandwidth-kernel-matching) reports uncertainty for
a different, smoothed target; it is not an interval for this contrast.

The script requires finite ordered dense IDs, binary treatment and indicators,
nonnegative finite pretreatment earnings, a nondegenerate assignment fit, at
least two retained people per arm and no more than 2,000,000 treated-control
candidate pairs. Scores rounded to zero or one are accepted only outside the
selected range and are never used in a distance. It rejects nonfinite matched
outcomes and aggregate contrasts and will not overwrite the pair export;
earlier session effects are not rolled back. This is a script composition, not
a general matching command.

From a fresh prepared directory:

```sh
/path/to/tabdat-explore-rs --json -f /path/to/repo/examples/mma25p1treatment/nearest_matching.td > nearest_result.jsonl
Rscript /path/to/repo/examples/mma25p1treatment/nearest_matching_reference.R /path/to/nswpsid.da1 nearest-reference
python3 /path/to/repo/examples/mma25p1treatment/check_nearest_matching.py /path/to/nswpsid.da1 nearest_result.jsonl nearest_pairs.csv nearest-reference
```

An independent base-R fit and brute-force search check all source rows and
nearest pairs, outcomes, exact ties, reuse, contrast and five matched means.
The tolerance is `1e-8 + 1e-9*abs(reference)`, with `1e-7` relative agreement
for tiny scores. Focused CLI tests cover matching, duplicated-control ties,
weighted means, rounded-score boundaries, invalid inputs and no-clobber
exports. These checks do not validate nearest-neighbor inference or a causal
interpretation.
