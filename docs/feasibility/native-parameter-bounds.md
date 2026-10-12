# Cross-family native packed-parameter bounds

Follow-up PR #183 adds an independent practical resource envelope
([final report](phase1-final-report.md)): low-level models ≤1024 nominal
parameters and ≤10,000,000 parameter-observation cells; conservative facade
stored-column and dataset limits. The 46,340 arithmetic predicate below is
retained/tested, **not the overall supported workload limit**.

[PR #181](https://github.com/SaehwanPark/tabdat-explore-rs/pull/181), base
`c122ab6`. **Pass for bounded packed-parameter arithmetic hardening**, not complete
native dimension/allocation/index/ownership or sanitizer/leak certification.

Heckman previously guarded `(tot*tot+tot)/2`; same pinned native source has signed
`k*(k+1)/2` / `(k*k+k)/2` in qr_estimate.c:190, estimate.c:908 and
gretl_model.c:2616/2969/5214/5330. c_int list length alone does not bound these
intermediates. A safe numerical wrapper must reject nominal size before C,
independent of hoped-for rank dropping. No oversized native UB was executed.

Pure `parameter_count_supported(k)` checks k>0 and checked k*(k+1)<=i32::MAX:
maximum **46,340**, 46,341 rejected; usize::MAX arithmetic never wraps. Also bounds
known ijton expression `nrows*i+j-i-((i-1)*i/2)` for valid indices at this count;
do not infer all source expressions are covered.

Generic list validates nominal list.len()-2 plus family auxiliary count: zero
OLS/classical binary/panel/quantile/Poisson, one NB2 dispersion or Tobit scale.
Duration special list excludes censor dummy/separator and includes Weibull scale
(not exponential). Heckman includes outcome beta/Mills and selection beta together.
IV checks structural and instrument counts independently; owner follow-up found
upstream tsls.c:1701 may prepend structural constant when absent in instruments,
so include that additional instrument before bound check. Dynamic capped API is
already far lower. Facades share owned InvalidProblem check before native dataset
construction; no panic leaks through ordinary facade on this new parameter limit.

Pure threshold tests0/1/46340/46341/usizeMAX and owned-facade errors. Small live
dataset + ~185KiB repeated-index lists prove arithmetic-specific failure before
numerical dispatch for every exposed family, including 46340+auxiliary and each
IV side/auto-added constant; normal OLS recovery verified. Existing direct Heckman
oversized-list test remains. Actual large square models/allocations are not tests.

Mac **54 all-feature / 25 default**, same emulated Linux **56 / 27** tests pass;
all nominal numerical/sample/covariance comparisons unchanged. Root locked full
fmt/check/test/Clippy, spike/adapter fmt/Clippy, strict audit/deny pass. Full unsafe
inventory unchanged **93 used adapter expressions**, known nested attribution93/
compiler forbid; no dependency/waiver/new unsafe/numerical tolerance change.
Logs `/tmp/parameter-bounds-{owner,default,linux-final,geiger}.log` (geiger JSON kept).

Three owner reviews, not independent: source signed intermediates/nominal parameter
accounting; pre-C validation/owned errors/no huge allocation/UB tests; numerical/
platform/policy evidence. IV inserted-constant correction explicitly followed up on
both platforms. No other known defect within this bounded count contract.

Other native dimension products, C allocation error paths, transitive/ReadStat/
DuckDB ownership, physical-host/sanitizer/leak and redistribution/backend selection
remain separate work. Parameter count itself is a safety cap, not estimator-wide
product parity. Phase1 partial, not whole unsafe-boundary acceptance.
