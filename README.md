<!-- File internal version: v0.3 -->
# GoreeCloud Advanced Download Manager

**Product internal version:** 0.1.0  
**Release lifecycle:** Development  
**Repository state:** Initial Rust source foundation; no supported download workflow or installable client is currently verified.

GoreeCloud Advanced Download Manager is the developing cross-platform download and transfer-orchestration application for Linux, Windows, and Android. Its target is a local-first common download engine with durable transfers, intelligent acceleration, queues, automation, privacy and security controls, browser integration, and authorized GoreeCloud ecosystem coordination.

## Current availability

There is currently **no supported build for ordinary download-manager use**. The repository now contains an initial Rust workspace with a shared core-domain crate and a minimal `gcdm` command-line shell, but it does not yet perform network downloads, persist jobs, or provide a supported graphical client.

The implemented source foundation currently covers bounded job identity/state contracts, sensitive URL redaction in debug output, progress invariants, final-file-promotion gating, and remote-validator decisions for safe future resume behavior. These foundations are not a complete download engine.

## Product direction

The implementation prioritizes a durable local transfer core before multipart acceleration, browser interception, remote control, synchronization, or broader ecosystem integrations. BitTorrent and magnet transfers remain planned for delegation to GoreeCloud Swarm rather than duplicating its protocol stack.

## Source foundation

The initial implementation uses a Rust workspace pinned to Rust 1.98.1. `crates/download-core` is the portable shared core foundation; `crates/gcdm` is a minimal Development-stage CLI shell. CI is intended to validate formatting, Clippy, tests, native workspace checks, repository policy, and an Android shared-core compilation target.

## Documentation

- [Product specification](SPECIFICATIONS.md)
- [Feature roadmap](FEATURE-ROADMAP.md)
- [Current feature state](FEATURES.md)
- [Architecture](ARCHITECTURE.md)
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

The Rust core/runtime direction is now selected for the shared engine foundation. Persistence, HTTP implementation, HTTP/3, IPC/API technology, graphical client frameworks, credential storage, packaging, signing, and production recovery mechanisms remain separate implementation decisions and evidence gates. No platform is treated as supported or Stable merely because the shared core compiles for it.
