# Reshape fixture collision causal replay

Status: **investigation in progress**, base `a15d0fe`; issue #167.

Review recovered the former helper from `c122ab6^`: its candidate is PID plus
`SystemTime::now().duration_since(UNIX_EPOCH).as_nanos()`, and `create_dir_all`
authorizes two guards to adopt/delete the same directory. A native Rust clock
probe on the reported macOS host observed 32,000 samples, 26,815 distinct values,
4,807 duplicated timestamps and 5,185 duplicate samples. Nanosecond *units* are
not nanosecond *resolution* or uniqueness; sample values are microsecond-aligned.
This alone does not establish the exact historical failing interleaving.

## Bounded plan / contract

- Replay actual concurrent former-helper claims under an exclusively owned outer
  fixture, using live SystemTime values, not injected duplicate timestamps.
- Retain all guards, identify an actual shared root, write/read synthetic Parquet,
  drop one adopter while another remains live, and reproduce native missing-file
  and runtime ParquetRead signatures. This manual clock-dependent probe is ignored
  in ordinary portable tests; no clock collision is required for routine gates.
- Extend deterministic exclusive-claim coverage to real Parquet/runtime reshape:
  rejected claim cleanup must not delete the owner's file or break its session.
- No production/runtime/dependency changes, sleeps, read retries or tolerance changes.
  Do not restore unsafe fixture ownership outside the isolated test harness.
- Verify focused/stress/root checks; update issue disposition only to the proved
  mechanism, retaining the limitation that original per-fixture traces are absent.

Rejected renderer PR #164 was reviewed and closed as superseded by #169/ADR0011
and #183/ADR0021, not as audit-remediated. Its remote branch and commit
`69ac05169db1fef54b56cd1d64e2d152fa388818` remain intact for historical evidence.
