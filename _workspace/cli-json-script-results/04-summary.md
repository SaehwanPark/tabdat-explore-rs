# Summary: CLI JSONL results from `.td` scripts

**Producer:** task owner. **Consumer:** next maintainer.
**State:** complete for the bounded slice; PR #160 is the review and merge
handoff. **Rust implementation revision:**
`e9747e7893f1191dfd217598d18330e7bccc3f05`.

## Outcome

`--json -f <script.td>` streams versioned `LoadResult` and `CountResult`
envelopes as commands succeed. Nested `run` commands preserve result order.
Unsupported script commands are rejected before effects and produce one
source-located `ScriptError`, preserving prior result lines. Runtime failures,
inactive branches, and non-JSON scripts retain their existing behavior.

## Validation

- Pinned Python CLI script tests: 4 passed.
- Focused Rust CLI integration tests: 11 passed.
- `cargo fmt --all -- --check`: passed.
- `cargo check --locked --workspace --all-targets`: passed.
- `cargo test --locked --workspace --all-targets`: passed.
- `cargo clippy --locked --workspace --all-targets -- -D warnings`: passed.
- Changed-document relative links and `git diff --check`: passed.
- GitHub Actions disabled; no hosted result claimed.

## Remaining work

Full Phase 5 JSON reporting and complete Python script result parity remain
unchecked. Add serializers and fixtures for other typed runtime results in
future bounded slices. In particular, the pinned Python nested script contract
also emits `PreviewResult`, which this Rust surface does not yet support.
