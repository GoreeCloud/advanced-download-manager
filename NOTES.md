---
title: "GoreeCloud Advanced Download Manager — Repository Notes"
document_type: "Repository Notes"
version: "v0.3"
product_version: "0.1.0"
release_lifecycle: "Development"
status: "Active"
classification: "Internal"
last_updated: "2026-09-15"
---

# GoreeCloud Advanced Download Manager — Repository Notes

## Verified current state

The repository is in **Development** with a bounded Rust shared-core source foundation. `SPECIFICATIONS.md` and `FEATURE-ROADMAP.md` remain authoritative for planned scope and delivery sequencing. The current source includes portable core job/state contracts, HTTP resume-safety contracts, backend-neutral durable-state/recovery contracts, a minimal CLI shell, repository validation, and cross-platform CI definitions.

No network transfer engine, durable persistence backend, serialization layer, graphical client, installable release, production deployment, or representative runtime acceptance is verified yet.

Product internal version remains **0.1.0**. Development lifecycle reflects active source implementation and does not imply Release Candidate, Stable, or production readiness.

## Current decisions

- The common shared-core implementation direction is Rust, pinned to toolchain 1.98.1 for the current foundation.
- Core operation is local-first and must not depend on an account.
- `crates/download-core` owns portable core-domain contracts.
- `crates/download-http` owns transport-independent HTTP full/restart/resume safety decisions and performs no network I/O.
- `crates/download-state` owns backend-neutral schema, checkpoint, staging-path, atomic mutation-batch, and restart-recovery contracts and performs no persistence or filesystem I/O.
- `crates/gcdm` is currently only a narrow Development CLI shell.
- Durable single-stream HTTP/HTTPS downloading precedes multipart acceleration.
- Specialized BitTorrent/magnet work is delegated to GoreeCloud Swarm.
- Remote, browser, synchronization, and ecosystem integrations are later layers and must not become dependencies of the basic local transfer engine.
- All seven Integral Platform Systems remain evaluated as applicable but blocked/unaccepted for this application.

## Open architecture work

The next bounded implementation decision is the actual transaction-safe persistence backend and serialization/migration adapter that will satisfy ADR-0002. Filesystem checkpoint ordering, synchronization, locking, corruption handling, and restart fixtures must be validated before runtime durability is claimed. Actual HTTP/HTTPS client technology and the first single-stream transport path remain pending after or alongside that durable-state adapter.

IPC/API technology, platform UI frameworks, Android bindings, credential storage, packaging/signing, broader dependency/security scanning, and production recovery mechanics remain open.

## Repository hygiene

Merged documentation and implementation branches are cleanup candidates after their work is fully merged and verified. Branch existence is not implementation, release, or acceptance evidence. The current GitHub connector does not expose branch deletion, so branch cleanup remains an explicit follow-up rather than being represented as completed.

The live `main` branch is currently reported by GitHub as unprotected. GoreeCloud repository policy expects protection for the default authoritative branch; configuration remains a repository-governance follow-up because the currently available connector does not expose branch-protection mutation.
