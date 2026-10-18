# Fishing-mode binary illustration: Cameron–Trivedi (2005), §14.2

This bounded example validates the **classical and unclustered robust logit
and probit coefficient columns of Table 14.2** (p. 465), under the explicit
converged-information conventions below, and probability data underlying
Figure 14.1 (p. 466). It adds a separately validated continuous average partial
derivative with conditional-design delta-method uncertainty for each family. That effect
is an extension, **not** a claimed published table result.

It does not complete the entire illustration: the full cross-model comparison,
remaining author outputs (including robust omnibus Wald tests) and the actual
figure remain required work.
No author data or code is redistributed here.

## Source, sample and design

The authors' [program](https://cameron.econ.ucdavis.edu/mmabook/mma14p1binary.do)
and [Stata 8 output](https://cameron.econ.ucdavis.edu/mmabook/mma14p1binary.txt)
identify §14.2, pp. 464–466, Tables 14.1–14.2 and Figure 14.1. The
[index](https://cameron.econ.ucdavis.edu/mmabook/mmaprograms.html) abbreviates the
page range as 464–465. Data originate in Herriges and Kling (1999),
*Review of Economics and Statistics* 81:62–72.

SHA-256 provenance:

| File | SHA-256 |
| --- | --- |
| `Nldata.asc` | `1fb6f7d5abaf790d4df2c6a13a499ca42c0db4f950d941ba92565c30b4ab6c22` |
| `mma14p1binary.do` | `a443a2988eb737ed8f58dd0537c015a1a639f8de2a8c7bcef41c8a5f49f9cfa5` |
| `mma14p1binary.txt` | `dbf9bb8f00a126269b099090e7acd569725a2cf218275c384901ac13c5ab6d03` |

The ASCII stream has **16 numeric fields per person**, 1,182 people, CRLF line
endings and no final newline. `nldata_to_csv.awk` groups tokens, not physical
lines, preserves decimal tokens and adds an original **1-based `source_row`**.
It rejects malformed/incomplete/empty records; the recipe only publishes a CSV
after successful conversion. The converter does not impute or filter people.

- Keep mode **2 (pier)** or **4 (charter)**: 630 people, 178 pier and 452 charter.
  One source comment reverses these codes; the executable code and output resolve
  them as stated here.
- Outcome `charter = 1(mode == 4)`, predictor `lnrelp = ln(pcharter/ppier)`, intercept.
  Source `pchart` is Stata's abbreviation for `pcharter`. Both the logit and the
  probit script use this identical preparation.
- Preserve source-row order. No additional missing/nonfinite exclusions occur.
  The fit's retained indices `0..629` refer to the prepared 630-row cohort;
  `source_row` identifies each original person in the 1,182-record input.
- Read source decimals and prepare the design in float64. Do not silently emulate
  Stata's float32 import/generated-variable storage. Author **reported rounded**
  results agree; independent R validates the float64 calculation at higher precision.

## Run and independently check

From the repository root, with `curl`, POSIX `awk` and the built Rust binary:

```sh
(
set -eu
cargo build --locked --bin tabdat-explore-rs
repo="$PWD"
work=$(mktemp -d)
curl -fsSL https://cameron.econ.ucdavis.edu/mmabook/Nldata.asc -o "$work/Nldata.asc"
printf '%s  %s\n' \
  1fb6f7d5abaf790d4df2c6a13a499ca42c0db4f950d941ba92565c30b4ab6c22 \
  "$work/Nldata.asc" | shasum -a 256 -c -
awk -f examples/mma14p1binary/nldata_to_csv.awk "$work/Nldata.asc" \
  > "$work/Nldata.csv.tmp" && mv "$work/Nldata.csv.tmp" "$work/Nldata.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json \
  -f "$repo/examples/mma14p1binary/logit.td")
Rscript examples/mma14p1binary/reference_logit.R "$work/Nldata.csv" \
  "$work/R_predictions.csv" "$work/fishing_logit_predictions.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json \
  -f "$repo/examples/mma14p1binary/probit.td")
Rscript examples/mma14p1binary/reference_probit.R "$work/Nldata.csv" \
  "$work/R_probit_predictions.csv" "$work/fishing_probit_predictions.csv"
printf 'Outputs: %s\n' "$work"
)
```

Stop if the checksum/conversion fails. Each script writes its own
`fishing_*_predictions.csv`; each intentionally refuses to overwrite an existing
file, so a replay needs a fresh output directory (copy `Nldata.csv` there). For
a text replay, replace `--json` with `--text`. No arguments enter the persistent
terminal session; the same commands work there. **Bare `predict` retains the
existing xb parser default**; request `, pr` explicitly for probabilities.

R 4.6.1 `stats::glm` is an optional **development-only** reference, no R/Python
is needed by TabDat. Both reference scripts reevaluate the information
matrices at the returned coefficients, independent of any last-iteration
working-weight lag in glm's cached QR covariance; both verify **every
source-row ID**, outcome, design value, probability and linear index, not just
N. The convergence conventions differ by family: for the **logit** the plain
glm stop (deviance change) already sits at the score fixed point; for the
**probit** the deviance is flat near the optimum, so `reference_probit.R`
polishes glm's solution with Newton steps on the analytic observed information
until the gradient norm is < 1e-13 and prints the glm-stop and fixed-point
score norms (on this design: 2.94579e-07 → 2.57012e-14, one polishing step).

| Logit quantity | Independent R reference |
| --- | ---: |
| Intercept / slope | 2.0531249423051365 / -1.8225304317903031 |
| Model-based SE, intercept / slope | 0.16893067646375609 / 0.14456807108042660 |
| Log likelihood / null log likelihood | -206.82697122830305 / -375.06166897675286 |
| LR chi-square(1) / p | 336.46939549689961 / 3.7478749624517373e-75 |
| McFadden pseudo-R² | 0.44855209599911783 |
| Mean / min / max probability | 0.7174603174603175 / 0.004719638213908227 / 0.9974745605784411 |
| Continuous AME / delta SE | -0.18392143550221252 / 0.0052985086445694511 |
| AME normal 95% interval | [-0.19430632161734276, -0.17353654938708227] |
| AME normal two-sided p | 5.2063951837379612e-264 |

Native replay matches coefficients, covariance, SEs, intervals and effects at
`1e-10` absolute; log likelihoods/LR at `1e-8` absolute; all-row probabilities and
indices at `1e-10` absolute; test probabilities at `1e-10` **relative** plus
`f64::MIN_POSITIVE` (so a fabricated zero cannot pass for the tiny AME tail).
Observed largest logit probability/index errors were approximately `2.0e-15`/`4.2e-14`.

| Probit quantity | Independent R reference |
| --- | ---: |
| Intercept / slope | 1.1943604331481330 / -1.0555145307066702 |
| Model-based SE, intercept / slope | 0.089508361628277577 / 0.076115584870155517 |
| Log likelihood / null log likelihood | -204.4108645034095 / -375.06166897675286 |
| LR chi-square(1) / p | 341.30160894668671 / 3.3220403402590648e-76 |
| McFadden pseudo-R² | 0.45499398789248358 |
| Mean / min / max probability | 0.72018995284383147126 / 0.00098774792299152303 / 0.99973774175233098838 |
| Continuous AME / delta SE | -0.18675505365724743 / 0.0054545071552692895 |
| AME normal 95% interval | [-0.19744569123499126, -0.1760644160795036] |
| AME normal two-sided p | 6.4315448068460033e-257 |

The probit replay satisfies the same tolerance policy, with observed largest
probability/index errors of approximately `5.0e-14`/`3.0e-13`. The committed
synthetic references in `crates/tabdat-stats` and `crates/tabdat-runtime` use
`1e-10` absolute for moderate outputs.
`sh examples/mma14p1binary/test_converter.sh` additionally checks CRLF/final-record
preservation and malformed/incomplete/empty-input failure without publishing over
an existing destination.

## Unclustered robust covariance and reproducible reference

`robust_logit.td` and `robust_probit.td` repeat the exact preparation above with
`, robust`. They preserve the same coefficients, sample and predictions. The
covariance is `N/(N-1) * B * sum(s_i s_i') * B`, where `s_i` is the individual
score and `B` is inverse observed information at the converged fit (equal to
inverse Fisher for logit). The [Stata ML correction](https://www.stata.com/manuals/u20.pdf)
(§20.22.2) is **not** OLS HC1's N/(N-K). Independent observations are assumed;
this is not cluster/survey inference, a causal estimand, or protection against a
wrong conditional-mean interpretation.

Coefficient and retained-X delta-AME inference remain asymptotic normal. JSON
labels both fit and margins `robust_sandwich_n_over_n_minus_1`. The LR statistic,
restriction count and pseudo-R² remain descriptive; **LR p-value and reference
are null**. No robust omnibus Wald test is supplied.

With `work` set to a directory containing the checksum-verified `Nldata.csv`
acquired above, run from the repository root (fresh robust output filenames):

```sh
repo="$PWD"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json \
  -f "$repo/examples/mma14p1binary/robust_logit.td" > robust_logit.jsonl)
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json \
  -f "$repo/examples/mma14p1binary/robust_probit.td" > robust_probit.jsonl)
Rscript crates/tabdat-stats/tests/fixtures/binary_robust_reference.R \
  "$work/Nldata.csv" > "$work/robust_reference.csv"
python3 examples/mma14p1binary/check_robust_reference.py \
  "$work/robust_reference.csv" "$work/robust_logit.jsonl" "$work/robust_probit.jsonl"
Rscript examples/mma14p1binary/reference_logit.R "$work/Nldata.csv" \
  "$work/R_robust_logit.csv" "$work/fishing_robust_logit_predictions.csv"
Rscript examples/mma14p1binary/reference_probit.R "$work/Nldata.csv" \
  "$work/R_robust_probit.csv" "$work/fishing_robust_probit_predictions.csv"
```

The R 4.6.1 script independently computes the scores, converged information,
sandwich and full delta gradients; it also generates the committed synthetic
CSV reference when run without a data argument. The standard-library Python
checker is development-only and compares all **44** coefficient/covariance/AME
quantities; R's existing prediction checkers verify all 630 source-row IDs,
outcomes, designs, probabilities and indices. Neither tool is needed at runtime.
Tolerances remain 1e-10 absolute for moderate outputs and 1e-10 relative plus
`f64::MIN_POSITIVE` for test probabilities. Native synthetic tests additionally
cover both families with/without intercept, one/two predictors, intercept-only
models, predictor units, invalid outcomes and unsupported cluster/weight designs.

| Robust quantity | Independent R fixed-point reference |
| --- | ---: |
| Logit intercept / slope SE | 0.147347754734682 / 0.130755606121941 |
| Logit continuous AME SE | 0.00570499760609184 |
| Probit intercept / slope SE | 0.0794501824989468 / 0.0692948032742171 |
| Probit continuous AME SE | 0.00568139267322038 |

Rounded Table 14.2 coefficient/z entries agree. This is **not** an exact replay
of Stata 8's stored information: its reported probit robust SEs are
0.0794429/0.0692881, versus the converged-information values above. As for the
classical probit reference, the target is the independently validated fixed
point, not a lagged Hessian; sandwich bread enters twice. Float64 preparation
also does not emulate the author's float32 storage. These convention differences
are explicit; no reference tolerance was widened to hide them. Robust support
advances this subset, not all author output or the whole binary family.

## Interpretation, numerical limits and remaining work

Both default fits are unweighted independent-Bernoulli maximum likelihood,
normal coefficient/AME inference and asymptotic chi-square LR inference,
**without N-K covariance scaling**. The robust variants follow the sandwich
and suppressed-LR-inference contract above. The default logit reports inverse-Fisher
covariance; the default probit reports inverse per-observation observed information at the
converged fit, computed from the converged QR. With an intercept the LR null
estimates a constant probability; without it the zero-coefficient null is
probability 0.5.

Stata 8 computes its probit SEs from the penultimate-iterate Hessian, so its
classical-column SEs can differ from these converged fixed-point values in
trailing digits. This example does not replicate that lagged-iterate
convention; the reference standard is the R fixed point described above.

`estat margins` averages the family-specific continuous derivative over the
**retained training design**: logit `beta_j*p*(1-p)`, probit
`beta_j*phi(beta_0 + beta'x)`. Its full delta gradient includes
intercept/slopes' covariance, conditional on observed X. Current-data edits do
not change this estimand. The effect is a
predictor-column derivative, not an automatic discrete change, interaction chain
rule, or causal effect. Here the negative AME is probability units per unit of
**log relative price**, not per price unit or an exact finite-change prediction.

Fitting requires both outcome classes with exact 0/1 values on the complete
finite sample. Native Decimal outcomes are validated on their exact integer
payload/scale before conversion: nearby values must not round into a binary one.
Domain rejection follows complete-case selection, so an otherwise excluded row
still does not enter the model. Scaled Newton/QR uses stable likelihood/curvature, backtracking,
100 information evaluations and absolute `1e-10` score **and full undamped step**
bounds in unit-column-norm coordinates. A flat likelihood/small score alone must
not publish a separated fit. Combined outlier/separating-contrast regressions
cover a false-convergence trap found in independent review. This is not an
analytic separation classifier; numerical singularity/nonconvergence is an
explicit error, including extreme finite designs without adequate absolute
numerical resolution. Identified models with saturated outlier probabilities
are not rejected merely because probabilities round to zero/one.

Cluster/weighted/survey forms, categorical/factor/interactions and
discrete changes with uncertainty, prediction intervals, general `lincom`/`test`
after either family and rendered graphics are not delivered by this slice. They
remain required where specified by the roadmap/illustrations; neither this
example nor passing synthetic tests establish binary-family or full-book
completion.
