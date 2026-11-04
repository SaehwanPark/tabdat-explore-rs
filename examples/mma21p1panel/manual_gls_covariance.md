# Manual fixed-GLS-theta covariance comparison

[manual_gls_covariance.td](manual_gls_covariance.td) delivers P2-GLS-IID/CL/HC1,
manual source264–269/278–279 in the [pinned inventory](manual_inventory.md).
It loads/preflights MOM → fits native population RE-GLS/predicts → constructs the
original fixed-theta quasi design → fits three auxiliary covariance choices →
reports named models/both coefficient intervals → exports raw/component/mean/
population/quasi predictions. Existing primitives only; no production Rust,
parser/API/schema or dependency change. Not a complete mixed source table,
component-refitting bootstrap, marginal ML information or chapter/book closure.

## Design and covariance contract

Original N5320/G532/T10; supported3..532 complete ten-year people, unique exact
source_row/id/year in ranges1..5320/1..532/1979..1988, finite abs(lnhr/lnwg)<=20,
identified within/between wage designs, positive within error variance and
finite theta in[0,1), plus existing rank/work limits. Missing/incomplete model
samples fail instead of deleting rows. No weights/extra slopes/time effects.
The converter prefixes physical input lines; exports retain all nine raw/key
columns exactly and are ordered by source_row. Native population predictions
are made before changing active design; final quasi predictions belong to the
last clustered auxiliary OLS, not a restored native RE model.

Balanced Swamy–Arora components are computed **once**, independently of the
covariance choices: e=RSS_within/(N-G-1), raw_u=RSS_between/(G-2)-e/10,
u=max(raw_u,0), theta=1-sqrt(e/(e+10u)). Negative raw_u is recorded, not hidden
or rejected; valid theta0 is retained. No refitting components per covariance
or replication. Quasi y/x are original y/x minus theta times person means.

