# Ordinary auxiliary Hausman diagnostic — WIP contract

P2-HM-IID source314–316, §21.4.3 pp717–719 in [manual inventory](manual_inventory.md).
Use existing robust diagnostic's declared balanced joint-finite sample/Swamy–Arora
preparation: G>3/T>=2, exact bounded original keys, positive within error variance,
identified within/between designs and finite theta[0,1). Original N5320/G532/T10.
OLS quasi_y ~ intercept + quasi_x + within_x, K3; covariance RSS/(N-3)*(X'X)^-1,
slope contrast delta=within slope minus Between slope, H=(delta/SE)^2.
This is not direct FE-minus-RE, covariance subtraction, robust or ML inference.
Native lincom t/test F(1,N-3) remain honestly labelled working finite-sample
references; asymptotic chi-square(1) requires null RE efficiency/exogeneity and
regularity. No production chi-square-tail API is added or silently substituted.

Acceptance before implementation:
1. Independent R/plm theta + full auxiliary QR checks39 coefficient/full3x3
   covariance/fit/interval/F/DF fields and all18 original source/transform/
   prediction columns, explicit within/Between coefficient equivalence. abs1e-9,
   p-relative1e-8/exact underflow0; source fields exact. Original working-F and
   asymptotic chi-square tails recorded separately in notes.
2. Reuse independent dyadic16-person interior/theta0 controls, freeze39-field/
   prediction references, both CLI modes and owned CI/F after prediction/summarize.
3. Duplicate/fractional keys, unbalanced joint sample, within/between rank or
   error-component failures do not export; existing export/source owner preserved.
   No whole-script rollback. Existing robust workflow/reference unchanged.

New standalone script/reference/auditor/focused actual-script tests only; no
production Rust/API/dependency change. Native generic F output is not whole
Hausman-command parity or exact finite-sample validity with estimated theta.
Full combined theta/H report, fixed-ML work, companion inventories, full mixed
comparisons and book coverage remain required.
