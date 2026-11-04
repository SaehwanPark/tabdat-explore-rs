# Chapter 22 wage Shea relevance diagnostic

## Bounded contract

Reproduce the author program's §22.3 p.756 base-instrument wage diagnostic,
not a general weak-identification test. Use pinned `MOMprecise.dat`, 532 complete
1979–1988 people and the existing 4,256-row, five-difference/nine-instrument
panel design. All seven auxiliary regressions include an intercept: residualize
wage difference on the other four differences; regress each difference on the
nine instruments and obtain five fitted values; residualize fitted wage on the
other four fitted values. Report the squared Pearson correlation of the two
residual vectors. This intercept-including author diagnostic is distinct from
Shea diagnostics of the no-intercept structural 2SLS model.

Validation contract:

1. Execute load/guard/prepare → seven fits and current-data predictions →
   explicitly labelled diagnostic summary → row export in text and CLI-v1
   JSON. Check source membership/order and every design/fitted/residual row,
   seven coefficient vectors and the correlation/its square against independent
   R QR/correlation; compare selected author-log values with explicit precision.
2. Include a small inspectable synthetic actual-script control, reversed input,
   missing/duplicate-year and singular-design errors. Check zero-variance/rank
   failure and no-clobber output; scripts are ordered rather than transactional.
3. Update the capability map and coverage ledger only with this bounded source
   result, documenting assumptions and any observed historical discrepancies.
   Run affected integration tests and proportionate checks.

No claim of joint instrument strength, causality, finite-sample inference,
weak-IV-robust confidence sets, whole Chapter 22, arbitrary included exogenous
variables or a general-purpose new command. The squared correlation is a
descriptive relevance measure, not a test of instrument validity.

## Reproduction and evidence

Prepare the pinned data and independent base R design using the commands in the
[example README](README.md), then run in a fresh output directory:

```sh
/path/to/tabdat-explore-rs --json -f /path/to/repo/examples/mma22p1panel/shea.td > shea-result.jsonl
Rscript /path/to/repo/examples/mma22p1panel/shea_reference.R /path/to/reference/design.csv shea-reference
python3 /path/to/repo/examples/mma22p1panel/check_shea.py /path/to/MOMprecise.dat shea-result.jsonl shea_design.csv shea-reference
```

Independent R 4.6.1 QR regressions and Pearson correlation give rho
0.06039379111916 and squared correlation 0.003647410005745, matching the author
log's printed 0.00364741 to eight decimal places. The checker verifies the pinned
source hash, all 4,256 design/prediction rows, all seven coefficient vectors,
classical covariance matrices, SEs, p-values and R², plus the diagnostic, at
`1e-9 + 1e-9*abs(reference)`. These auxiliary OLS inference results are not
weak-IV-robust structural inference. Independent R also supplies a deterministic
320-row/40-person synthetic rho and squared-correlation control; actual-script
tests cover reversed input, missing/duplicate years, zero-wage/rank failure,
zero residual variance and no-clobber export. The zero-variance SQL guard fails
before export, but the current SQL boundary reports only `sql failed` with its
script location, not the guard's detailed reason. The text and JSON source
workflows export identical rows. Prior script effects persist on later failure.
