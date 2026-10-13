# Syntax fuzzing (external validation)

This separate workspace instruments the safe language/domain crates with
libFuzzer and AddressSanitizer. It adds no production runtime dependency.
The target checks deterministic command/script parsing and tokenization on UTF-8
inputs; non-UTF-8 inputs are outside this target's contract. Corpus and crash
artifacts are local, ignored outputs. Keep useful failures as ordinary regression
tests before removing their artifacts.

Install cargo-fuzz 0.13.2 and a nightly Rust toolchain, then run from the repo root:

```sh
cargo install cargo-fuzz --version 0.13.2 --locked
rustup toolchain install nightly
python3 tools/seed_parser_fuzz.py
cargo +nightly fuzz run syntax --fuzz-dir fuzz --target-dir target/language-fuzz -- -seed=24301 -max_total_time=60 -max_len=4096
```

The seed tool copies all pinned parser cases and generates adversarial recursive
inputs, including the original 40,014-byte stack-overflow reproduction. LibFuzzer
replays initial corpus entries larger than `max_len`; the length limit bounds
subsequent mutations. A smoke run is bounded evidence, not a correctness proof.
Replay an individual artifact with the same cargo command followed by its path.
Format this workspace separately with `cargo fmt --manifest-path fuzz/Cargo.toml`.

[ADR 0023](../docs/adr/0023-syntax-resource-budgets-and-fuzz-tooling.md) records the
resource limits, compatibility restriction and validation dependency licenses.
