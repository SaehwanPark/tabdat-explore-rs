# Summary: JSON preview results

PR #161 adds bounded JSONL success output for `head` and `tail` in repeated
batch commands and `.td` scripts, including nested `run`. Each preview emits
the existing ordered columns and rows as `PreviewResult`. Cell values follow
the pinned Python conversion policy for scalars, exact integers, decimals,
non-finite floats, text, and bytes.

The pinned Python CLI checks (3 tests) and all four locked Rust workspace gates
passed. GitHub Actions remain disabled; no hosted checks were run. Evidence and
the review are in this directory, with the current implementation scope
recorded in `SPEC.md` and the Phase 7.3 roadmap note.

Full result-surface JSON serialization and full Python script JSON parity
remain deferred.
