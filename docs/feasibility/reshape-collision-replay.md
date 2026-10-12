# Reshape fixture collision causal replay

Status: **reproduced fixture defect; exclusive-ownership fix verified**.
Base `a15d0fe`; [PR #184](https://github.com/SaehwanPark/tabdat-explore-rs/pull/184),
[issue #167](https://github.com/SaehwanPark/tabdat-explore-rs/issues/167).

## Cause and evidence

Review recovered the former helper from `c122ab6^`: its candidate is PID plus
`SystemTime::now().duration_since(UNIX_EPOCH).as_nanos()`, and `create_dir_all`
lets two guards adopt/delete the same directory. On the original macOS 27.0.1
arm64 / Rust 1.97.1 host, a native Rust clock probe observed 32,000 samples,
26,815 distinct values, 4,807 duplicated timestamps and 5,185 duplicate samples.
Nanosecond **units** do not imply nanosecond resolution or uniqueness; observed
values are microsecond-aligned.

The repository's manual causal probe now constructs the former helper concurrently,
using **live SystemTime values**, not injected duplicate timestamps. All legacy
claims are confined below an exclusively owned outer fixture. With 32 workers ×
100 rounds, the passing run produced **261 shared-root claims among 3,200 claims**.
Owner follow-up observed **315 shared-root claims** in another 3,200-claim run.
It retains the guards, writes real synthetic Parquet into one shared root,
verifies the file exists, then drops one adopter while another guard remains live.
The file disappears. A fresh DuckDB read returns the native missing-file error;
current runtime `use` returns `FileNotFound`.

This proves the former clock/adoption/cleanup mechanism can cause the observed
missing-Parquet class of failure. The original historical per-fixture creation/
cleanup trace was not retained, so the **exact original interleaving is not
proven**. Historical `ParquetRead` / “Cannot open file” errors are consistent with
deletion after validation/binding; this replay deletes before those stages and
therefore reports “No files found that match the pattern” / `FileNotFound`.
No separate DuckDB/reshape algorithm defect is demonstrated.

## Resolution and regression

PR #180 already fixed the underlying ownership defect: AtomicU64 candidates,
exclusive `create_dir`, retry only `AlreadyExists`, and no ownership/cleanup for
rejected claims. A clock is no longer the uniqueness authority. This follow-up
adds a deterministic **real-Parquet/runtime reshape** test proving a rejected
claim cannot delete the original owner's file or break its session.

The legacy probe is deliberately ignored in ordinary portable tests: a live clock
collision is platform-dependent and must not become a required correctness gate.
The normal regression uses explicit duplicate candidates with exclusive claims
and does not need timing, sleeps, read retries or numerical tolerance changes.
No production/runtime/dependency code changes.

```sh
cargo test --locked -p tabdat-runtime --test reshape_contract
cargo test --locked -p tabdat-runtime --test reshape_contract \
  former_live_clock_claims_reproduce_missing_parquet -- --ignored --nocapture
```

Validation on the reported macOS host:

- Default reshape suite: **15 pass / 1 manual diagnostic ignored**.
- **100 repeated suites / 1,500 passing tests**; no recurrence.
- Manual live-clock former-helper probe passes with the counts above and actual
  file deletion while another owner remains live.
- Root locked fmt/check/test/Clippy, strict audit/deny, changed-document links and
  `git diff --check` pass. No full runtime/Linux or sanitizer claim for this slice.
- Three explicit owner passes covered isolated legacy cleanup, deterministic
  Parquet/session ownership, and evidence/portability/scope; follow-up passes.
  No independent reviewer claim. Follow-up collects filesystem/clock errors until
  all barrier rounds finish so an I/O failure cannot strand remaining workers.

Initial harness assertions were too specific about error timing: the native fresh
binder reported “No files found” rather than later-open “Cannot open file”, and
current runtime prevalidation reported `FileNotFound` rather than `ParquetRead`.
Both failed diagnostic runs are retained locally; assertions now test the exact
fresh-bind/prevalidation errors actually exercised, without claiming an exact
historical-stage replay. Local logs: `/tmp/reshape-causal-*`; commands/tests and
this report are the durable evidence, not a promise to retain temporary logs.

## Issue disposition / prevention

Close #167 as addressed by the reproduced fixture ownership defect, merged #180
fix and real-Parquet regression/stress evidence. This is **not proof that all
possible missing-file failures are cured**. Reopen with new per-fixture path,
creation/drop timing and native error evidence if it recurs under exclusive
ownership; a new independent runtime/filesystem defect would need separate work.

Future test fixtures must claim ownership atomically, not infer it from timestamps,
PIDs, random-looking names or `create_dir_all`. Cleanup is authorized only by a
successful exclusive claim. The lesson is preserved here alongside the original
[ownership report](reshape-fixture-ownership.md), rather than duplicating a new
lessons system.

## Rejected renderer disposition

PR #164 was reviewed and closed as **rejected/superseded**, not audit-remediated:
#169/ADR 0011 and #183/ADR 0021 adopt policy-clean native SVG/resvg. Its vulnerable
Vega/Deno graph must not merge. The PR's artifact/audit evidence and remote branch
`feat/phase1-visualization` at `69ac05169db1fef54b56cd1d64e2d152fa388818` remain
intact. Closing the abandoned proposal does not delete the rejected evidence or
claim Vega-Lite parity.
