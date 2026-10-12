# Phase 1 preliminary feasibility assessment

**Disposition: partial / blocked, not complete.** Updated 2026-10-11 after
bounded PRs #162, #163, #165, #166 and the measurement slice #168. The
[roadmap](../TABDAT_RUST_PORT_ROADMAP.md) remains the acceptance tracker.
This assessment does not replace a final accepted backend decision or authorize
shipping native artifacts. Proposal intent is not implemented support.

## Evidence and decisions at the current boundary

| Capability | Verified evidence | Current disposition | Missing acceptance evidence |
| --- | --- | --- | --- |
| DuckDB | Local CSV/Parquet/Arrow and bounded runtime; owned connection; HTTP ranges/values/NULL/Arrow and unsigned S3 path-style loopback reads | Continue canonical-engine bounded evaluation under ADRs 0003/0007 | TLS/authenticated S3, extension/offline/native distribution and comprehensive upstream ownership/unsafe review |
| ReadStat | Existing pinned macOS/Linux native builds; Rust DTA facade, labels and system/tagged missing fixtures | Continue isolated DTA candidate, **do not accept product ingestion yet** | Recorded source-quality findings, callback/native review, broader format/oracle references, string-key/empty-string policies, redistribution |
| libgretl | Existing cross-platform build; owned serialized/checked boundary; NIST/Python OLS, Python/R HC1+cluster, Python/R logit+probit; measured first/repeated use | Continue isolated statistical candidate under ADRs 0005/0006/0010; **primary backend not yet accepted** | Remaining estimator fixtures, plugin lifecycle/packaging, general sample/model/inference normalization, updated platform/unsafe/native-license review |
| Visualization | Draft #164 Rust spec/SVG/PNG and byte-identical Python scatter; release measurements | **Blocked: do not merge/adopt the tested dependency graph** | Required audit fails; updated tested/policy-clean candidate graph, transitive license/native review, platform/artifact coverage |
| Linear algebra | Current safe workspace QR/OLS boundary and libgretl BLAS/LAPACK build evidence | No general linear-algebra substrate is newly selected by these spikes | Bounded faer versus BLAS/LAPACK benchmark, ownership/build/thread/license evidence and accepted ADR |

The original proposal's primary-libgretl and ReadStat preferences are candidates,
not automatic acceptance. The current native OLS/statistical workspace code does
not establish a broad numerical substrate decision or replace the libgretl
estimator feasibility requirements.

## Blocking / unresolved findings

1. **Plotting acceptance blocker:** reproducible `vl-convert-rs` 1.9.0 renders
   correctly but its Cargo graph audit fails with 19 vulnerabilities and 32 denied
   warnings. Published 2.0.0-rc7 lock audit also fails (4 vulnerabilities, 7
   warnings); it was not built or freshly resolved as an application. No advisory
   ignore/waiver was added. Detailed evidence is preserved on
   [draft PR #164](https://github.com/SaehwanPark/tabdat-explore-rs/pull/164),
   branch `feat/phase1-visualization`, in its `docs/feasibility/visualization.md`.
   That report/source is deliberately **not merged into main**.
2. **Native estimator prerequisite:** the pinned library dispatches count models
   to dynamically loaded `count_data_estimate`; current build scripts only build
   libgretl. Resolve plugins and their init/search-path/thread/ownership/packaging
   boundaries before claiming those estimator fixtures pass. Plugin build/use
   was not attempted in this session; this is source reconnaissance, not a
   demonstrated plugin build failure.
3. **Unmet numerical gates:** quantile, Tobit, Poisson, negative binomial,
   IV/2SLS, panel FE/RE, dynamic panel, Heckman and survival fixtures are still
   unchecked. Passed linear/binary fixtures cannot substitute for these families.
4. **Safety/distribution gates:** new libgretl safe-API defects were corrected and
   reviewed, but no comprehensive native/transitive unsafe review, sanitizer/leak
   proof or license/NOTICE/corresponding-source distribution approval ran. The
   ReadStat report retains warnings about native non-DTA error paths; do not
   interpret build-warning demotions as source-quality approval.
5. **Verification risk:** an unchanged runtime reshape test failed once on a
   generated temporary Parquet read; focused and full reruns passed, cause
   unproven. Tracked separately as
   [issue #167](https://github.com/SaehwanPark/tabdat-explore-rs/issues/167).

The "no unresolved P0 feasibility blocker" gate cannot be marked complete while
mandatory plotting dependency policy fails. The other unmet gates independently
prevent a whole-phase completion claim, even if a newer renderer graph passes.

## Next bounded loops

1. Recover draft #164 without overwriting it. Evaluate a **freshly resolved**
   current renderer graph; compare the same scatter spec/artifacts and audit/deny
   it. If no policy-compliant graph exists, record an upstream fix or alternative
   design decision rather than silently widening policy.
2. Investigate one libgretl plugin-backed estimator and its native lifecycle/build
   contract. Open a new draft PR at the contract checkpoint, then validate with
   pinned Python and an independent trusted reference before proceeding.
3. Port/validate the remaining estimator families in small dependency-ordered
   PRs, preserving sample identity, covariance/optimizer conventions and failures.
4. Review ReadStat and DuckDB unsafe/lifecycle boundaries and distribution matrix;
   close native source-quality/semantics issues or explicitly reject candidates.
5. Benchmark/select the initial general linear-algebra route and record accepted
   backend/license/packaging decisions in ADRs. Review every Phase 1 exit gate.

## Workflow / verification summary

Every implemented target started with an early draft PR and progress commits.
Completed, locally verified slices were merged autonomously per user authority;
the failed-policy renderer remains draft. Actions stay disabled under ADR 0009.
Root locked format/check/test/Clippy, advisory/deny and first-party unsafe policy
checks are separate from statistical/FFI acceptance. Python/R were external
validation tools only; no Python/R runtime dependency was added.

Reviews were three separate **owner** passes per completed slice (with follow-up
on corrected VCV behavior), not independent hosted reviewers. No independent
reviewer runtime was available. The source/results, remaining limitations and
review disposition are recorded in individual reports and PRs. Do not represent
this preliminary matrix or any PR merge as completing the requested Phase 1.
