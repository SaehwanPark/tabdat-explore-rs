# Chapter21 pooled-panel GLS companion source inventory

This source-scoped inventory pins the March2005 actual publisher program
[`mma21p4pangls.do`](https://cameron.econ.ucdavis.edu/mmabook/mma21p4pangls.do)
SHA256 `dde40722e3de67cb2b447792e28ffc14e7b6144ad6dae89ecb23f0e0fa694b6f`
and [author output](https://cameron.econ.ucdavis.edu/mmabook/mma21p4pangls.txt)
SHA256 `eac03f7c644eb141819c31d6d39617a5e6616e979db5effecb8c7f332d34465b`.
The overview's `mma21p4panpangls.do` is a stale HTTP404 alias, not this source.
The header says Table21.6 p725 while executable-section comments/report say
Table21.7; source labels conflict, and this inventory does not independently
resolve the printed textbook table number. No publisher data/program is bundled.

Common sample: `MOM.dat` 532 people x10 years1979–1988, N5320; unweighted
Gaussian identity-link `lnhr ~ 1 + lnwg`, no missing case deletion. The source
fits model-working and semi-robust unit-cluster covariance versions for each
working correlation; it stores each, then reports coefficients/SEs in two
comparative tables. Those are population-averaged working models, not causal
estimates, native FE, or automatically correct covariance assumptions.

| ID / source lines | Specified analysis and post-estimation | Current status / required work |
| --- | --- | --- |
| P4-LOAD / 54–70 | Read/describe/summarize MOM, define B500 | Shared pinned [source preparation](README.md) and scripts cover MOM; source-specific report below. |
| P4-IND / 74–81 | Gaussian independent-working `xtgee`, working and unit-semi-robust covariance, normal z/95 and chi-square | Validated bounded [independent workflow](independent_gee.md): all5,320 source/prediction rows, both full matrices and532 unit scores independently checked; RSS/N, G/(G-1) and normal/chi-square distinct from OLS t/CR1. Float32-input hypothesis matches24 printed facts; historical precision not established. |
| P4-IND-BOOT / 82–85 | Reset seed10001; whole-person B500 coefficients/SE | Related [pooled person bootstrap](pooled_bootstrap.md) validates all500 native-stream refits, but author log says requested500 and displays only72 `Reps` for each coefficient. No historical RNG/draw or failed-replicate explanation established; do not claim exact source bootstrap parity. |
| P4-EXCH / 87–94 | Exchangeable-working GEE, working and unit-semi-robust covariance | Validated bounded [P4 exchangeable report](exchangeable_p4.md): existing legacy-Pearson fit/working covariance and full G/(G-1) unit sandwich, normal inference, all5,320 source/prediction rows and532 unit scores checked against R gee/dense matrices;25 P4 logged facts match printed precision. Raw sandwich and retained pooled OLS remain distinct; not automatically RE-GLS. |
| P4-EXCH-BOOT / 95–98 | Seed10001, whole-person exchangeable GEE B500, refitting working correlation | Open; source log shows requested500 but72 `Reps` per coefficient. Refit working correlation per draw and handle duplicate resampled IDs explicitly; do not substitute fixed theta or infer cause of historical72. |
| P4-AR1 / 100–107 | Identity Gaussian AR(1) working GEE with explicit id/year, working and unit-semi-robust inference | Open; validate exact author estimation/scale/correlation/iteration and numerical failure semantics against independent reference before claiming it. |
| P4-AR1-BOOT / 108–111 | Seed10001, whole-person AR1 B500; bootstrap command omits `t(year)` present in original fit | Open; log reports500 coefficient replications. Source-as-written time omission, duplicate IDs and refit correlations need explicit decisions/evidence; do not silently substitute fixed correlation. |
| P4-UNSTRUCT / 113–120 | Homoskedastic unstructured working correlation, working and unit-semi-robust inference | Open; validate full working matrix/positive definiteness, sample and convergence independently. |
| P4-UNSTRUCT-BOOT / 121–126 | Bootstrap command is commented out | Not executable in source. It is a documented unresolved requirement, not successful B500 evidence. |
| P4-REPORT / 128–140 | Eight named coefficient/SE fits in two tables; three saved bootstrap SE vectors and one missing | Partial: existing compositions do not provide whole mixed comparison. Source attempts `matrix list unstrbootse` despite commenting out its definition; the author log ends with `matrix unstrbootse not found`, `r(111)`. Never claim a successful complete four-bootstrap report. |

References are source/log observations, not proof of native behavior or exact
historical internal precision. Complete source/chapter/book inventory, independent
numerical evidence and all deferred entries remain required.
