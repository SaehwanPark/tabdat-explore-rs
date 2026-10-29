# Chapter 23 patents: pooled Poisson illustration

This is the **pooled Poisson column** of Cameron and Trivedi (2005), §23.3,
Table 23.1 (p.794), from the March 2005
[`mma23p1pannonlin.do`](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.do)
and [author output](https://cameron.econ.ucdavis.edu/mmabook/mma23p1pannonlin.txt).
The complete [native script](pooled_poisson.td) prepares a pooled sample, checks
firm/year keys and balanced five-year membership, fits model-based, independent-
row robust, and firm-clustered Poisson, reports continuous effects, summaries,
means and linear indices, and exports the original/design/prediction rows.
Text and CLI-v1 JSON use the same script. This is not the full table or chapter.

## Source, sample and design

Download [`patr7079.asc`](https://cameron.econ.ucdavis.edu/mmabook/patr7079.asc)
under its applicable terms; no author data/program is bundled. Its SHA-256 is
`87525a1ae9112c235326bf3197cb57dced73c6f166d84e261395a737a390fee8`.
The source contains 346 firm records of 25 free-format tokens each across
four physical lines: five firm metadata fields, ten log-R&D observations, and
ten patent-count observations for 1970–79. The converter is independent of
physical line breaks and assigns firm `id` by **original record order**, as the
author does; CUSIP is not used as the key. Ten missing `ARDSSIC` industry-code
metadata tokens (`.`) become missing CSV values, not zero. The converter rejects
invalid required fields/counts and incomplete or empty input before publishing;
it stages beside the destination and refuses to replace an existing file. CUSIP
and patent-count tokens must use nonnegative integer spellings, optionally with
only zero decimal digits; patent counts are bounded at 1,000,000,000.

Only 1975–79 are modeled: 1,730 original firm-years, 346 firms with five years
apiece, no further model exclusions, count sum 60,155 and 337 zero counts. The
outcome is `pat`, the regressor `logr` and an intercept form the design, and
there are no extra regressors, offset or weights. All 22 firms with zero patents
across these five years remain in the **pooled** sample; they are dropped by the
separate author conditional fixed-effects model, which this workflow does not fit.
The export includes the original firm metadata, id/year, selected PAT/LOGR and
current-data `poisson_mu`/`poisson_xb` for every selected observation.

## Inference and interpretation

All three fits share the pooled log-link mean and coefficients. Model covariance
is the inverse Poisson information. For scores `s_it = x_it(y_it−mu_it)` and
bread `B = (X' diag(mu) X)^−1`, the independent-row sandwich uses
`N/(N−1) B (sum s_it s_it') B`; the firm sandwich uses
`G/(G−1) B (sum_i S_i S_i') B`, `S_i=sum_t s_it`. The robust modes report
asymptotic normal coefficient/effect inference and a chi-square slope Wald test;
the model-based LR reference is withheld for robust fits. Continuous average
`logr` effects use the retained original design and conditional-design delta
uncertainty, not a discrete R&D policy change. Current means/indices use current
rows. Full Poisson likelihood and descriptive Pearson/deviance quantities do
not establish the conditional count distribution or a specification test.

| Native float64 | Intercept | LOGR slope | Intercept SE | LOGR SE |
| --- | ---: | ---: | ---: | ---: |
| Model-based | 1.7115282420 | 0.6929337099 | 0.0097669535 | 0.0022453708 |
| Independent-row robust | 1.7115282420 | 0.6929337099 | 0.0620025218 | 0.0198091733 |
| Firm-cluster robust | 1.7115282420 | 0.6929337099 | 0.1340309035 | 0.0430441306 |

The source log prints slope `.6929337` and SEs `.0022454`, `.0198092` and
`.0430441`; these selected rounded facts agree. The independent-row modes are
not panel robust. Firm clustering assumes independent firms, permits within-firm
dependence, and offers no few-cluster guarantee. Pooled Poisson ignores firm
heterogeneity and does not identify a causal R&D effect; robust covariance does
not repair a misspecified conditional mean.

## Reproduce and validate

In an empty directory, with R 4.6.1 and Python 3 for development validation:

```sh
repo=/absolute/path/to/tabdat-explore-rs
work=/absolute/path/to/empty/work-directory
curl -fsSL https://cameron.econ.ucdavis.edu/mmabook/patr7079.asc -o "$work/patr7079.asc"
(cd "$repo" && cargo build --locked --bin tabdat-explore-rs)
sh "$repo/examples/mma23p1pannonlin/prepare.sh" "$work/patr7079.asc" "$work/patents.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json -f "$repo/examples/mma23p1pannonlin/pooled_poisson.td" > result.jsonl)
Rscript "$repo/examples/mma23p1pannonlin/pooled_poisson_reference.R" "$work/patr7079.asc" "$work/r-reference"
python3 "$repo/examples/mma23p1pannonlin/check_pooled_poisson.py" "$work" "$work/r-reference" "$work/patr7079.asc"
```

Base R independently reshapes raw source and fits `glm`; an explicit score and
information calculation supplies all three covariances and delta-effect SEs.
The checker pins the source hash, validates 103 R quantities including all
covariance entries and effects, and checks 1,730 source-derived records/19,030
exported cells. Numeric tolerance is `1e-10 + 1e-10*abs(reference)`; nonzero
p-value tails use relative `1e-9` with a minimal positive floor. The relative
rule checks tiny tails without treating them as zero. Selected author-log values
are checked to their displayed decimal precision. Original text/JSON CSV exports
were byte-identical. The committed synthetic 40-firm fixture and workflow tests
check the same 103 independent R quantities, predictions and preparation,
key/panel guard and no-clobber failure paths; regenerate it with
`synthetic_source.R` and `pooled_poisson_reference.R` in fresh directories.
R/Python are not production dependencies.

Separate workflows now cover [conditional-FE Poisson](conditional_fe.md),
[exponential-mean NLS](exponential_nls.md), and whole-firm bootstraps for
[pooled](pooled_poisson_bootstrap.md) and [conditional-FE](conditional_fe_bootstrap.md)
Poisson, without historical RNG parity. The bounded [exchangeable Poisson GEE](exchangeable_gee.md)
is also separate. Random-effects Poisson, general GEE, other bootstraps and the
figure remain required. These bounded workflows
do not complete Table 23.1, the empirical inventory or the chapter.
