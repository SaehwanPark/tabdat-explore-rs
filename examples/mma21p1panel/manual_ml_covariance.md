# Manual fixed-ML-theta auxiliary covariance comparison — WIP contract

Required P2-ML-IID/CL, source288–293 of the [manual inventory](manual_inventory.md).
Risk: high (statistical meaning and retained state). Original MOM N5320/G532/T10;
finite bounded complete ten-year people and exact original keys, same input bounds as
[GLS comparison](manual_gls_covariance.md). Fit native balanced Gaussian RE-ML,
retain population predictions, freeze **original ML theta**, construct quasi y/x,
fit named ordinary and person-cluster auxiliary OLS, report both coefficient
intervals/covariances and export ordered source/means/theta/population/quasi
predictions. This is not joint observed-information covariance, component-refitting
bootstrap, corrected ML-HC1, source-labelled GLS-HC1 or a complete mixed table.
No shared Rust/API/dependency change is intended.

Candidate route: recover original ML component ratio from native fitted population
coefficients and balanced Gaussian likelihood score equations on the identical
sample, guard interior/boundary and certify against native reported components;
otherwise stop and select a demonstrably correct existing route rather than silently
substituting GLS theta. Determine concrete tolerance from independent numerical
behavior, not an unexplained widening. Keep ML normal inference distinct from
auxiliary Student-t/cluster working references and transformed-intercept scale.

Acceptance before implementation:
1. Independent R/nlme or direct likelihood/QR reference checks original native ML
   components/theta and full auxiliary IID/CR1 covariance, both coefficient
   estimates/intervals/inference DF and all source/design/prediction cells; exact
   raw membership/order, versioned reference, justified fixed tolerances.
2. Two independent small interleaved controls cover positive ML unit variance and
   constrained zero boundary, both text/JSON modes and fixed original predictions;
   explicitly demonstrate theta is ML rather than Swamy–Arora where they differ.
3. Duplicate/fractional keys, missing/incomplete sample, rank/variance failures and
   existing-export collision fail without overwriting source or export. No whole-
   script rollback claim. Test retained last cluster inference after reports/predict.
4. Named estimates identify auxiliary models and exact sample/cluster ownership;
   reproduce original with fresh paths and development-only R, document limits.

Inspect existing native ML and profile implementations, previous GLS reference and
controls before implementing. Start with focused new tests, then root package suite;
update SPEC, roadmap and ledger only after evidence, review diff and CI before merge.
Implement exactly this plan; if it conflicts with the codebase, stop and report the
conflict instead of improvising. Report files, tests, deviations and unresolved risks.
