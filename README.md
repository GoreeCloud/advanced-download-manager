<!-- File internal version: v0.6 -->
# GoreeCloud Advanced Download Manager

**Product internal version:** 0.1.0  
**Release lifecycle:** Development  
**Repository state:** Development-stage Rust application foundation plus an application-owned Firefox client under `clients/firefox/`. The standalone application engine is not yet a supported downloader; the Firefox client separately retains its accepted Stable 0.2.12 release evidence.

GoreeCloud Advanced Download Manager is the developing cross-platform download and transfer-orchestration application for Linux, Windows, and Android. Its target is a local-first common download engine with durable transfers, intelligent acceleration, queues, automation, privacy and security controls, browser integration, and authorized GoreeCloud ecosystem coordination.

## Current availability

There is currently **no supported standalone Advanced Download Manager application build for ordinary download-manager use**. The repository contains a Rust workspace with portable core-domain contracts, HTTP resume-safety contracts, durable state/recovery contracts, a transaction-safe SQLite job-state backend, bounded durable staging-file primitives, and a minimal `gcdm` command-line shell, but it does not yet perform network downloads, coordinate filesystem durability and final promotion with persisted checkpoints in an end-to-end transfer workflow, prove runtime restart recovery, or provide a supported Linux/Windows/Android graphical application client. Separately, `clients/firefox/` contains the Mozilla-signed GoreeCloud Download Manager Firefox client, whose Stable 0.2.12 status applies only to that Firefox client and its accepted native helper boundary.

The implemented source foundation covers bounded job identity/state contracts, sensitive URL redaction in debug output, progress invariants, final-file-promotion gating, remote-validator decisions, HTTP full/restart/resume request planning, safe range-response disposition, schema compatibility, deterministic partial-file naming, checkpoint/recovery planning, partial-file reconciliation, final-artifact revalidation, generation-based atomic mutation-batch boundaries, SQLite schema/version validation, optimistic generation checks, integrity checks, lossless native-path persistence, reopen/round-trip coverage, synchronized staging-file append/truncation, checkpoint-length reconciliation, and non-overwriting final-file promotion. These foundations are not a complete download engine.

## Product direction

The implementation prioritizes a durable local transfer core before multipart acceleration, browser interception, remote control, synchronization, or broader ecosystem integrations. BitTorrent and magnet transfers remain planned for delegation to GoreeCloud Swarm rather than duplicating its protocol stack.

## Source foundation

The implementation uses a Rust workspace pinned to Rust 1.98.1:

- `crates/download-core` — portable shared job/state/resume-domain contracts;
- `crates/download-http` — dependency-free HTTP resume-safety request/response contracts with no network I/O;
- `crates/download-state` — dependency-free durable-state and recovery contracts;
- `crates/download-fs` — bounded staging-file durability primitives for checkpoint-length reconciliation, synchronized append/truncate operations, and safe final-file promotion; it performs no network I/O;
- `crates/download-store-sqlite` — transaction-safe SQLite durable job-state adapter;
- `crates/gcdm` — minimal Development-stage CLI shell.
- `clients/firefox/` — application-owned GoreeCloud Download Manager Firefox client; Stable 0.2.12 with accepted native helper 0.2.11 / protocol 2 and separately governed release evidence.

CI validates formatting, Clippy, tests, native workspace checks, repository policy, and portable shared-core compilation for the Android target. A separate Firefox-client workflow validates the migrated Firefox source/tests and requires deterministic 0.2.12 package parity with the previously accepted candidate SHA-256. A successful Android target compilation is not an Android application acceptance result.

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
- [Firefox client](clients/firefox/README.md) — application-owned Firefox source, native-helper boundary, validation, packaging, and version-specific release evidence.

## Platform contract

`goreecloud.platform.yaml` records the current GoreeCloud Platform Contract declaration. All nine Integral Platform Systems — GoreeCloud Manager, Privacy Shield, Wardveil Security, Everkeep, Glaze UI, GoreeCloud Mesh, GoreeCloud Identity, GoreeCloud Policy, and GoreeCloud Observability — remain unaccepted for this application. The current required Stable Glaze UI consumer target is 1.5.1, but no runtime UI or application-specific Glaze acceptance exists. Contract metadata does not establish integration.

## Development boundaries

The Rust core/runtime direction is selected for the shared engine foundation, backend-neutral HTTP resume contracts exist, the initial SQLite durable job-state backend is implemented, and bounded staging-file durability primitives are implemented. Real HTTP/HTTPS execution in the shared Rust application engine, transaction ordering between persisted checkpoints and filesystem writes, end-to-end application-engine restart recovery, HTTP/3, IPC/API technology, Linux/Windows/Android graphical client frameworks, credential storage, application packaging/signing, and production recovery mechanisms remain separate implementation decisions and evidence gates. The attached Firefox client is a separately evidenced exception: Stable 0.2.12 applies only to `clients/firefox/` and does not promote the Development application core or other platforms.
