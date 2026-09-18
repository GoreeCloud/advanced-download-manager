<!-- File internal version: v0.7 -->
# GoreeCloud Advanced Download Manager

**Product internal version:** 0.1.0  
**Release lifecycle:** Development  
**Repository state:** Development-stage Rust source with verified SQLite durable job-state persistence, bounded staging-file durability, and a native single-stream HTTP runtime; no supported end-user download workflow or installable client is currently verified.

GoreeCloud Advanced Download Manager is the developing cross-platform download and transfer-orchestration application for Linux, Windows, and Android. Its target is a local-first common download engine with durable transfers, intelligent acceleration, queues, automation, privacy and security controls, browser integration, and authorized GoreeCloud ecosystem coordination.

## Current availability

There is currently **no supported build for ordinary download-manager use**. The repository contains a Rust workspace with portable core-domain contracts, HTTP resume-safety contracts, durable state/recovery contracts, a transaction-safe SQLite job-state backend, bounded durable staging-file primitives, a native `download-runtime` single-stream transport/orchestration crate, and a minimal `gcdm` command-line shell. The runtime can execute bounded HTTP/HTTPS requests through reqwest/Rustls source, but no supported CLI or graphical client exposes that path yet; representative live HTTPS, Android transport, active pause/cancel control, redirect/authentication/proxy policy, packaging, and production acceptance remain unverified.

The implemented source foundation covers bounded job identity/state contracts, sensitive URL redaction in debug output, progress invariants, final-file-promotion gating, remote-validator decisions, HTTP full/restart/resume request planning, `Content-Range` parsing and response disposition, schema compatibility, deterministic partial-file naming, checkpoint/recovery planning, partial-file reconciliation, final-artifact revalidation, generation-based atomic mutation-batch boundaries, SQLite schema/version validation, optimistic generation checks, integrity checks, lossless native-path persistence, synchronized staging-file append/truncation, non-overwriting final-file promotion, filesystem-before-database checkpoint ordering, native single-stream full/resume execution, and recovery across SQLite reopen and the promotion/checkpoint crash window. These foundations are still not a complete supported download manager.

## Product direction

The implementation prioritizes a durable local transfer core before multipart acceleration, browser interception, remote control, synchronization, or broader ecosystem integrations. BitTorrent and magnet transfers remain planned for delegation to GoreeCloud Swarm rather than duplicating its protocol stack.

## Source foundation

The implementation uses a Rust workspace pinned to Rust 1.98.1:

- `crates/download-core` — portable shared job/state/resume-domain contracts;
- `crates/download-http` — dependency-free HTTP resume-safety request/response contracts with no network I/O;
- `crates/download-state` — dependency-free durable-state and recovery contracts;
- `crates/download-fs` — bounded staging-file durability primitives for checkpoint-length reconciliation, synchronized append/truncate operations, and safe final-file promotion; it performs no network I/O;
- `crates/download-store-sqlite` — transaction-safe SQLite durable job-state adapter;
- `crates/download-runtime` — native single-stream HTTP/HTTPS execution and durability orchestration using pinned reqwest 0.13.5 with Rustls, automatic redirects disabled, ambient system proxies disabled, and identity transfer encoding for range safety;
- `crates/gcdm` — minimal Development-stage CLI shell.

CI validates formatting, Clippy, tests, native workspace checks, repository policy, and portable shared-core compilation for the Android target. A successful target compilation is not an Android application acceptance result.

## Documentation

- [Product specification](SPECIFICATIONS.md)
- [Feature roadmap](FEATURE-ROADMAP.md)
- [Current feature state](FEATURES.md)
- [Architecture](ARCHITECTURE.md)
- [ADR-0001 — Rust shared core](docs/architecture/ADR-0001-rust-shared-core.md)
- [ADR-0002 — Durable job state and recovery contracts](docs/architecture/ADR-0002-durable-job-state-and-recovery-contracts.md)
- [ADR-0003 — SQLite durable job store](docs/architecture/ADR-0003-sqlite-durable-job-store.md)
- [ADR-0004 — Native single-stream HTTP/HTTPS runtime](docs/architecture/ADR-0004-native-single-stream-http-runtime.md)
- [User manual](USER-MANUAL.md)
- [Privacy policy](PRIVACY%20POLICY.md)
- [Security guidance](SECURITY.md)
- [Benefits](BENEFITS.md)
- [Competitive objectives](COMPETITIVE-OBJECTIVES.md)
- [Branding](BRANDING.md)
- [Repository notes](NOTES.md)
- [Changelog](CHANGELOG.md)

## Platform contract

`goreecloud.platform.yaml` records the current GoreeCloud Platform Contract declaration. All nine Integral Platform Systems — GoreeCloud Manager, Privacy Shield, Wardveil Security, Everkeep, Glaze UI, GoreeCloud Mesh, GoreeCloud Identity, GoreeCloud Policy, and GoreeCloud Observability — remain unaccepted for this application. The current required Stable Glaze UI consumer target is 1.5.1, but no runtime UI or application-specific Glaze acceptance exists. Contract metadata does not establish integration.

## Development boundaries

The Rust core/runtime direction, SQLite metadata backend, bounded staging-file adapter, and first native single-stream HTTP/HTTPS runtime are selected and implemented. Exact-head CI verifies live loopback HTTP full/resume requests plus durability/reopen recovery on Ubuntu and Windows; the Rustls HTTPS path is compiled but representative live TLS acceptance is still pending. Android currently validates only the portable/shared-core packages, not `download-runtime`. Active pause/cancel orchestration, redirects, authentication/cookies, proxies, broader retry/storage-failure behavior, service/IPC, graphical clients, packaging, signing, and production recovery remain separate implementation and evidence gates. No platform is treated as supported or Stable merely because source or a bounded test passes.
