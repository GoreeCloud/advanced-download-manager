<!-- File internal version: v0.4 -->
# GoreeCloud Advanced Download Manager

**Product internal version:** 0.1.0  
**Release lifecycle:** Development  
**Repository state:** Initial Rust source foundation; no supported download workflow or installable client is currently verified.

GoreeCloud Advanced Download Manager is the developing cross-platform download and transfer-orchestration application for Linux, Windows, and Android. Its target is a local-first common download engine with durable transfers, intelligent acceleration, queues, automation, privacy and security controls, browser integration, and authorized GoreeCloud ecosystem coordination.

## Current availability

There is currently **no supported build for ordinary download-manager use**. The repository contains a Rust workspace with portable core-domain contracts, HTTP resume-safety contracts, backend-neutral durable-state/recovery contracts, and a minimal `gcdm` command-line shell, but it does not yet perform network downloads, persist jobs to a durable backend, recover runtime jobs after restart, or provide a supported graphical client.

The implemented source foundation covers bounded job identity/state contracts, sensitive URL redaction in debug output, progress invariants, final-file-promotion gating, remote-validator decisions, HTTP full/restart/resume request planning, safe range-response disposition, schema compatibility, deterministic partial-file naming, checkpoint/recovery planning, partial-file reconciliation, final-artifact revalidation, and generation-based atomic mutation-batch boundaries. These foundations are not a complete download engine.

## Product direction

The implementation prioritizes a durable local transfer core before multipart acceleration, browser interception, remote control, synchronization, or broader ecosystem integrations. BitTorrent and magnet transfers remain planned for delegation to GoreeCloud Swarm rather than duplicating its protocol stack.

## Source foundation

The implementation uses a Rust workspace pinned to Rust 1.98.1:

- `crates/download-core` — portable shared job/state/resume-domain contracts;
- `crates/download-http` — dependency-free HTTP resume-safety request/response contracts with no network I/O;
- `crates/download-state` — dependency-free durable-state and recovery contracts with no persistence or filesystem I/O;
- `crates/gcdm` — minimal Development-stage CLI shell.

CI validates formatting, Clippy, tests, native workspace checks, repository policy, and portable shared-core compilation for the Android target. A successful target compilation is not an Android application acceptance result.

## Documentation

- [Product specification](SPECIFICATIONS.md)
- [Feature roadmap](FEATURE-ROADMAP.md)
- [Current feature state](FEATURES.md)
- [Architecture](ARCHITECTURE.md)
- [ADR-0001 — Rust shared core](docs/architecture/ADR-0001-rust-shared-core.md)
- [ADR-0002 — Durable job state and recovery contracts](docs/architecture/ADR-0002-durable-job-state-and-recovery-contracts.md)
- [User manual](USER-MANUAL.md)
- [Privacy policy](PRIVACY%20POLICY.md)
- [Security guidance](SECURITY.md)
- [Benefits](BENEFITS.md)
- [Competitive objectives](COMPETITIVE-OBJECTIVES.md)
- [Branding](BRANDING.md)
- [Repository notes](NOTES.md)
- [Changelog](CHANGELOG.md)

## Platform contract

`goreecloud.platform.yaml` records the current GoreeCloud Platform Contract declaration. All seven Integral Platform Systems remain unaccepted for this application; metadata does not establish integration.

## Development boundaries

The Rust core/runtime direction is selected for the shared engine foundation, and backend-neutral HTTP resume and durable-state/recovery contracts now exist. The persistence backend, serialization/migration implementation, real HTTP client, HTTP/3, IPC/API technology, graphical client frameworks, credential storage, packaging, signing, and production recovery mechanisms remain separate implementation decisions and evidence gates. No platform is treated as supported or Stable merely because portable source compiles for it.
