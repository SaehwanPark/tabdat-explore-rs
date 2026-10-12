# Exclusive reshape fixture-directory ownership

[PR #180](https://github.com/SaehwanPark/tabdat-explore-rs/pull/180), base
`e3a6618`; [issue167](https://github.com/SaehwanPark/tabdat-explore-rs/issues/167).
**Verified test-infrastructure hardening**, not a runtime/backend semantic fix.
Issue remained open after #180. [Follow-up #184](reshape-collision-replay.md)
reproduces live-clock shared ownership and deletion of another guard's real Parquet,
verifies the fix, and closes the reproduced fixture defect. The exact historical
interleaving remains unavailable; reopen if exclusive ownership sees recurrence.

Unchanged root reshape tests intermittently failed reading generated Parquet,
including line301 missing reshape_wide_collision.parquet during #179 verification.
Focused/full reruns passed; original failure cause remains unproven.

Old fixture chose pid+SystemTime.as_nanos then create_dir_all, silently adopting
existing directories. Drop recursively removed its root. Clock uniqueness does
not establish exclusive ownership. Concrete risk is corrected without claiming
an observed wall-clock collision caused either original event.

Process-local AtomicU64 sequence (Relaxed for distinct candidates) supplies IDs;
exclusive create_dir establishes ownership. AlreadyExists selects another candidate,
including leftovers from a reused PID; no existing root is adopted/deleted. Other
I/O errors remain errors, never hidden behind a read retry or sleep. Drop removes
only successfully claimed directory. No dependency/runtime/semantic/tolerance change.

Deterministic existing-directory double claim yields AlreadyExists; rejected claim
cannot remove owner's marker. Missing-parent error stays NotFound. Thirty-two
concurrent retained fixtures have distinct roots; dropping all but one preserves
retained marker. These test the contract, not probabilistic timer resolution.

macOS 27.0.1 arm64 / Rust1.97.1: **14 reshape tests**, **30 repeated suites / 420
passes**, full locked root fmt/check/test/Clippy and follow-up focused/Clippy pass.
Same exact helper/two ownership tests extracted to external rustc std-only harness
on emulated Linux x86_64: **2 passes**. This is not full Linux DuckDB/runtime parity;
no production code or native library changed. Original helper/tests are in
`crates/tabdat-runtime/tests/reshape_contract.rs`. Logs `/tmp/reshape-ownership-*`.

Three owner reviews, not independent: atomic identity vs filesystem ownership;
AlreadyExists/other errors/destructor independence; stress/proportional evidence/
no original-cause overclaim. Final marker/error follow-up passes. Other integration
fixture helpers and full native filesystem/lifetime diagnosis remain separate work.

```sh
cargo test --locked -p tabdat-runtime --test reshape_contract
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
```

No whole-phase acceptance or issue closure from a successful stress run. Remaining
Phase1 numerical/backend/native safety/license/distribution gates remain open.
