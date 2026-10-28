# Chapter 22 wage Shea relevance diagnostic — work in progress

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

Before completion:

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
