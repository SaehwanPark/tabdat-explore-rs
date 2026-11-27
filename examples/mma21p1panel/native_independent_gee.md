# Native independent Gaussian identity-link GEE

## Estimator and source

This is the native `xtgee` command for the Chapter21 P4 independent-working
design: `lnhr` on an intercept and `lnwg`, grouped by person. The original
publisher `MOM.dat` SHA256 is
`2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77`; its
5,320 rows contain 532 complete people observed for ten years each. The source
program and historical output are linked from the [GLS inventory](gls_inventory.md).

The command is:

```text
xtgee lnhr lnwg, family(gaussian) link(identity) corr(independent) id(id)
```

It fits the Gaussian identity-link marginal mean on joint-finite rows. The
working correlation is fixed at `I`, so coefficients equal OLS on that sample.
For design matrix `X`, residuals `e`, and `N` rows, working scale is
`RSS/N` and working covariance is `(RSS/N)(X'X)^(-1)`. The separately reported
raw group-score sandwich is

```text
(X'X)^(-1) sum_g (X_g' e_g)(X_g' e_g)' (X'X)^(-1)
```

There is no `G/(G-1)` or HC1 multiplier. Coefficient inference is asymptotic
normal; the one-slope model test is chi-square(1). Independent-working GEE
allows unbalanced groups and singletons, subject to an identified design,
positive scale, `G > K`, and an estimable group sandwich. No time key is needed.
The native result owns the GEE fit for subsequent `predict`, JSON/text reporting,
and explicitly selected working or sandwich named estimates.

On the original MOM sample, the Base-R reference gives slope `0.0827435452801`,
working SE `0.00912342107754`, and raw sandwich SE `0.0292408738783`. The author
log prints slope `0.0827436`, working SE `0.0091234`, and robust SE
`0.0292684`. Multiplying the native raw sandwich covariance by `532/531`
(equivalently its SE by `sqrt(532/531)`) gives the `G/(G-1)` convention and
matches the logged robust SE to its printed precision. The older
[SQL composition](independent_gee.md) reports that adjusted convention while
retaining pooled OLS as its owned fit; this native command instead retains GEE
state and reports the raw covariance. This correction relationship does not
claim exact Stata output parity.

## Independent evidence

[`native_independent_gee_reference.R`](native_independent_gee_reference.R)
uses Base R 4.6.2 `lm.fit` QR, QR-based bread and independently grouped unit
scores. [`check_native_independent_gee.py`](check_native_independent_gee.py)
checks the native JSON result, both full covariance matrices, normal/chi-square
inference, named-estimate table and all source-ordered predictions against that
reference. It also verifies the raw `MOM.dat` checksum and exact identity/order
of all 42,560 original data values. Absolute numerical tolerance is `1e-9`;
nonzero tail probabilities use relative tolerance `1e-8`, and underflow must
match exact zero.

The native reference checks 8 covariance cells, 28 coefficient/inference
values, 37,240 prediction values over 5,320 rows, and all 42,560 raw source
values. The runtime fixture independently covers unequal group sizes and a
singleton; statistical tests cover synthetic QR agreement, rank deficiency and
numerically zero scale. Parser and serde tests preserve the existing
exchangeable wire while round-tripping the independent selector. Runtime
contracts cover current-row mean/residual prediction, named working/sandwich
reports and failed-fit state preservation. Fresh CLI runs in JSON and text modes
both produce the documented independent method and covariance labels.

Reproduce with the publisher data available locally and a fresh work directory:

```sh
root="$PWD"; work=/path/to/fresh/work
mkdir -p "$work"
sh "$root/examples/mma21p1panel/prepare.sh" /path/to/MOM.dat "$work/MOM.csv"
(cd "$work" && "$root/target/debug/tabdat-explore-rs" --json -f "$root/examples/mma21p1panel/native_independent_gee.td" > output.jsonl)
Rscript "$root/examples/mma21p1panel/native_independent_gee_reference.R" /path/to/MOM.dat "$work/reference"
python3 "$root/examples/mma21p1panel/check_native_independent_gee.py" "$work/output.jsonl" "$work/native_independent_gee_predictions.csv" "$work/reference" /path/to/MOM.dat
```

Use another fresh work directory for `--text`, because exports refuse to
overwrite existing files.

## Scope limits

Independent Poisson GEE, weights, time-varying correlation, bootstrap, a
combined four-correlation report, and exact Stata 8 parity are not included.
The selected period-standardized unstructured Gaussian estimator has its own
[formula and evidence](unstructured_gee.md); this independent result does not
validate or change it. Chapter21 and book coverage remain partial.
