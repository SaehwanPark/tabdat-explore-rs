# NLS model-variance sandwich for Table 5.7

This source-data workflow reproduces the auxiliary variance calculation in
[mma05p2nls.do](https://cameron.econ.ucdavis.edu/mmabook/mma05p2nls.do),
§5.9.2–3, using the same generated 10,000 rows and unweighted NLS fit as the
[unweighted workflow](README.md). It computes the full 2×2 covariance and two
standard errors under the specified exponential variance model. It does not
change the NLS estimator or add a general variance-model API.

For `mu_i=exp(-a-b*x_i)`, parameter order `(a,b)`, the conditional mean is
assumed correctly specified and `Var(y_i|x_i)=mu_i²`. The variance is *not*
known numerically: the script substitutes the fitted means. Let
`J_i=(-mu_i,-x_i*mu_i)`, `B=sum(J_i J_i')` and
`M=sum(mu_i² J_i J_i')`. The reported plug-in covariance is `B^-1 M B^-1`,
with no HC or degrees-of-freedom correction, matching the author's calculation.
It assumes independent rows and the stated variance model; it is not the
residual-based HC1 covariance and is not robust to a misspecified variance
function or conditional mean. The NLS fit shown before it separately reports
its usual homoskedastic working covariance and approximate-t inference. This
workflow does not claim exact t inference for the plug-in standard errors and
does not construct intervals or p-values from them.

Preflight requires 3–10,000 complete finite observations, positive y,
nonconstant x and unique dense source ordinals. The existing fit uses explicit
starts (1,0) and its rank, convergence and positive-residual-variance rules.
Fitted mean powers, moment sums and covariance must be positive where required
and finite. The script rejects `det(B)<=1e-12*B00*B11` instead of inverting a
numerically ill-conditioned matrix, and rejects nonpositive covariance
determinants. These explicit computational limits may reject an otherwise
identified design. Export is no-clobber; script commands execute sequentially,
so the workflow is not a transaction.

Independent R 4.6.1 profiles the amplitude and solves the NLS slope score, then
forms `J`, `B`, `M` and `B^-1 M B^-1` with matrix operations. The pinned source
ASCII SHA256 is `460f5d731f1848c8695d78a1fbf0db348d0daced4b8b5b0c2734bffa20b3e31c`;
the preparation tool preserves all source ordinals and numeric tokens. The audit
checks all 10,000 prepared rows, fit sample, fitted means/residuals, matrix
entries and covariance/SEs at `1e-11+1e-10*abs(reference)`. Native and R SEs
are (0.2109705576, 0.0879810936). The author's historical output prints
(0.21097066, 0.08798113), which agrees at four decimals but not all printed
decimals; the cause is not established and no tolerance was widened to claim
exact parity. Text/JSON CSVs agree byte for byte. A three-row shifted-design
control has closed-form nonzero off-diagonal covariance, and input failure and
no-clobber checks exercise the actual script.

From a fresh output directory:

```sh
python3 /path/to/repo/examples/mma05p2nls/prepare.py /path/to/mma05data.asc nls.csv
/path/to/tabdat-explore-rs --json -f /path/to/repo/examples/mma05p2nls/model_variance.td > result.jsonl
Rscript /path/to/repo/examples/mma05p2nls/model_variance_reference.R /path/to/mma05data.asc reference
python3 /path/to/repo/examples/mma05p2nls/check_model_variance.py /path/to/mma05data.asc result.jsonl nls_modelvar.csv reference
```

This is one auxiliary source calculation, not new general NLS inference,
weighted/cluster/survey covariance, a complete Table 5.7 or validation of other
mean and variance models.
