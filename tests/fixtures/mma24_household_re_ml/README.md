# Chapter 24 household RE-ML synthetic references

All three fixtures use 180 inspectable source rows, one missing outcome, 179
complete cases and 16 unbalanced communes including one singleton. The
`interior` input is `../mma24_household_re/interior_input.csv`; both full and
restricted-null unit variances are positive. The `restricted_boundary` input is
`../mma24_household_re/boundary_input.csv`: the full unit variance is positive,
but the null unit variance is zero, so the ordinary slope-LR reference is
withheld. `zero_boundary_input.csv` is generated from the interior input by
`examples/mma24p1olscluster/household_re_ml_boundary_synthetic.R`: demeaned,
orthogonal residuals place the full unit-variance optimum at zero. The native
boundary fit exposes constrained information, a unit LR of zero/p=1 and no
regular unit-component Wald uncertainty; it also withholds the ordinary slope-LR
reference because the restricted null is on the boundary.

`*_reference.csv` gives 210, 209 and 187 scalar quantities, respectively,
including full joint covariance and component uncertainty when defined;
`*_predictions.csv` has every complete case, source field, population xb and
residual. The independent R 4.6.2 profile likelihood uses `nlme` 3.1.171 (or OLS
at the boundary), analytic observed information cross-checked against a
finite-difference Hessian, and the same explicit sample and six-column design.
Fixtures are synthetic and do not redistribute publisher data.

Recreate each reference in a fresh directory:

```sh
Rscript examples/mma24p1olscluster/household_re_ml_reference.R /path/to/input.csv /path/to/fresh-reference --csv
```

`cargo test --locked --test mma24_household_re_ml_workflow` checks every fixture
scalar and prediction, text/JSON summaries, source order, retained fit, source
integrity, no-clobber export and invalid key/rank/group inputs. Absolute 1e-9
and nonzero-p relative 1e-8 tolerances apply; independently computed interior
profile scores must each have absolute value <1e-8 because the native root uses
finite eta bisection rather than exactly reproducing R's root. Boundary score is
compared directly. No production R dependency is introduced.
