# Chapter 21 classical FE–RE Hausman comparison

This bounded §21.4.3 illustration uses the original 5,320-row/532-person MOM
panel, lnhr outcome, lnwg slope and intercept. `classical_hausman.td` checks raw
keys and source-row uniqueness, fits balanced Swamy–Arora RE-GLS, runs
`estat hausman`, reports current population predictions and residual summaries,
and exports all original rows with fitted values and residuals. It runs in text
or CLI-v1 JSON. See the [data and preparation instructions](README.md).

`estat hausman` uses the original owned joint-finite RE design, not current data,
to refit homoskedastic FE and RE. For d=b_FE−b_RE and v=V_FE−V_RE it reports
d²/v with a chi-square(1) upper tail, only if v is finite, positive and larger
than 128 machine epsilons times the larger variance. A negative/zero or
cancellation-dominated v is an error: no pseudoinverse, ridge or absolute
value. Estimates, both variances, sample/group counts, df, statistic, p-value
and assumptions are reported. The diagnostic does not mutate the fit or data.

Only a balanced model-based nonclustered Swamy–Arora fit with an intercept and
one slope is supported. ML, robust/clustered, unbalanced and multiple-slope fits
are rejected. Older serialized RE states without the owned design are readable;
refit the supported RE model before this diagnostic. The unit variance may be
at zero; the covariance-difference guard still applies. No named-pair or general
multi-parameter test, covariance rescaling or automatic model selection is
provided. The conventional test assumes RE efficiency under the null; remaining
within-person correlation can invalidate it, as the author warns. It is not
cluster robust, and failure to reject does not establish exogeneity. The
[separate robust auxiliary comparison](README.md#robust-auxiliary-hausman-comparison--2143-p719)
is a distinct method, not a robust covariance substituted here.

## Reference and reproduction

For the pinned `MOM.dat` SHA-256
`2fe194d6113e5790a008adcac1ca168be5531924b40349a810a6683aef800d77`, native
float64 and R 4.6.1/plm 2.6.7 agree on FE slope 0.1676754886293, RE slope
0.1193322439406, variance difference 0.000170266967403, chi-square(1)
13.7259113889 and p 0.0002115161044. The historical author log rounds the
statistic and p-value to 13.73 and 0.0002. This is not a claim of exact Stata
float32 or whole-table parity.

After downloading the author data under its terms, from a fresh working directory:

```sh
root=/absolute/path/to/tabdat-explore-rs
work=/absolute/path/to/empty/work-directory
cp /path/to/MOM.dat "$work/MOM.dat"
(cd "$root" && cargo build --locked --bin tabdat-explore-rs)
"$root/examples/mma21p1panel/prepare.sh" "$work/MOM.dat" "$work/MOM.csv"
(cd "$work" && "$root/target/debug/tabdat-explore-rs" --json -f "$root/examples/mma21p1panel/classical_hausman.td" > result.jsonl)
Rscript "$root/examples/mma21p1panel/classical_hausman_reference.R" "$work/MOM.dat" "$work/hausman-reference"
Rscript "$root/examples/mma21p1panel/random_effects_reference.R" "$work/MOM.dat" "$work/re-reference"
python3 "$root/examples/mma21p1panel/check_classical_hausman.py" "$work" "$work/hausman-reference" "$work/re-reference" "$work/MOM.dat"
```

The checker verifies all original input values, 12 independent plm Hausman
quantities and 58,520 exported original/design/prediction cells at absolute plus
relative tolerance 1e-9. A deterministic synthetic 40-person reference in
`classical_hausman_synthetic_reference.R` and Rust tests checks all 12 quantities
at 1e-10 absolute plus relative tolerance, as well as rejected nonpositive and
cancellation-dominated differences. A separate negative-difference fixture is
rejected rather than adopting `plm::phtest`'s positive statistic for that case.
The original data are not bundled; R and Python are development-only audit tools.