For X=[1,quasi_x], e_q=quasi_y-X*b and B=(X'X)^-1:

| Auxiliary model | Covariance | Fit residual DF | Coefficient inference DF |
| --- | --- | --- | --- |
| p2_gls_iid_working | RSS_quasi/(N-2)*B |5318|5318|
| p2_gls_hc1_working | N/(N-2)*B*sum_i(score_i score_i')*B |5318|5318|
| p2_gls_person_cluster | G/(G-1)*(N-1)/(N-2)*B*sum_g(s_g s_g')*B |5318|531|

Scores are X_i*e_qi, summed per person for s_g. **Do not substitute the within
component e for RSS_quasi/(N-2).** Native RE population coefficients use normal
inference; auxiliary coefficients use Student-t, with named
`inference_degrees_of_freedom`/lincom `df_residual` carrying G-1 for cluster mode.
Generic fit `df_residual` remains N-2. These are working/large-sample references,
not exact finite-sample/nominal coverage when theta is estimated; HC1 is not a
within-person robust covariance. CR1 assumes independent people/adequate cluster
asymptotics. No general random-effects adequacy/causal or variance-component
inference is supplied by this comparison.

Quasi-intercept=(1-theta)*population intercept; slope is unchanged. With
D=diag(1-theta,1), auxiliary IID covariance equals D*V_native_population*D.
The auditor checks both coefficient and full covariance scaling, not just slope
agreement. Population xb=alpha+beta*x; population residual=y-xb includes the
unobserved unit component, not a BLUP/conditional residual. Quasi residual is
population residual minus theta times its person mean. Both quasi
coefficients/SE/t/p/95CI are validated. This is not the
zero-by-construction FE auxiliary intercept or a joint-ML covariance. No full
joint covariance inversion is attempted: boundary controls can have singular
score covariance while individual coefficient contrasts remain usable under
their stated working interpretation. Both final cluster intervals execute
**after** named reports, SQL notes, prediction columns and summarize, protecting
owned covariance/DF. Exact retained sample indices/rowwise cluster labels and
copied named-report provenance are checked. Native RE snapshots/full mixed
comparison remain unsupported, not silently replaced by auxiliary reports.

## Reference and failure evidence

[R4.6.2/plm2.6.7 reference](manual_gls_covariance_reference.R) independently uses
raw within/between means/QR, plm Swamy–Arora, separate quasi QR and explicit row/
person score products. Original MOM validates120 numerical fields:30 native
normal/component/Wald quantities plus90 auxiliary full-matrix/both-coefficient/
interval/DF quantities. All122,360 exported cells are compared, including47,880
exact original raw/key cells. [Standalone auditor](check_manual_gls_covariance.py)
also verifies native/quasi scaling, named samples/provenance, inference notes,
last-owned interval ordering and successful final export; an old CSV is not
proof of completed execution. Thirteen forged normal-CI/covariance/DF/sample/
group/order/raw/population/transform/note/export/boundary traces are rejected.
The original raw hash/source code/log line pins
remain in the inventory. No publisher data/program is redistributed.

Original theta0.5847092377438329 and raw_u0.026000700300867485; native slope
0.11933224394062582. Quasi slope SEs are0.013631220623790269 (IID),
0.04268967775979778 (HC1),0.051401584845370704 (person CR1).
Fixed tolerances: abs1e-9; p relative1e-8/exact underflow0. No widened tolerances.

[Deterministic interleaved16-person controls](manual_gls_covariance_synthetic.py)
cover positive raw_u/theta and raw_u=-0.02786583533653846 truncated to u0/theta0,
with positive ordinary error and coefficient score variances. Frozen independent
references in [test fixtures](../../tests/fixtures/mma21_manual_gls_covariance/)
are consumed by [three actual-script tests](../../tests/mma21_manual_gls_covariance_workflow.rs):
two120-field/3680-cell controls, text/JSON prediction-byte parity, exact registry
samples/groups, late-owned inference and export/source-owner collision. Seven
invalid duplicate/missing/incomplete/fractional-key/within/between/degenerate-error
variants fail before export and preserve source bytes. Earlier commands are not
rolled back as a whole session.

**Reference IO lesson:** the existing [R decimal-parser warning](../mma17p4duration/README.md)
recurred on R4.6.2: `as.numeric('7.6857223510742188')` gave hex
`0x1.ebe2e00000001p+2`, versus the intended/native/Python `0x1.ebe2ep+2`.
Shortest decimal text was not a general remedy. Controls use a coarse exact
1/256 grid and all raw-cell audits pass; original MOM is unchanged and exact.
Do not conceal a reference-input parser discrepancy by relaxing raw equality
or numerical tolerances. The theta0 control may emit plm's expected essentially
perfect *between-mean* fit warning; within error/score variance remain positive.

## Reproduce

From repo root with a current Rust binary (`cargo build --locked`), the author
MOM.dat locally downloaded and R/plm available for reference only:

```sh
repo=$PWD
raw=/absolute/path/to/MOM.dat
work=$(mktemp -d)
sh examples/mma21p1panel/prepare.sh "$raw" "$work/MOM.csv"
(cd "$work" && "$repo/target/debug/tabdat-explore-rs" --json \
  -f "$repo/examples/mma21p1panel/manual_gls_covariance.td" > manual_gls_covariance.jsonl)
Rscript examples/mma21p1panel/manual_gls_covariance_reference.R "$raw" "$work/reference"
python3 examples/mma21p1panel/check_manual_gls_covariance.py "$work" "$work/reference" "$raw"
```

Use a separate fresh directory/`--text` for readable output. Converter/export
refuse existing targets; R requires a fresh reference directory. Synthetic audit
requires `--synthetic` and waives only the original raw hash, not sample/numerical
checks. Python/R/plm are reference-only, not production dependencies. Historical
float32/printed-SE parity, fixed-ML auxiliary work, source GLS-under-ML-HC1
labelling/full reporting, residual/GLS companion inventories and book coverage
remain open.
