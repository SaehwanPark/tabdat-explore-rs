# Independent review: bounded linear-regression `test`

Three independent read-only review passes examined runtime behavior, numerical routines, and contract/evidence accuracy. Their findings and fixes are recorded here for the final PR head.

## Findings addressed

- Added a pinned intercept restriction (`test intercept = 0`) with exact result fields and a direct statsmodels/SciPy reference.
- Matched the pinned executor's state transition by clearing a previous fit before a new regression attempt validates; a failed attempt leaves active data intact, clears model state, and makes a following `test` return `no active estimation results found`.
- Made F-survival input construction stable in log space and handled underflowed beta arguments and degrees-of-freedom halves. Added tests for positive extreme tails, underflowed x with a unit survival result, tiny positive degrees of freedom, and the minimum subnormal df limit. The extreme-tail reference assertion now requires a positive result and uses relative error.
- Normalized the restriction difference before evaluating the Wald quadratic form, so a representable statistic survives when the raw difference square underflows.
- Added `reference_probe.py` and the preserved `03-reference-output.json`; this now reproduces the pinned executor and statsmodels/SciPy values, the failed-regression state probe, and exact labels such as `x2 = 2`.

## Follow-up review

Reviewers rechecked the runtime state transition, intercept coverage, scaled Wald calculation, incomplete-beta tail, subnormal df handling, and generated reference output. They reported no remaining concrete findings.

## Focused verification

- `cargo test --locked -p tabdat-runtime --test test_runtime_contract`: 5 passed.
- `cargo test --locked -p tabdat-stats --test f_distribution_contract`: 3 passed.
- `cargo test --locked -p tabdat-stats --test linear_combination_inference_contract`: 6 passed.
- Pinned Python `tests/test_statistical_testing.py`: 8 passed.
