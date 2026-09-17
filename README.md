<!-- File internal version: v0.5 -->
# GoreeCloud Advanced Download Manager

**Product internal version:** 0.1.0  
**Release lifecycle:** Development  
**Repository state:** Initial Rust source foundation with verified SQLite durable job-state persistence; no supported network download workflow or installable client is currently verified.

GoreeCloud Advanced Download Manager is the developing cross-platform download and transfer-orchestration application for Linux, Windows, and Android. Its target is a local-first common download engine with durable transfers, intelligent acceleration, queues, automation, privacy and security controls, browser integration, and authorized GoreeCloud ecosystem coordination.

## Current availability

There is currently **no supported build for ordinary download-manager use**. The repository contains a Rust workspace with portable core-domain contracts, HTTP resume-safety contracts, durable state/recovery contracts, a transaction-safe SQLite job-state backend, and a minimal `gcdm` command-line shell, but it does not yet perform network downloads, execute filesystem durability/final-promotion workflows, prove end-to-end runtime restart recovery, or provide a supported graphical client.

The implemented source foundation covers bounded job identity/state contracts, sensitive URL redaction in debug output, progress invariants, final-file-promotion gating, remote-validator decisions, HTTP full/restart/resume request planning, safe range-response disposition, schema compatibility, deterministic partial-file naming, checkpoint/recovery planning, partial-file reconciliation, final-artifact revalidation, generation-based atomic mutation-batch boundaries, SQLite schema/version validation, optimistic generation checks, integrity checks, lossless native-path persistence, and reopen/round-trip coverage. These foundations are not a complete download engine.

## Product direction

The implementation prioritizes a durable local transfer core before multipart acceleration, browser interception, remote control, synchronization, or broader ecosystem integrations. BitTorrent and magnet transfers remain planned for delegation to GoreeCloud Swarm rather than duplicating its protocol stack.

## Source foundation

The implementation uses a Rust workspace pinned to Rust 1.98.1:

- `crates/download-core` — portable shared job/state/resume-domain contracts;
- `crates/download-http` — dependency-free HTTP resume-safety request/response contracts with no network I/O;
- `crates/download-state` — dependency-free durable-state and recovery contracts;
- `crates/download-store-sqlite` — transaction-safe SQLite durable job-state adapter;
- `crates/gcdm` — minimal Development-stage CLI shell.

CI validates formatting, Clippy, tests, native workspace checks, repository policy, and portable shared-core compilation for the Android target. A successful target compilation is not an Android application acceptance result.

## Documentation

- [Product specification](SPECIFICATIONS.md)
- [Feature roadmap](FEATURE-ROADMAP.md)
- [Current feature state](FEATURES.md)
- [Architecture](ARCHITECTURE.md)
- [ADR-0001 — Rust shared core](docs/architecture/ADR-0001-rust-shared-core.md)
- [ADR-0002 — Durable job state and recovery contracts](docs/architecture/ADR-0002-durable-job-state-and-recovery-contracts.md)
- [ADR-0003 — SQLite durable store](docs/architecture/ADR-0003-sqlite-durable-store.md)
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

The Rust core/runtime direction is selected for the shared engine foundation, backend-neutral HTTP resume contracts exist, and the initial SQLite durable job-state backend is implemented. Real HTTP/HTTPS network execution, filesystem mutation and durable write ordering, end-to-end restart recovery, HTTP/3, IPC/API technology, graphical client frameworks, credential storage, packaging, signing, and production recovery mechanisms remain separate implementation decisions and evidence gates. No platform is treated as supported or Stable merely because portable source compiles for it.
