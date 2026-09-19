# ADR-0008: Use the pure-Rust SHA-256 implementation for datasignature

Status: accepted for the bounded eager-runtime slice

Date: 2026-09-18

## Context

The pinned Python oracle defines `datasignature` as a reproducibility
fingerprint: it seeds SHA-256 with `tabdat-datasignature/v1\0` and applies an
exact length-framed schema/row/value encoding. SHA-256 and its byte protocol are
observable behavior, not an interchangeable implementation detail. The Rust
runtime currently has no hashing dependency.

## Decision

Use the `sha2` crate directly from the runtime adapter and implement the
protocol framing and value encoding in safe Rust. Keep the digest result owned
at the runtime boundary. Do not use a native crypto library, shell command, or
backend-specific hash; do not describe the fingerprint as an authenticity or
tamper-proof security mechanism.

## Consequences

- The runtime remains free of native crypto initialization and FFI ownership.
- The direct dependency makes the protocol auditable and keeps the algorithm
  stable across supported platforms.
- Cargo advisory/license checks must cover the added dependency before the PR
  is accepted.
- This decision covers the bounded eager command only; changing the protocol,
  adding a streaming backend, or claiming cryptographic authenticity requires a
  new decision.
